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
    fs::read_to_string("src/insert_property.js").unwrap()
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct InsertPayload {
    name: String,
    value: String,
}

#[post("/src/<selector>/insert", data = "<data>")]
fn insert(selector: String, data: Json<InsertPayload>) {
    let mut db = DBTree::new();
    db.load("test.css");
    let selector = parse_selector(&selector);
    let parts = selector.to_path_parts();
    db.insert_mut(selector, &parts, &data.name, &data.value);
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
            <script>{}</script>
            <div class=\"--editor\" spellcheck=\"false\">{}</div>",
            css(),
            insert_property_js(),
            code_html
        ),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, insert])
}
