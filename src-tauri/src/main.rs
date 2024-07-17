// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use db::*;
use html::*;
use parse_utils::{parse_property, parse_selector};
use serde::Deserialize;
use std::{fs, sync::Mutex};

mod db;
mod html;
mod parse_utils;

#[tauri::command]
fn search(state: tauri::State<Mutex<CSSDB>>, path: &str, q: &str) -> Vec<String> {
    let mut db = state.lock().unwrap();
    if !db.is_loaded(path) {
        db.load(path);
    }
    let parts: Vec<&str> = q.trim().split(" ").collect();

    let mut results: Vec<String> = db
        .all_selectors_with_properties()
        .iter()
        .filter(|selector| parts.iter().all(|q| selector.contains(q)))
        .cloned()
        .collect();

    results.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(&b)));

    results
        .iter()
        .map(|s| {
            parse_selector(s)
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
                .unwrap()
        })
        .map(|s| s.render_html(&RenderOptions::default()))
        .collect()
}

#[tauri::command]
fn insert_empty_rule(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));
    let selector_list = parse_selector(selector).unwrap();
    for selector in selector_list
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.insert_empty(&selector);
    }
    fs::write(path, db.serialize()).unwrap()
}

#[tauri::command]
fn render_rule(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str) -> String {
    let db = state.lock().unwrap();
    assert!(db.is_loaded(path));
    let selector = parse_selector(selector).unwrap();
    let paths: Vec<Vec<Part>> = selector
        .into_iter()
        .flat_map(|s| s.unwrap().to_css_db_paths())
        .collect();
    assert!(paths.len() == 1);
    let path = paths.first().unwrap();
    let tree = db.get(&path).unwrap();
    let rule = tree.rule.as_ref().unwrap();
    let mut rule_properties = rule.properties.clone();
    rule_properties.sort_by_key(|p| p.name.clone());

    format!(
        "
    <div data-kind=\"rule\">
        <div data-attr=\"selector\">{}</div>
        <div data-attr=\"properties\">{}</div>
    </div>
    ",
        parse_selector(&rule.selector.string)
            .unwrap()
            .render_html(&RenderOptions::default()),
        rule_properties
            .iter()
            .map(|p| p.render_html(&RenderOptions::default()))
            .collect::<String>(),
    )
}

#[tauri::command]
fn delete(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str, name: &str, value: &str) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));

    let selector_list = parse_selector(selector).unwrap();

    for path in selector_list
        .into_iter()
        .flat_map(|s| s.unwrap().to_css_db_paths())
    {
        db.delete(&path, name, value);
    }
    fs::write(path, db.serialize()).unwrap()
}

#[tauri::command]
fn disable(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str, name: &str, value: &str) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));

    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.set_state(&selector.path, name, value, State::Commented);
    }
    fs::write(path, db.serialize()).unwrap()
}

#[tauri::command]
fn enable(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str, name: &str, value: &str) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));

    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.set_state(&selector.path, name, value, State::Valid);
    }
    fs::write(path, db.serialize()).unwrap()
}

#[tauri::command]
fn insert_property(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str, property: &str) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));
    let property = parse_property(property).unwrap();
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.insert(&selector, &property);
    }
    fs::write(path, db.serialize()).unwrap()
}

#[derive(Deserialize)]
struct JsonProperty {
    is_commented: bool,
    name: String,
    value: String,
}

#[tauri::command]
fn replace_all_properties(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    properties: Vec<JsonProperty>,
) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));
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
                    // at this point we are just parsing to validate
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
    fs::write(path, db.serialize()).unwrap()
}

#[tauri::command(rename_all = "snake_case")]
fn update_value(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    name: &str,
    original_value: &str,
    value: &str,
) {
    let mut db = state.lock().unwrap();
    assert!(db.is_loaded(path));

    let property = parse_property(&format!("{}: {};", name, value)).unwrap();
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        let tree = db.get(&selector.path).unwrap();
        let rule = tree.rule.as_ref().unwrap();
        assert!(rule
            .properties
            .iter()
            .find(|p| p.name == name.trim() && p.value == original_value)
            .is_some());
        db.delete(&selector.path, name.trim(), original_value);
        db.insert(&selector, &property);
    }

    fs::write(path, db.serialize()).unwrap()
}

fn main() {
    let db = Mutex::new(CSSDB::new());

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            render_rule,
            search,
            delete,
            enable,
            disable,
            insert_property,
            replace_all_properties,
            update_value,
            insert_empty_rule,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
