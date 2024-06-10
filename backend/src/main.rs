#[macro_use]
extern crate rocket;

use std::fs;

use crate::storage::*;
use biome_css_parser::CssParserOptions;
use html::Render;
use parse_utils::parse_selector;
use rocket::http::ContentType;

mod html;
mod parse_utils;
mod storage;

fn css() -> String {
    fs::read_to_string("src/index.css").unwrap()
}

#[get("/src/<selector>")]
async fn index(selector: &str) -> (ContentType, String) {
    let mut db = DBTree::new();
    db.load("db.css");
    let selector = parse_selector(&selector.to_string());
    println!("{:?}", &selector.to_path_parts());
    let tree = db.get(&selector.to_path_parts()).unwrap();
    let out = biome_css_parser::parse_css(&tree.serialize(), CssParserOptions::default());
    let code_html = out.tree().render_html();
    (
        ContentType::HTML,
        format!(
            "<style>{}</style>
            <div class=\"--editor\" spellcheck=\"false\">{}</div>",
            css(),
            code_html
        ),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
