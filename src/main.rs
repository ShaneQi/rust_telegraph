extern crate handlebars;
extern crate pulldown_cmark;
extern crate yaml_rust;

use pulldown_cmark::{Parser, html};
use yaml_rust::YamlLoader;
use handlebars::{Handlebars, no_escape};
use std::fs::{create_dir_all, File};
use std::path::Path;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::BTreeMap;

fn main() {

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    let _ = handlebars.register_template_file("template", "./template.hbs");

    let path = Path::new("./posts/");
    let file_paths = path.read_dir().expect("");
    for dir_item in file_paths {
        let path = dir_item.expect("").path();
        let data = read_file(path);
        let html = handlebars.render("template", &data).expect("");
        let file_name = data["title"].as_str().to_string().replace(" ", "-");
        let _ = create_dir_all(&format!("./{}/", file_name));
        File::create(&format!("./{}/index.html", file_name))
            .and_then(|mut file| file.write_all(&html.as_bytes()))
            .expect("");
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> BTreeMap<String, String> {
    let file = File::open(path).expect("");
    let buf = BufReader::new(file);
    let mut yaml = String::new();
    let mut markdown = String::new();
    let mut yaml_began: Option<bool> = Option::None;
    for line in buf.lines() {
        let this_line = line.expect("");
        if this_line == "---" {
            match yaml_began {
                Some(true) => yaml_began = Some(false),
                _ => yaml_began = Some(true),
            }
        } else {
            match yaml_began {
                Some(true) => {
                    yaml = yaml + &this_line;
                    yaml = yaml + "\n";
                }
                Some(false) => {
                    markdown = markdown + &this_line;
                    markdown = markdown + "\n";
                }
                _ => (),
            }
        }
    }

    let mut content = String::new();
    let parser = Parser::new(&markdown);
    html::push_html(&mut content, parser);

    let yamls = YamlLoader::load_from_str(&yaml).expect("");
    let yaml_map = &yamls[0];
    let mut data = BTreeMap::new();
    data.insert(
        "title".to_string(),
        yaml_map["title"].as_str().expect("").to_string(),
    );
    data.insert(
        "date".to_string(),
        yaml_map["date"].as_str().expect("").to_string(),
    );
    data.insert("content".to_string(), content);
    return data;
}