#[macro_use]
extern crate rocket;

use std::{fs, rc::Rc};

use db::*;
use html::{Render, RenderOptions};
use parse_utils::{parse_property, parse_selector};
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use rocket::{http::ContentType, serde::Deserialize};
use url;

mod db;
mod html;
mod parse_utils;
mod properties;

fn css() -> String {
    fs::read_to_string("src/index.css").unwrap()
}

const JS_FILE_NAMES: [&str; 11] = [
    "insert_property",
    "toggle_property",
    "delete_property",
    "update_value",
    "preview_var",
    "search",
    "draggable_editor",
    "multi_editor",
    "menu",
    "focus",
    "undo",
];

fn editor_js() -> String {
    let all_properties = format!(
        "<script type=\"application/json\" id=\"css-properties\">[{}]</script>",
        properties::ALL_PROPERTIES
            .iter()
            .map(|p| format!("\"{}\",", p))
            .collect::<String>()
    );

    JS_FILE_NAMES
        .iter()
        .map(|name| fs::read_to_string(format!("src/js/{}.js", name)).unwrap())
        .map(|js| format!("<script type=\"module\">{}</script>", js))
        .collect::<String>()
        + &all_properties
}

#[post("/src/<selector>", data = "<property>")]
fn insert(selector: &str, property: &str) {
    let property = parse_property(property).unwrap();
    let mut db = CSSDB::new();
    db.load("test.css");
    db.insert(
        &parse_selector(selector).unwrap().to_selector(None),
        &property,
    );
    fs::write("test.css", db.serialize()).unwrap()
}
#[get("/search/<q>")]
fn search(q: &str) -> (ContentType, Json<Vec<String>>) {
    let mut db = CSSDB::new();
    db.load("test.css");

    let parts: Vec<_> = q.trim().split(" ").collect();

    let mut results = db
        .all_selectors()
        .iter()
        .filter(|selector| parts.iter().all(|q| selector.contains(q)))
        .map(|s| {
            parse_selector(s)
                .unwrap()
                .render_html(&RenderOptions::default())
        })
        .collect::<Vec<String>>();

    results.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));

    (ContentType::JSON, Json::from(results))
}

#[post("/src/<selector>/<name>/<value>/disable")]
fn disable(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector).unwrap().to_selector(None);
    db.set_state(&selector.path, name, value, State::Commented);
    fs::write("test.css", db.serialize()).unwrap()
}

#[delete("/src/<selector>/<name>/<value>")]
fn delete(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");

    db.delete(
        &parse_selector(selector).unwrap().to_css_db_path(),
        name,
        value,
    );
    fs::write("test.css", db.serialize()).unwrap()
}

#[post("/src/<selector>/<name>/<value>/enable")]
fn enable(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector).unwrap().to_selector(None);
    db.set_state(&selector.path, name, value, State::Valid);
    fs::write("test.css", db.serialize()).unwrap()
}

#[post("/src/<selector>/<name>/<old_value>", data = "<value>")]
fn set_value(selector: &str, name: &str, old_value: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let property = parse_property(&format!("{}: {};", name, value)).unwrap();
    let selector = parse_selector(selector).unwrap().to_selector(None);

    db.delete(&selector.path, name, old_value);
    db.insert(&selector, &property);

    fs::write("test.css", db.serialize()).unwrap()
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct JsonProperty {
    is_commented: bool,
    name: String,
    value: String,
}

#[post("/src/<selector>/replace_all_properties", data = "<properties>")]
fn replace_all_properties(selector: &str, properties: Json<Vec<JsonProperty>>) {
    let selector = parse_selector(selector).unwrap().to_selector(None);
    let mut db = CSSDB::new();
    db.load("test.css");
    db.get_mut(&selector.path).unwrap().drain();

    for property in properties.iter() {
        if property.is_commented {
            db.insert_commented(
                &selector,
                parse_property(&format!("{}: {};", property.name, property.value)).unwrap(),
            );
        } else {
            db.insert(
                &selector,
                &parse_property(&format!("{}: {};", property.name, property.value)).unwrap(),
            );
        }
    }

    fs::write("test.css", db.serialize()).unwrap()
}

fn render_rule(selector: &str, db: &CSSDB) -> String {
    let selector = parse_selector(selector).unwrap();
    let path = selector.to_css_db_path();
    let tree = db.get(&path).unwrap();
    let rule = tree.rule.as_ref().unwrap();
    let mut rule_properties = rule.properties.clone();
    let i_p = db.inherited_properties_for(&path);
    let mut inherited_properties = i_p.values().collect::<Vec<_>>();
    let i_v = db.inherited_vars_for(&path);
    let mut inherited_vars = i_v.values().collect::<Vec<_>>();
    rule_properties.sort_by_key(|p| p.name());
    inherited_properties.sort_by_key(|(_, p)| p.name());
    inherited_vars.sort_by_key(|(_, p)| p.name());

    fn link_for(selector_str: &String, property: &Rc<Property>) -> String {
        assert!(!selector_str.contains('\''));
        let selector = selector_str.trim();
        format!(
            "<a href='{}' title='{}'>{}</a>",
            url::form_urlencoded::byte_serialize(selector.as_bytes()).collect::<String>(),
            selector,
            property.render_html(&RenderOptions::default())
        )
    }
    format!(
        "
    <div data-kind=\"rule\">
        <div data-attr=\"selector\">{}</div>
        <div data-attr=\"properties\">{}</div>
        <div data-attr=\"inherited-properties\">{}</div>
    </div>
    ",
        parse_selector(&rule.selector.string)
            .unwrap()
            .render_html(&RenderOptions::default()),
        rule_properties
            .iter()
            .map(|p| p.render_html(&RenderOptions::default()))
            .collect::<String>(),
        inherited_properties
            .iter()
            .map(|(selector, p)| link_for(selector, p))
            .collect::<String>()
            + &inherited_vars
                .iter()
                .map(|(selector, p)| link_for(selector, p))
                .collect::<String>()
    )
}

#[get("/src/<selector_str>/rule")]
fn rule(selector_str: &str) -> (ContentType, String) {
    let mut db = CSSDB::new();
    db.load("test.css");
    (ContentType::HTML, render_rule(selector_str, &db))
}

#[get("/src")]
fn index() -> (ContentType, String) {
    (
        ContentType::HTML,
        format!(
            "
            <!DOCTYPE html>
            <html>
                <head>
                    <title>Charisma</title>
                    <style>{}</style>
                    {}
                </head>
                <body>
                    <div class=\"search-box\">
                        <div class=\"search\" contenteditable spellcheck=\"false\"></div>
                    </div>
                    <div class=\"canvas\"></div>
                </body>
            </html>",
            css(),
            editor_js(),
        ),
    )
}

#[get("/favicon.ico")]
async fn favicon() -> Option<NamedFile> {
    NamedFile::open("src/public/favicon.ico").await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![
            index,
            favicon,
            rule,
            insert,
            set_value,
            enable,
            disable,
            delete,
            replace_all_properties,
            search,
        ],
    )
}
