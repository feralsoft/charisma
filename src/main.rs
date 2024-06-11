#[macro_use]
extern crate rocket;

use std::fs;

use biome_css_parser::CssParserOptions;
use db::*;
use html::Render;
use parse_utils::parse_selector;
use rocket::{
    http::ContentType,
    serde::{json::Json, Deserialize},
};

mod db;
mod html;
mod parse_utils;

fn css() -> String {
    fs::read_to_string("src/index.css").unwrap()
}

fn insert_property_js() -> String {
    fs::read_to_string("src/js/insert_property.js").unwrap()
}

fn delete_property_js() -> String {
    fs::read_to_string("src/js/delete_property.js").unwrap()
}

#[post("/src/<selector>/<name>", data = "<value>")]
fn insert(selector: String, name: String, value: Json<String>) {
    let mut db = DBTree::new();
    db.load("test.css");
    let selector = parse_selector(&selector);
    let parts = selector.to_path_parts();
    db.insert_mut(selector, &parts, &name, &value);
    fs::write("test.css", db.serialize()).unwrap()
}

#[delete("/src/<selector>/<name>")]
fn delete(selector: String, name: String) {
    let mut db = DBTree::new();
    db.load("test.css");
    let selector = parse_selector(&selector);
    let path = selector.to_path_parts();
    db.delete_mut(&path, &name);
    fs::write("test.css", db.serialize()).unwrap()
}

#[get("/src/<selector>")]
fn index(selector: String) -> (ContentType, String) {
    let mut db = DBTree::new();
    db.load("test.css");
    let selector = parse_selector(&selector);
    let tree = db.get(&selector.to_path_parts()).unwrap();
    let out = biome_css_parser::parse_css(&tree.serialize(), CssParserOptions::default());
    let code_html = out.tree().render_html();
    (
        ContentType::HTML,
        format!(
            "<style>{}</style>
            <script>{}{}</script>
            <div class=\"--editor\" spellcheck=\"false\">{}</div>",
            css(),
            insert_property_js(),
            delete_property_js(),
            code_html
        ),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, insert, delete])
}
