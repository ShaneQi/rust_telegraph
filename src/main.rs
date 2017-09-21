#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate handlebars;

extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate hyper_tls;

use handlebars::Handlebars;
use handlebars::{RenderContext, RenderError, Helper};
use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;

use futures::{Future, Stream};
use hyper::{Client, Chunk};
use tokio_core::reactor::Core;
use hyper_tls::HttpsConnector;

fn helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let param = h.param(0).unwrap().value();
    if let Some(str) = param.as_str() {
        let rendered = format!("{}", str);
        try!(rc.writer.write(rendered.into_bytes().as_ref()));
    }
    Ok(())
}

fn read_list() -> Vec<Page> {
    let mut f = File::open("./content/list.json").expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect(
        "something went wrong reading the file",
    );
    serde_json::from_str::<Vec<Page>>(&contents).unwrap()
}

fn get_json(page_path: String) -> serde_json::Value {
    let mut core = Core::new().unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let uri = format!(
        "https://api.telegra.ph/getPage/{}?return_content=true",
        page_path
    ).parse()
        .unwrap();
    let work = client.get(uri).and_then(|res| {
        res.body().concat2().and_then(|body: Chunk| {
            serde_json::from_slice::<serde_json::Value>(&body).map_err(|_| hyper::Error::Incomplete)
        })
    });
    let object = core.run(work);
    return object.unwrap();
}

fn main() {
    let list = read_list();
    let mut handlebars = Handlebars::new();
    handlebars.register_helper("plain", Box::new(helper));
    // register template using given name
    if let Err(e) = handlebars.register_template_file("list", "./list.hbs") {
        panic!("{}", e);
    }
    if let Err(e) = handlebars.register_template_file("table", "./template.hbs") {
        panic!("{}", e);
    }
    if let Err(e) = handlebars.register_template_file("page", "./page.hbs") {
        panic!("{}", e);
    }

    let list_html = handlebars.render("list", &json!(&list)).unwrap();
    let mut file = File::create(Path::new("./index.html")).unwrap();
    file.write_all(&list_html.as_bytes()).unwrap();

    for page in list {

        let page_path = &page.page_path;
        let file_name = page.permalink.unwrap_or(page_path.to_string());
        let file = format!("./{}/index.html", file_name);
        let path = Path::new(&file);
        let json = get_json(page_path.to_string());
        let result = handlebars.render("table", &json).unwrap();

        fs::create_dir(Path::new(&format!("./{}", file_name)));
        let mut file = match File::create(&path) {
            Err(_) => panic!("couldn't create {}", file_name),
            Ok(file) => file,
        };

        print!("{}", result);
        match file.write_all(&result.as_bytes()) {
            Err(_) => panic!("couldn't write to {}", file_name),
            Ok(_) => println!("successfully wrote to {}", file_name),
        }
    }


}

#[derive(Debug, Deserialize, Serialize)]
struct Page {
    title: Option
    permalink: Option<String>,
    page_path: String,
}
