// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use css_tree::*;
use html::*;
use parse_utils::{parse_one, parse_property, parse_selector};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tauri::InvokeError;

mod css_tree;
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

#[derive(Serialize, Debug, Clone)]
pub enum CharismaError {
    DbLocked,
    ParseError(String),
    FailedToSave,
    RuleNotFound,
    AssertionError(String),
}

#[tauri::command]
fn search(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    q: &str,
) -> Result<RenderResult, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    let parts: Vec<&str> = q.trim().split(' ').map(|s| s.trim()).collect();

    let mut results: Vec<String> = db
        .all_selectors_with_properties()
        .iter()
        .filter(|selector| parts.iter().all(|q| selector.contains(q)))
        .cloned()
        .collect();

    results.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));

    let results = results
        .iter()
        .map(|s| {
            if s.starts_with('@') {
                match s.split("@keyframes").nth(1) {
                    Some(name) => RenderResult {
                        html: render_keyframes_selector(name.trim()),
                        errors: vec![],
                    },
                    None => RenderResult {
                        html: String::from(""),
                        errors: vec![CharismaError::ParseError(
                            "failed to render keyframes".to_string(),
                        )],
                    },
                }
            } else {
                match parse_selector(s) {
                    Ok(selector) => match selector.into_iter().next() {
                        Some(Ok(s)) => s.render_html(&RenderOptions::default()),
                        Some(Err(e)) => RenderResult {
                            html: "".to_string(),
                            errors: vec![CharismaError::ParseError(e.to_string())],
                        },
                        None => RenderResult {
                            html: "".to_string(),
                            errors: vec![CharismaError::ParseError("no selector found".into())],
                        },
                    },
                    Err(_) => RenderResult {
                        html: "".to_string(),
                        errors: vec![CharismaError::ParseError(s.to_string())],
                    },
                }
            }
        })
        .take(20)
        .reduce(|acc, RenderResult { html, errors }| RenderResult {
            errors: [acc.errors, errors].concat(),
            html: acc.html + &html,
        })
        .unwrap_or(RenderResult {
            html: String::from(""),
            errors: vec![],
        });

    Ok(results)
}

#[tauri::command]
fn find_property(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    q: &str,
) -> Result<Vec<(RenderResult, RenderResult)>, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    let parts: Vec<&str> = q.trim().split(' ').map(|s| s.trim()).collect();

    let mut results: Vec<(Arc<Property>, Selector)> = db.recursive_search_for_property(&parts);

    results.sort_by(|a, b| {
        let a_property = format!("{}: {};", a.0.name, a.0.value);
        let b_property = format!("{}: {};", b.0.name, b.0.value);
        a_property
            .len()
            .cmp(&b_property.len())
            .then_with(|| a_property.cmp(&b_property))
    });

    let results: Result<Vec<(RenderResult, RenderResult)>, _> = results
        .iter()
        .map(|(p, s)| (p, parse_selector(&s.string)))
        .map(
            |(property, selector)| -> Result<(RenderResult, RenderResult), CharismaError> {
                selector.map(|selector| {
                    (
                        property.render_html(&RenderOptions::default()),
                        selector.render_html(&RenderOptions::default()),
                    )
                })
            },
        )
        .take(100)
        .collect();

    Ok(results?)
}

// TODO: remove the need for this
#[tauri::command]
fn insert_empty_rule(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    if selector.starts_with("@keyframes") {
        match selector.split("@keyframes").nth(1) {
            Some(name) => db.insert_empty_keyframes_rule(name.trim().to_string()),
            None => {
                return Err(CharismaError::ParseError("keyframes parse error".to_string()).into())
            }
        }
    } else {
        let selector = parse_selector(selector)?.to_selector(None)?;
        db.insert_empty_regular_rule(&selector);
    }
    Ok(fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave)?)
}

#[tauri::command]
fn render_rule(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
) -> Result<String, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    if selector.starts_with("@keyframes") {
        let name = match selector.split("@keyframes").nth(1) {
            Some(name) => name.trim(),
            None => return Err(CharismaError::ParseError("keyframes typo".into()).into()),
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
                .reduce(|acc, RenderResult { html, errors }| RenderResult {
                    errors: [acc.errors, errors].concat(),
                    html: acc.html + &html,
                })
                .unwrap_or(RenderResult {
                    html: String::from(""),
                    errors: vec![],
                })
                .html
        ))
    } else {
        let selector = parse_selector(selector)?;
        let path = selector.to_css_db_path()?;

        let rule = match db
            .get(&path)
            .and_then(|tree| tree.rule.as_ref())
            .and_then(|r| r.as_regular_rule())
        {
            Some(rule) => rule,
            None => return Err(CharismaError::AssertionError("expected_rule".into()).into()),
        };
        let mut properties = rule.properties;
        properties.sort_by_key(|p| p.name.clone());

        Ok(format!(
            "
    <div data-kind=\"rule\">
        <div data-attr=\"selector\">{}</div>
        <div data-attr=\"properties\">{}</div>
    </div>
    ",
            selector.render_html(&RenderOptions::default()).html,
            properties
                .iter()
                .map(|p| p.render_html(&RenderOptions::default()))
                .reduce(|acc, RenderResult { html, errors }| RenderResult {
                    errors: [acc.errors, errors].concat(),
                    html: acc.html + &html,
                })
                .unwrap_or(RenderResult {
                    html: String::from(""),
                    errors: vec![],
                })
                .html
        ))
    }
}

#[tauri::command]
fn delete(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
    name: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let db_path = parse_selector(selector)?.to_css_db_path()?;
    db.delete(&db_path, name, value);

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command]
fn disable(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
    name: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let db_path = parse_selector(selector)?.to_css_db_path()?;
    db.set_state(&db_path, name, value, State::Commented);

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command]
fn enable(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
    name: &str,
    value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let db_path = parse_selector(selector)?.to_css_db_path()?;
    db.set_state(&db_path, name, value, State::Valid);

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command]
fn all_properties() -> &'static str {
    include_str!("../all_properties.json")
}

#[tauri::command]
fn insert_property(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
    property: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }
    let property = parse_property(property)?;
    let selector = parse_selector(selector)?.to_selector(None)?;
    db.insert_regular_rule(&selector, &property)?;

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
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    selector: &str,
    properties: Vec<JsonProperty>,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let selector = parse_selector(selector)?.to_selector(None)?;

    match db.get_mut(&selector.path) {
        Some(rule) => rule.drain(),
        None => return Err(CharismaError::RuleNotFound.into()),
    };

    for property in properties.iter() {
        // at this point we are just parsing to validate
        let parsed_property = parse_property(&format!("{}: {};", property.name, property.value))?;
        if property.is_commented {
            db.insert_regular_rule_commented(&selector, parsed_property)?;
        } else {
            db.insert_regular_rule(&selector, &parsed_property)?;
        }
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command(rename_all = "snake_case")]
fn update_value(
    state: tauri::State<Mutex<CssTree>>,
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

    let property = parse_property(&format!("{}: {};", name, value))?;

    let selector = parse_selector(selector)?.to_selector(None)?;

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

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command(rename_all = "snake_case")]
fn load_rule(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    rule: &str,
) -> Result<String, InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let rule = parse_one(rule)?;
    let selector = rule.prelude().to_selector(None)?;
    let block = rule
        .block()
        .map_err(|e| CharismaError::ParseError(e.to_string()))?;
    let block = match block.as_css_declaration_or_rule_block() {
        Some(b) => b,
        None => return Err(CharismaError::ParseError("invalid rule".to_string()).into()),
    };

    for item in block.items() {
        // TODO: if this fails, we should revert everything
        // what we need to is make a clone of the original db before making changes
        // and then, revert it back
        let property = match item.as_css_declaration_with_semicolon() {
            Some(p) => p,
            None => return Err(CharismaError::ParseError("invalid decl".to_string()).into()),
        };
        db.insert_regular_rule(&selector, property)?;
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave)?;

    Ok(selector.string)
}

#[tauri::command(rename_all = "snake_case")]
fn rename_rule(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    old_selector: &str,
    new_selector: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let old_selector_path = parse_selector(old_selector)?.to_css_db_path()?;

    let old_tree = match db.get_mut(&old_selector_path) {
        Some(t) => t,
        None => return Err(CharismaError::RuleNotFound.into()),
    };

    let rule = match old_tree.rule.as_ref().and_then(|r| r.as_regular_rule()) {
        Some(rule) => rule,
        None => return Err(CharismaError::RuleNotFound.into()),
    };

    let old_properties = rule.properties.clone();
    old_tree.drain();

    let new_selector = parse_selector(new_selector)?.to_selector(None)?;

    for property_to_be_moved in old_properties {
        db.insert_regular_property(&new_selector, property_to_be_moved.as_ref())
            .unwrap();
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

#[tauri::command(rename_all = "snake_case")]
fn rename_property(
    state: tauri::State<Mutex<CssTree>>,
    path: &str,
    is_commented: bool,
    selector: &str,
    old_property_name: &str,
    new_property_name: &str,
    property_value: &str,
) -> Result<(), InvokeError> {
    let mut db = state.lock().map_err(|_| CharismaError::DbLocked)?;
    if !db.is_loaded(path) {
        db.load(path)?;
    }

    let selector_path = parse_selector(selector)?.to_css_db_path()?;

    let state = if is_commented {
        State::Commented
    } else {
        State::Valid
    };

    match db
        .get_mut(&selector_path)
        .and_then(|t| t.rule.as_mut())
        .and_then(|r| r.as_mut_regular_rule())
    {
        Some(rule) => {
            rule.remove(&Property {
                state: state.clone(),
                name: old_property_name.into(),
                value: property_value.into(),
            });
            rule.insert(Property {
                state: state.clone(),
                name: new_property_name.into(),
                value: property_value.into(),
            })
        }
        None => todo!(),
    }

    fs::write(path, db.serialize()).map_err(|_| CharismaError::FailedToSave.into())
}

fn main() {
    let db = Mutex::new(CssTree::new());

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            render_rule,
            search,
            find_property,
            delete,
            all_properties,
            enable,
            disable,
            insert_property,
            replace_all_properties,
            update_value,
            insert_empty_rule,
            load_rule,
            rename_rule,
            rename_property,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
