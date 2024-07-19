// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use db::*;
use html::*;
use parse_utils::{parse_one, parse_property, parse_selector};
use serde::{Deserialize, Serialize};
use std::{fs, sync::Mutex};
use tauri::InvokeError;

mod db;
mod html;
mod parse_utils;

fn render_keyframes_selector(name: &str) -> String {
    format!(
        "<div data-kind=\"keyframes-selector\" data-string-value=\"@keyframes {}\">
                            <div data-attr=\"name\">{}</div>
                        </div>",
        name,
        render_value(name)
    )
}

#[derive(Serialize, Debug)]
pub enum CharismaError {
    DbLocked,
    ParseError,
    FailedToSave,
}

#[tauri::command]
fn search(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    q: &str,
) -> Result<Vec<String>, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path);
    }
    let parts: Vec<&str> = q.trim().split(' ').collect();

    let mut results: Vec<String> = db
        .all_selectors_with_properties()
        .iter()
        .filter(|selector| parts.iter().all(|q| selector.contains(q)))
        .cloned()
        .collect();

    results.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));

    let results: Result<Vec<String>, _> = results
        .iter()
        .map(|s| -> Result<String, CharismaError> {
            if s.starts_with('@') {
                match s.split("@keyframes").skip(1).next() {
                    Some(name) => Ok(render_keyframes_selector(name.trim())),
                    None => Err(CharismaError::ParseError),
                }
            } else {
                match parse_selector(s).and_then(|s| s.into_iter().next()) {
                    Some(selector) => selector
                        .map_err(|_| CharismaError::ParseError)
                        .map(|s| s.render_html(&RenderOptions::default())),
                    None => Err(CharismaError::ParseError),
                }
            }
        })
        .collect();

    Ok(results?)
}

#[tauri::command]
fn insert_empty_rule(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path);
    }
    if selector.starts_with("@keyframes") {
        match selector.split("@keyframes").skip(1).next() {
            Some(name) => db.insert_empty_keyframes_rule(name.trim().to_string()),
            None => return Err(CharismaError::ParseError.into()),
        }
    } else {
        match parse_selector(selector) {
            Some(selector_list) => match selector_list
                .into_iter()
                .map(|s| {
                    s.map_err(|_| CharismaError::ParseError)
                        .map(|s| s.to_selectors(None))
                })
                .collect::<Result<Vec<Vec<Selector>>, _>>()
            {
                Ok(selectors) => selectors
                    .iter()
                    .flatten()
                    .for_each(|selector| db.insert_empty_regular_rule(selector)),
                Err(e) => return Err(e.into()),
            },
            None => return Err(CharismaError::ParseError.into()),
        }
    }
    Ok(fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave)?)
}

#[tauri::command]
fn render_rule(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str) -> String {
    let mut db = state.lock().unwrap();
    if !db.is_loaded(path) {
        db.load(path);
    }
    if selector.starts_with("@keyframes") {
        let name = selector.split("@keyframes").skip(1).next().unwrap().trim();
        let path = [
            Part::AtRule(AtRulePart::Keyframes),
            Part::AtRule(AtRulePart::Name(name.to_string())),
        ];

        let tree = db.get(&path).unwrap();
        let keyframes_rule = tree.rule.as_ref().and_then(|r| r.as_keyframes()).unwrap();

        format!(
            "
            <div data-kind=\"keyframes-rule\">
                <div data-attr=\"selector\">{}</div>
                <div data-attr=\"frames\">{}</div>
            </div>
            ",
            render_keyframes_selector(name),
            keyframes_rule
                .frames
                .iter()
                .map(|frame| frame.render_html(&RenderOptions::default()))
                .collect::<String>()
        )
    } else {
        let selector = parse_selector(selector).unwrap();
        let paths: Vec<Vec<Part>> = selector
            .into_iter()
            .flat_map(|s| s.unwrap().to_css_db_paths())
            .collect();
        assert!(paths.len() == 1);
        let path = paths.first().unwrap();
        let tree = db.get(path).unwrap();
        let rule = tree.rule.as_ref().unwrap().as_regular_rule().unwrap();
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
}

#[tauri::command]
fn delete(state: tauri::State<Mutex<CSSDB>>, path: &str, selector: &str, name: &str, value: &str) {
    let mut db = state.lock().unwrap();
    if !db.is_loaded(path) {
        db.load(path);
    }

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
    if !db.is_loaded(path) {
        db.load(path);
    }

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
    if !db.is_loaded(path) {
        db.load(path);
    }

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
    if !db.is_loaded(path) {
        db.load(path);
    }
    let property = parse_property(property).unwrap();
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.insert_regular_rule(&selector, &property);
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
    if !db.is_loaded(path) {
        db.load(path);
    }
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        db.get_mut(&selector.path).unwrap().drain();

        for property in properties.iter() {
            if property.is_commented {
                db.insert_regular_rule_commented(
                    &selector,
                    // at this point we are just parsing to validate
                    parse_property(&format!("{}: {};", property.name, property.value)).unwrap(),
                );
            } else {
                db.insert_regular_rule(
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
    if !db.is_loaded(path) {
        db.load(path);
    }

    let property = parse_property(&format!("{}: {};", name, value)).unwrap();
    for selector in parse_selector(selector)
        .unwrap()
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        let tree = db.get(&selector.path).unwrap();
        let rule = tree.rule.as_ref().unwrap().as_regular_rule().unwrap();
        assert!(rule
            .properties
            .iter()
            .any(|p| p.name == name.trim() && p.value == original_value));
        db.delete(&selector.path, name.trim(), original_value);
        db.insert_regular_rule(&selector, &property);
    }

    fs::write(path, db.serialize()).unwrap()
}

#[tauri::command(rename_all = "snake_case")]
fn load_rule(state: tauri::State<Mutex<CSSDB>>, path: &str, rule: &str) -> String {
    let mut db = state.lock().unwrap();
    if !db.is_loaded(path) {
        db.load(path);
    }

    let rule = parse_one(rule).unwrap();

    let selector = rule.prelude();
    let block = rule.block().unwrap();
    let block = block.as_css_declaration_or_rule_block().unwrap();

    for selector in (&selector)
        .into_iter()
        .flat_map(|s| s.unwrap().to_selectors(None))
    {
        for item in block.items() {
            let property = item.as_css_declaration_with_semicolon().unwrap();
            db.insert_regular_rule(&selector, &property);
        }
    }

    fs::write(path, db.serialize()).unwrap();

    selector
        .into_iter()
        .map(|s| s.unwrap().to_string())
        .collect::<String>()
        .trim()
        .to_string()
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
            load_rule,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
