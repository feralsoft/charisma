// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use biome_css_syntax::AnyCssSelector;
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

fn after_css_comment(string: String) -> String {
    if let Some(idx) = string.find("*/") {
        string.get((idx + 2)..).unwrap().to_string()
    } else {
        string
    }
}

#[tauri::command]
fn search(q: &str) -> Vec<String> {
    let mut db = CSSDB::new();
    db.load("test.css");

    let parts: Vec<&str> = q.trim().split(" ").collect();

    let mut results: Vec<AnyCssSelector> = db
        .all_selectors_with_properties()
        .iter()
        // .unwrap() since, it should never crash
        .flat_map(|s| parse_selector(s).unwrap())
        .map(|s| s.unwrap())
        .filter(|selector| {
            let str = after_css_comment(selector.to_string());
            parts.iter().all(|q| str.contains(q))
        })
        .collect();

    results.sort_by(|a, b| {
        let a = after_css_comment(a.to_string());
        let b = after_css_comment(b.to_string());
        a.len().cmp(&b.len()).then_with(|| a.cmp(&b))
    });

    results
        .iter()
        .map(|s| s.render_html(&RenderOptions::default()))
        .collect()
}

#[tauri::command]
fn insert_empty_rule(selector: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector_list = parse_selector(selector).unwrap();
    for selector in selector_list
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.insert_empty(&selector);
    }
    fs::write("test.css", db.serialize()).unwrap()
}

#[tauri::command]
fn render_rule(selector: &str) -> String {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector = parse_selector(selector).unwrap();
    let paths: Vec<Vec<String>> = selector
        .into_iter()
        .flat_map(|s| s.unwrap().to_css_db_paths())
        .collect();
    assert!(paths.len() == 1);
    let path = paths.first().unwrap();
    let tree = db.get(&path).unwrap();
    let rule = tree.rule.as_ref().unwrap();
    let mut rule_properties = rule.properties.clone();
    let inherited_properties_map = db.inherited_properties_for(&path);
    let mut inherited_properties = inherited_properties_map.values().collect::<Vec<_>>();
    let inherited_vars_map = db.inherited_vars_for(&path, &inherited_properties_map);
    let mut inherited_vars = inherited_vars_map.values().collect::<Vec<_>>();
    rule_properties.sort_by_key(|p| p.name());
    inherited_properties.sort_by_key(|(_, p)| p.name());
    inherited_vars.sort_by_key(|(_, p)| p.name());

    fn link_for(selector_str: &String, property: &Rc<Property>) -> String {
        assert!(!selector_str.contains('\''));
        let selector = selector_str.trim();
        format!(
            "<a href='{}'>{}</a>",
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
            .map(|(selector, p)| link_for(&selector.string, p))
            .collect::<String>()
            + &inherited_vars
                .iter()
                .map(|(selector, p)| link_for(&selector.string, p))
                .collect::<String>()
    )
}

#[tauri::command]
fn delete(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let selector_list = parse_selector(selector).unwrap();

    for path in selector_list
        .into_iter()
        .flat_map(|s| s.unwrap().to_css_db_paths())
    {
        db.delete(&path, name, value);
    }
    fs::write("test.css", db.serialize()).unwrap()
}

#[tauri::command]
fn disable(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.set_state(&selector.path, name, value, State::Commented);
    }
    fs::write("test.css", db.serialize()).unwrap()
}

#[tauri::command]
fn enable(selector: &str, name: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.set_state(&selector.path, name, value, State::Valid);
    }
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
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.insert(&selector, &property);
    }
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
    let mut db = CSSDB::new();
    db.load("test.css");
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
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
    }
    fs::write("test.css", db.serialize()).unwrap()
}

#[tauri::command(rename_all = "snake_case")]
fn update_value(selector: &str, name: &str, original_value: &str, value: &str) {
    let mut db = CSSDB::new();
    db.load("test.css");
    let property = parse_property(&format!("{}: {};", name, value)).unwrap();
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.delete(&selector.path, name.trim(), original_value);
        db.insert(&selector, &property);
    }

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
            update_value,
            insert_empty_rule
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
