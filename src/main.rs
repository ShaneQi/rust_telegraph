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
    let _ = handlebars.register_template_file("post", "./templates/post.hbs");
    let _ = handlebars.register_template_file("index", "./templates/index.hbs");

    let mut posts: Vec<BTreeMap<String, String>> = vec![];

    for dir_item in Path::new("./posts/").read_dir().expect("") {
        let path = dir_item.expect("").path();
        let data = read_file(path);
        let post_html = handlebars.render("post", &data).expect("");
        let perma_name = data["perma_name"].as_str().to_string();
        let _ = create_dir_all(&format!("./{}/", perma_name));
        File::create(&format!("./{}/index.html", perma_name))
            .and_then(|mut file| file.write_all(&post_html.as_bytes()))
            .expect("");
        posts.push(data);
    }

    posts.sort_by(|a, b| b["date"].as_str().cmp(a["date"].as_str()) );

    let index_post_html = handlebars.render("index", &posts).expect("");
    File::create("./index.html")
        .and_then(|mut file| file.write_all(&index_post_html.as_bytes()))
        .expect("");
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
    let title = yaml_map["title"].as_str().expect("").to_string();
    let date = yaml_map["date"].as_str().expect("");
    data.insert("date".to_string(), date.to_string());
    data.insert("title".to_string(), title.clone());
    data.insert(
        "perma_name".to_string(),
        title.replace(" ", "-") + "-" + &date.chars().skip(2).take(8).collect::<String>(),
    );
    data.insert("content".to_string(), content);
    return data;
}