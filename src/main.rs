extern crate handlebars;
extern crate pulldown_cmark;
extern crate yaml_rust;

use pulldown_cmark::{Parser, html};
use yaml_rust::YamlLoader;
use handlebars::{Handlebars, no_escape};
use std::fs::{self, File};
use std::path::Path;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::BTreeMap;
use std::env;

fn main() {
    // Get working paths.
    let mut input_path = env::args().nth(1).unwrap_or("./".to_string());
    let mut output_path = env::args().nth(2).unwrap_or("./".to_string());

    // Check if path has trailing slash.
    if input_path.chars().rev().nth(0).unwrap() != '/' {
        input_path += "/";
    }
    if output_path.chars().rev().nth(0).unwrap() != '/' {
        output_path += "/";
    }
    println!("Input path: {}", input_path);
    println!("Onput path: {}", output_path);

    // Config handlerbars.
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    let _ = handlebars.register_template_file("post", "./templates/post.hbs");
    let _ = handlebars.register_template_file("index", "./templates/index.hbs");

    // Process md files.
    let mut posts: Vec<BTreeMap<String, String>> = vec![];
    for dir_item in Path::new(&format!("{}posts/", input_path))
        .read_dir()
        .expect(&format!(
            "Failed to find post directory: {}posts/",
            input_path
        ))
    {
        let path = dir_item.expect("").path();
        if path.extension().and_then(|s| s.to_str()).unwrap_or("") != "md" {
            println!("Skipping file: {:?}", path);
            continue;
        }
        println!("Processing file: {:?}", path);
        let data = read_file(path);
        let post_html = handlebars.render("post", &data).expect("");
        let permalink = data["permalink"].as_str().to_string();
        let _ = fs::create_dir_all(&format!("{}{}/", output_path, permalink));
        File::create(&format!("{}{}/index.html", output_path, permalink))
            .and_then(|mut file| file.write_all(&post_html.as_bytes()))
            .expect("");
        posts.push(data);
    }
    posts.sort_by(|a, b| b["date"].as_str().cmp(a["date"].as_str()));

    // Generate homepage.
    let index_post_html = handlebars.render("index", &posts).expect("");
    File::create(&format!("{}index.html", output_path))
        .and_then(|mut file| file.write_all(&index_post_html.as_bytes()))
        .expect("");

    // Copy assets.
    let _ = fs::create_dir_all(&format!("{}css/", output_path));
    fs::copy(
        "./assets/css/telegraph.css",
        &format!("{}css/telegraph.css", output_path),
    ).expect("");
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

    // Parse markdown to html.
    let mut content = String::new();
    let parser = Parser::new(&markdown);
    html::push_html(&mut content, parser);

    // Parse yaml to post info.
    let yamls = YamlLoader::load_from_str(&yaml).expect("");
    let yaml_map = &yamls[0];
    let mut data = BTreeMap::new();
    let title = yaml_map["title"].as_str().expect("").to_string();
    let date = yaml_map["date"].as_str().expect("");
    data.insert("date".to_string(), date.to_string());
    data.insert("title".to_string(), title.clone());
    data.insert(
        "permalink".to_string(),
        yaml_map["permalink"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or(
                title.to_lowercase().replace(" ", "-") + "-" +
                    &date.chars().skip(2).take(8).collect::<String>(),
            ),
    );
    data.insert("content".to_string(), content);

    return data;
}