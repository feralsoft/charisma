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
    RuleNotFound,
    AssertionError(String),
}

#[tauri::command]
fn search(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    q: &str,
) -> Result<Vec<String>, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
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
                        .and_then(|s| s.render_html(&RenderOptions::default())),
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
        db.load(path)?;
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
fn render_rule(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
) -> Result<String, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    if selector.starts_with("@keyframes") {
        let name = match selector.split("@keyframes").skip(1).next() {
            Some(name) => name.trim(),
            None => return Err(CharismaError::ParseError.into()),
        };

        let path = [
            Part::AtRule(AtRulePart::Keyframes),
            Part::AtRule(AtRulePart::Name(name.trim().to_string())),
        ];

        let keyframes_rule = match db
            .get(&path)
            .and_then(|tree| tree.rule.as_ref())
            .and_then(|r| r.as_keyframes())
        {
            Some(keyframes_rule) => keyframes_rule,
            None => return Err(CharismaError::RuleNotFound.into()),
        };

        Ok(format!(
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
                .collect::<Result<String, _>>()?
        ))
    } else {
        let selector_list = match parse_selector(selector) {
            Some(selector) => selector,
            None => return Err(CharismaError::ParseError.into()),
        };
        let selectors: Result<Vec<_>, _> = selector_list.into_iter().map(|s| s).collect();
        let selectors = match selectors {
            Ok(list) => list,
            Err(_) => return Err(CharismaError::ParseError.into()),
        };

        let paths: Vec<Vec<Part>> = selectors.iter().flat_map(|s| s.to_css_db_paths()).collect();
        if paths.len() != 1 {
            return Err(CharismaError::AssertionError("expected_path".into()).into());
        }

        let path = match paths.first() {
            Some(path) => path,
            None => return Err(CharismaError::AssertionError("expected_one_path".into()).into()),
        };
        let rule = match db
            .get(path)
            .and_then(|tree| tree.rule.as_ref())
            .and_then(|r| r.as_regular_rule())
        {
            Some(rule) => rule,
            None => return Err(CharismaError::AssertionError("expected_rule".into()).into()),
        };

        let mut rule_properties = rule.properties.clone();
        rule_properties.sort_by_key(|p| p.name.clone());
        let selector = match parse_selector(&rule.selector.string) {
            Some(s) => s,
            None => return Err(CharismaError::ParseError.into()),
        };

        Ok(format!(
            "
    <div data-kind=\"rule\">
        <div data-attr=\"selector\">{}</div>
        <div data-attr=\"properties\">{}</div>
    </div>
    ",
            selector.render_html(&RenderOptions::default())?,
            rule_properties
                .iter()
                .map(|p| p.render_html(&RenderOptions::default()))
                .collect::<Result<String, _>>()?,
        ))
    }
}

#[tauri::command]
fn delete(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    name: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let selector_list: Vec<_> = match parse_selector(selector) {
        Some(list) => list
            .into_iter()
            .map(|r| r.map_err(|_| CharismaError::ParseError))
            .collect::<Result<_, _>>(),
        None => return Err(CharismaError::ParseError.into()),
    }?;

    for path in selector_list.into_iter().flat_map(|s| s.to_css_db_paths()) {
        db.delete(&path, name, value);
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command]
fn disable(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    name: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let selector_list: Vec<_> = match parse_selector(selector) {
        Some(list) => list
            .into_iter()
            .map(|r| r.map_err(|_| CharismaError::ParseError))
            .collect::<Result<_, _>>(),
        None => return Err(CharismaError::ParseError.into()),
    }?;

    for selector in selector_list.iter().flat_map(|s| s.to_selectors(None)) {
        db.set_state(&selector.path, name, value, State::Commented);
    }
    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command]
fn enable(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    name: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let selector_list: Vec<_> = match parse_selector(selector) {
        Some(list) => list
            .into_iter()
            .map(|r| r.map_err(|_| CharismaError::ParseError))
            .collect::<Result<_, _>>(),
        None => return Err(CharismaError::ParseError.into()),
    }?;

    for selector in selector_list.iter().flat_map(|s| s.to_selectors(None)) {
        db.set_state(&selector.path, name, value, State::Valid);
    }
    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command]
fn insert_property(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    property: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    let property = match parse_property(property) {
        Some(p) => p,
        None => return Err(CharismaError::ParseError.into()),
    };

    let selector_list: Vec<_> = match parse_selector(selector) {
        Some(list) => list
            .into_iter()
            .map(|r| r.map_err(|_| CharismaError::ParseError))
            .collect::<Result<_, _>>(),
        None => return Err(CharismaError::ParseError.into()),
    }?;

    for selector in selector_list.iter().flat_map(|s| s.to_selectors(None)) {
        db.insert_regular_rule(&selector, &property)?;
    }
    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
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
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let selector_list: Vec<_> = match parse_selector(selector) {
        Some(list) => list
            .into_iter()
            .map(|r| r.map_err(|_| CharismaError::ParseError))
            .collect::<Result<_, _>>(),
        None => return Err(CharismaError::ParseError.into()),
    }?;
    // TODO: if we fail, we should revert all the things .. ugh
    for selector in selector_list.iter().flat_map(|s| s.to_selectors(None)) {
        match db.get_mut(&selector.path) {
            Some(rule) => rule.drain(),
            None => return Err(CharismaError::RuleNotFound.into()),
        };

        for property in properties.iter() {
            // at this point we are just parsing to validate

            let parsed_property =
                match parse_property(&format!("{}: {};", property.name, property.value)) {
                    Some(p) => p,
                    None => return Err(CharismaError::ParseError.into()),
                };
            if property.is_commented {
                db.insert_regular_rule_commented(&selector, parsed_property)?;
            } else {
                db.insert_regular_rule(&selector, &parsed_property)?;
            }
        }
    }
    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command(rename_all = "snake_case")]
fn update_value(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    selector: &str,
    name: &str,
    original_value: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let property = match parse_property(&format!("{}: {};", name, value)) {
        Some(p) => p,
        None => return Err(CharismaError::ParseError.into()),
    };

    let selector_list: Vec<_> = match parse_selector(selector) {
        Some(list) => list
            .into_iter()
            .map(|r| r.map_err(|_| CharismaError::ParseError))
            .collect::<Result<_, _>>(),
        None => return Err(CharismaError::ParseError.into()),
    }?;

    for selector in selector_list.iter().flat_map(|s| s.to_selectors(None)) {
        let rule = match db
            .get(&selector.path)
            .and_then(|t| t.rule.as_ref())
            .and_then(|r| r.as_regular_rule())
        {
            Some(rule) => rule,
            None => return Err(CharismaError::RuleNotFound.into()),
        };

        if rule
            .properties
            .iter()
            .any(|p| p.name == name.trim() && p.value == original_value)
        {
            db.delete(&selector.path, name.trim(), original_value);
            db.insert_regular_rule(&selector, &property)?;
        } else {
            return Err(CharismaError::AssertionError(
                "updating value without knowing previous value".into(),
            )
            .into());
        }
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command(rename_all = "snake_case")]
fn load_rule(
    state: tauri::State<Mutex<CSSDB>>,
    path: &str,
    rule: &str,
) -> Result<String, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let rule = match parse_one(rule) {
        Some(r) => r,
        None => return Err(CharismaError::ParseError.into()),
    };

    let selector = rule.prelude();
    let block = rule.block().map_err(|_| CharismaError::ParseError)?;
    let block = match block.as_css_declaration_or_rule_block() {
        Some(b) => b,
        None => return Err(CharismaError::ParseError.into()),
    };

    let selector_list: Vec<_> = selector
        .into_iter()
        .map(|r| r.map_err(|_| CharismaError::ParseError))
        .collect::<Result<_, _>>()?;

    for selector in (&selector_list).iter().flat_map(|s| s.to_selectors(None)) {
        for item in block.items() {
            // TODO: if this fails, we should revert everything
            // what we need to is make a clone of the original db before making changes
            // and then, revert it back
            let property = match item.as_css_declaration_with_semicolon() {
                Some(p) => p,
                None => return Err(CharismaError::ParseError.into()),
            };
            db.insert_regular_rule(&selector, &property)?;
        }
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave)?;

    Ok(selector_list
        .iter()
        .map(|s| s.to_string())
        .collect::<String>()
        .trim()
        .to_string())
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
