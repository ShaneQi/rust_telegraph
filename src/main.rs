extern crate handlebars;
extern crate pulldown_cmark;

use pulldown_cmark::{Parser, html};
use handlebars::{Handlebars, no_escape};
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::BTreeMap;

fn main() {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    // register template using given name
    if let Err(e) = handlebars.register_template_file("template", "./template.hbs") {
        panic!("{}", e);
    }

    let mut data = BTreeMap::new();
    data.insert("content".to_string(), read_file("post.md"));
    let list_html = handlebars.render("template", &data).unwrap();
    let mut file = File::create(Path::new("./index.html")).unwrap();
    file.write_all(&list_html.as_bytes()).unwrap();

}

fn read_file(file_name: &str) -> String {
    let mut file = File::open(file_name).expect("");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("");
    let mut s = String::new();
    let p = Parser::new(&contents);
    html::push_html(&mut s, p);
    return s;
}
