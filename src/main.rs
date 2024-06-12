#[macro_use]
extern crate rocket;

use std::fs;

use db::*;
use html::Render;
use parse_utils::{parse_property, parse_selector};
use rocket::{http::ContentType, serde::json::Json};

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

fn explore_siblings_js() -> String {
    fs::read_to_string("src/js/explore_siblings.js").unwrap()
}

#[post("/src/<selector>", data = "<property>")]
fn insert(selector: &str, property: &str) {
    println!("{:?}", property);
    let property = parse_property(property);
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector);
    let path = selector.to_css_db_path();
    db.insert(selector, &path, property);
    fs::write("test.css", db.serialize()).unwrap()
}

#[delete("/src/<selector>/<name>")]
fn delete(selector: &str, name: String) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector);
    let path = selector.to_css_db_path();
    db.delete(&path, &name);
    fs::write("test.css", db.serialize()).unwrap()
}

#[get("/src/<selector>/siblings")]
fn siblings(selector: &str) -> (ContentType, Json<Vec<Vec<(String, String)>>>) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector);
    let path = selector.to_css_db_path();
    let siblings = db
        .siblings_for(&path)
        .iter()
        .map(|tree| tree.rule.as_ref().unwrap())
        .map(|rule| {
            rule.selector
                .to_css_db_path()
                .iter()
                .map(|part| {
                    let selector_html = if part == " " {
                        "so sad, what to do here :(".to_owned()
                    } else {
                        parse_selector(&part).render_html()
                    };
                    (part.to_owned(), selector_html)
                })
                .collect::<Vec<(String, String)>>()
        })
        .collect::<Vec<Vec<(String, String)>>>();

    (ContentType::JSON, Json::from(siblings))
}

#[get("/src/<selector>")]
fn index(selector: String) -> (ContentType, String) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(&selector);
    let path = selector.to_css_db_path();
    let tree = db.get(&path).unwrap();
    let rule = tree.rule.as_ref().unwrap();
    let inherited_properties = db.inherited_properties_for(&path);
    let inherited_vars = db.inherited_vars_for(&path);
    let rule_html = format!(
        "
    <div data-kind=rule>
        <div class=options>
            <label>inherited properties<input type=checkbox class=show-inherited-properties checked></label>
            <label>inherited vars<input type=checkbox class=show-inherited-vars checked></label>
        </div>
        <div data-attr=selector>{}</div>
        <div data-attr=properties>{}</div>
        <div data-attr=inherited-properties>{}</div>
        <div data-attr=inherited-vars>{}</div>
    </div>
    ",
        rule.selector.render_html(),
        rule.properties
            .iter()
            .map(|p| p.render_html())
            .collect::<String>(),
        inherited_properties
            .iter()
            .map(|(_, p)| p.render_html())
            .collect::<String>(),
        inherited_vars
            .iter()
            .map(|(_, p)| p.render_html())
            .collect::<String>()
    );

    (
        ContentType::HTML,
        format!(
            "<style>{}</style>
            <script>{}{}{}</script>
            <div class=\"--editor\" spellcheck=\"false\">{}<div>",
            css(),
            insert_property_js(),
            delete_property_js(),
            explore_siblings_js(),
            rule_html
        ),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, insert, delete, siblings])
}
