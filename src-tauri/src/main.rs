// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use db::*;
use html::*;
use parse_utils::{parse_property, parse_selector};
use serde::Deserialize;
use std::fs;
use std::rc::Rc;

mod db;
mod html;
mod parse_utils;
mod properties;

#[tauri::command]
fn search(q: &str) -> Vec<String> {
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

    results
}

#[tauri::command]
fn render_rule(selector: &str) -> String {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector).unwrap();
    let path = selector.to_css_db_path();
    let tree = db.get(&path).unwrap();
    let rule = tree.rule.as_ref().unwrap();
    let mut rule_properties = rule.properties.clone();
    let i_p = db.inherited_properties_for(&path);
    let mut inherited_properties = i_p.values().collect::<Vec<_>>();
    let i_v = db.inherited_vars_for(&path, &i_p);
    let mut inherited_vars = i_v.values().collect::<Vec<_>>();
    rule_properties.sort_by_key(|p| p.name());
    inherited_properties.sort_by_key(|(_, p)| p.name());
    inherited_vars.sort_by_key(|(_, p)| p.name());

    fn link_for(selector_str: &String, property: &Rc<Property>) -> String {
        assert!(!selector_str.contains('\''));
        let selector = selector_str.trim();
        format!(
            "<a href='{}' title='{}'>{}</a>",
            selector,
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

#[tauri::command]
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

#[tauri::command]
fn disable(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector).unwrap().to_selector(None);
    db.set_state(&selector.path, name, value, State::Commented);
    fs::write("test.css", db.serialize()).unwrap()
}

#[tauri::command]
fn enable(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector).unwrap().to_selector(None);
    db.set_state(&selector.path, name, value, State::Valid);
    fs::write("test.css", db.serialize()).unwrap()
}

#[tauri::command]
fn get_all_properties() -> Vec<&'static str> {
    return properties::ALL_PROPERTIES.to_vec();
}

#[tauri::command]
fn insert_property(selector: &str, property: &str) {
    let property = parse_property(property).unwrap();
    let mut db = CSSDB::new();
    db.load("test.css");
    db.insert(
        &parse_selector(selector).unwrap().to_selector(None),
        &property,
    );
    fs::write("test.css", db.serialize()).unwrap()
}

#[derive(Deserialize)]
struct JsonProperty {
    is_commented: bool,
    name: String,
    value: String,
}

#[tauri::command]
fn replace_all_properties(selector: &str, properties: Vec<JsonProperty>) {
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

#[tauri::command(rename_all = "snake_case")]
fn update_value(selector: &str, name: &str, original_value: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let property = parse_property(&format!("{}: {};", name, value)).unwrap();
    let selector = parse_selector(selector).unwrap().to_selector(None);

    db.delete(&selector.path, name.trim(), original_value);
    db.insert(&selector, &property);

    fs::write("test.css", db.serialize()).unwrap()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            render_rule,
            search,
            delete,
            enable,
            disable,
            get_all_properties,
            insert_property,
            replace_all_properties,
            update_value
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
