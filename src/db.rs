use crate::parse_utils::{parse_property, parse_selector};
use std::{collections::HashMap, fs};

use biome_css_syntax::{
    AnyCssSelector::{self, *},
    AnyCssSubSelector::*,
    CssDeclarationWithSemicolon,
};

const INHERITABLE_PROPERTIES: [&str; 29] = [
    "azimuth",
    "border-collapse",
    "border-spacing",
    "caption-side",
    "color",
    "cursor",
    "direction",
    "empty-cells",
    "font-family",
    "font-size",
    "font-style",
    "font-variant",
    "font-weight",
    "font",
    "letter-spacing",
    "line-height",
    "list-style-image",
    "list-style-position",
    "list-style-type",
    "list-style",
    "orphans",
    "quotes",
    "text-align",
    "text-indent",
    "text-transform",
    "visibility",
    "white-space",
    "widows",
    "word-spacing",
];

fn name_of_item(item: &CssDeclarationWithSemicolon) -> String {
    let decl = item.declaration();
    let property = decl.unwrap().property().unwrap();
    let property = property.as_css_generic_property().unwrap();
    let name_node = property.name().unwrap();
    let value = name_node
        .as_css_identifier()
        .unwrap()
        .value_token()
        .unwrap();
    value.token_text_trimmed().to_string()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selector: AnyCssSelector,
    pub properties: Vec<CssDeclarationWithSemicolon>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSSDB {
    children: HashMap<String, CSSDB>,
    pub rule: Option<Rule>,
}

impl CSSDB {
    pub fn new() -> CSSDB {
        CSSDB {
            children: HashMap::new(),
            rule: None,
        }
    }

    pub fn load(&mut self, css_path: &str) {
        let css = fs::read_to_string(css_path).unwrap();
        let ast = biome_css_parser::parse_css(&css, biome_css_parser::CssParserOptions::default());
        for rule in ast.tree().rules() {
            let rule = rule.as_css_qualified_rule().unwrap();
            let selector = rule.prelude();
            assert!((&selector).into_iter().collect::<Vec<_>>().len() == 1);
            let selector = selector.into_iter().next().unwrap().unwrap();
            let properties = rule.block().unwrap();
            let properties = properties
                .as_css_declaration_or_rule_block()
                .unwrap()
                .items();
            for property in properties {
                let property = property.as_css_declaration_with_semicolon().unwrap();
                let property = property.declaration().unwrap().property().unwrap();
                let property = property.as_css_generic_property().unwrap();
                let name = property.name().unwrap().to_string().trim().to_string();
                let value = property.value();
                assert!((&value).into_iter().len() == 1);
                let value = value
                    .into_iter()
                    .next()
                    .unwrap()
                    .to_string()
                    .trim()
                    .to_string();
                self.insert_mut(
                    selector.to_owned(),
                    &selector.to_css_db_path(),
                    &name,
                    &value,
                )
            }
        }
    }

    pub fn serialize(&self) -> String {
        let rule = match &self.rule {
            Some(Rule {
                properties,
                selector,
            }) => format!(
                "{} {{\n  {}\n}}\n",
                selector.to_string().trim(),
                properties
                    .iter()
                    .map(|p| p.to_string() + "\n  ")
                    .collect::<String>()
                    .trim()
            ),
            None => String::from(""),
        };

        format!(
            "{}{}",
            rule,
            self.children
                .values()
                .map(|node| node.serialize())
                .collect::<String>()
        )
    }

    fn super_pathes_of_aux(
        &self,
        path: &[String],
        is_root: bool,
        super_paths: &mut Vec<Vec<String>>,
    ) {
        if !is_root {
            if let Some(path) = self
                .get(path)
                .and_then(|n| n.rule.as_ref().map(|r| r.selector.to_css_db_path()))
            {
                super_paths.push(path)
            }
        }

        for (_, t) in &self.children {
            t.super_pathes_of_aux(path, false, super_paths);
        }
    }
    pub fn super_pathes_of(&self, path: &[String]) -> Vec<Vec<String>> {
        let mut super_paths: Vec<Vec<String>> = vec![];
        self.super_pathes_of_aux(path, true, &mut super_paths);
        super_paths
    }

    fn inheritable_properties(&self) -> HashMap<String, CssDeclarationWithSemicolon> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| INHERITABLE_PROPERTIES.contains(&name_of_item(p).as_str()))
                .map(|p| (name_of_item(p), p.clone()))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        }
    }

    fn inherited_properties_for_aux(
        &self,
        path: &[String],
        inhertied_properties: &mut HashMap<String, CssDeclarationWithSemicolon>,
    ) {
        let inherited_properties_from_self = self.inheritable_properties();
        match path {
            [] => panic!("should never reach the end of path"),
            [_part] => {
                inhertied_properties.extend(inherited_properties_from_self);
            }
            [part, parts @ ..] => {
                inhertied_properties.extend(inherited_properties_from_self);
                self.children
                    .get(part)
                    .unwrap()
                    .inherited_properties_for_aux(parts, inhertied_properties)
            }
        }
    }

    pub fn inherited_properties_for(
        &self,
        path: &[String],
    ) -> HashMap<String, CssDeclarationWithSemicolon> {
        let mut properties: HashMap<String, CssDeclarationWithSemicolon> = HashMap::new();
        self.inherited_properties_for_aux(path, &mut properties);
        for super_path in self.super_pathes_of(path) {
            properties.extend(self.get(&super_path).unwrap().inheritable_properties());
        }
        properties
    }

    fn vars(&self) -> HashMap<String, CssDeclarationWithSemicolon> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| is_var(p))
                .map(|p| (name_of_item(p), p.clone()))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        }
    }

    fn inherited_vars_for_aux(
        &self,
        path: &[String],
        inherited_vars: &mut HashMap<String, CssDeclarationWithSemicolon>,
    ) {
        let inherited_vars_from_self = self.vars();
        match path {
            [] => panic!("should never reach the end of path"),
            [_part] => {
                inherited_vars.extend(inherited_vars_from_self);
            }
            [part, parts @ ..] => {
                inherited_vars.extend(inherited_vars_from_self);
                self.children
                    .get(part)
                    .unwrap()
                    .inherited_vars_for_aux(parts, inherited_vars);
            }
        }
    }

    pub fn inherited_vars_for(
        &self,
        path: &[String],
    ) -> HashMap<String, CssDeclarationWithSemicolon> {
        let mut vars: HashMap<String, CssDeclarationWithSemicolon> = HashMap::new();
        self.inherited_vars_for_aux(path, &mut vars);
        for super_path in self.super_pathes_of(path) {
            vars.extend(self.get(&super_path).unwrap().vars());
        }
        vars
    }

    pub fn siblings_for(&self, path: &[String]) -> Vec<&CSSDB> {
        assert!(path.len() > 0);
        let (last_part, parent_path) = path.split_last().unwrap();
        let root = self.get(parent_path);
        assert!(root.is_some());
        let root = root.unwrap();
        root.children
            .iter()
            .filter(|(part, _)| *part != last_part)
            .map(|(_, tree)| tree)
            .filter(|tree| tree.rule.is_some())
            .collect()
    }

    pub fn delete_mut(&mut self, path: &[String], property_name: &String) {
        let tree = self.get_mut(path).unwrap();
        assert!(
            tree.rule.is_some(),
            "can't delete property from rule that doesn't exist"
        );
        let rule = tree.rule.as_mut().unwrap();
        rule.properties
            .retain(|p| &name_of_item(p) != property_name);
    }

    pub fn insert_mut(
        &mut self,
        selector: AnyCssSelector,
        path: &[String],
        name: &String,
        value: &String,
    ) {
        match path {
            [] => {
                match &mut self.rule {
                    Some(rule) => rule.properties.push(parse_property(name, value)),
                    None => {
                        self.rule = Some(Rule {
                            selector,
                            properties: vec![parse_property(name, value)],
                        })
                    }
                };
            }
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_mut(selector, parts, name, value),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert_mut(selector, parts, name, value);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn get(&self, path: &[String]) -> Option<&CSSDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => match self.children.get(part) {
                Some(child) => child.get(parts),
                None => None,
            },
        }
    }

    pub fn get_mut(&mut self, path: &[String]) -> Option<&mut CSSDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(child) => child.get_mut(parts),
                None => None,
            },
        }
    }
}

fn is_var(property: &CssDeclarationWithSemicolon) -> bool {
    let decl = property.as_fields().declaration.unwrap();
    let property = decl.as_fields().property.unwrap();
    let property = property.as_css_generic_property().unwrap();
    let name = property.as_fields().name.unwrap();
    let name = name.as_css_identifier().unwrap();
    name.to_string().starts_with("--")
}

#[test]
fn one_level_super_path() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_css_db_path();
    tree.insert_mut(s1, &s1_path, &"color".to_string(), &"red".to_string());
    let s2 = parse_selector(&".container .card".to_owned());
    let s2_path = s2.to_css_db_path();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());

    let paths = tree.super_pathes_of(&s1_path);
    assert_eq!(
        paths,
        vec![vec![
            ".container".to_string(),
            " ".to_string(),
            ".card".to_string()
        ]]
    );
}

#[test]
fn two_level_super_path() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_css_db_path();
    tree.insert_mut(s1, &s1_path, &"color".to_string(), &"red".to_string());
    let s2 = parse_selector(&".main .container .card".to_owned());
    let s2_path = s2.to_css_db_path();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());

    let paths = tree.super_pathes_of(&s1_path);
    assert_eq!(
        paths,
        vec![vec![
            ".main".to_string(),
            " ".to_string(),
            ".container".to_string(),
            " ".to_string(),
            ".card".to_string()
        ]]
    );
}

#[test]
fn no_super_pathes() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_css_db_path();
    tree.insert_mut(s1, &s1_path, &"color".to_string(), &"red".to_string());
    let s2 = parse_selector(&".main .container".to_owned());
    let s2_path = s2.to_css_db_path();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());

    let paths = tree.super_pathes_of(&s1_path);
    assert_eq!(paths, vec![] as Vec<Vec<String>>);
}

#[test]
fn var_is_inherited() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_css_db_path();
    tree.insert_mut(s1, &s1_path, &"--var".to_string(), &"red".to_string());
    let s2 = parse_selector(&".card .btn".to_owned());
    let s2_path = s2.to_css_db_path();
    tree.insert_mut(
        s2,
        &s2_path,
        &"font-size".to_string(),
        &"var(--var)".to_string(),
    );
    let inhertied_vars = tree.inherited_vars_for(&s2_path);
    assert_eq!(inhertied_vars.contains_key("--var"), true);
}

#[test]
fn color_is_inherited() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_css_db_path();
    tree.insert_mut(s1, &s1_path, &"color".to_string(), &"red".to_string());
    let s2 = parse_selector(&".card .btn".to_owned());
    let s2_path = s2.to_css_db_path();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());
    let inherited_properties = tree.inherited_properties_for(&s2_path);
    assert_eq!(inherited_properties.contains_key("color"), true);
}

#[test]
fn display_is_not_inherited() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_css_db_path();
    tree.insert_mut(s1, &s1_path, &"display".to_string(), &"flex".to_string());
    let s2 = parse_selector(&".card .btn".to_owned());
    let s2_path = s2.to_css_db_path();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());
    let inherited_properties = tree.inherited_properties_for(&s2_path);
    assert_eq!(inherited_properties.contains_key("display"), false);
}

#[test]
fn delete() {
    let s1 = parse_selector(&".btn".to_owned());
    let s1_path = s1.to_css_db_path();
    let s2 = parse_selector(&".card".to_owned());
    let s2_path = s2.to_css_db_path();
    let mut tree = CSSDB::new();
    tree.insert_mut(s1, &s1_path, &"font-size".to_owned(), &"20px".to_owned());
    tree.insert_mut(s2, &s2_path, &"color".to_owned(), &"red".to_owned());
    tree.delete_mut(&s1_path, &"font-size".to_owned());

    assert_eq!(
        tree.children
            .values()
            .filter_map(|p| p.rule.clone())
            .flat_map(|rule| rule
                .properties
                .iter()
                .map(|p| p.to_string().trim().to_string())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec!["color: red;".to_string()]
    );
}

#[test]
fn siblings() {
    let s1 = parse_selector(&".btn".to_owned());
    let s1_path = s1.to_css_db_path();
    let s2 = parse_selector(&".card".to_owned());
    let s2_path = s2.to_css_db_path();
    let s3 = parse_selector(&".table".to_owned());
    let s3_path = s3.to_css_db_path();
    let mut tree = CSSDB::new();
    tree.insert_mut(s1, &s1_path, &"font-size".to_owned(), &"20px".to_owned());
    tree.insert_mut(s2, &s2_path, &"color".to_owned(), &"red".to_owned());
    tree.insert_mut(s3, &s3_path, &"display".to_owned(), &"flex".to_owned());
    let s1_siblings = tree.siblings_for(&s1_path);
    let mut sibling_selectors: Vec<String> = s1_siblings
        .iter()
        .map(|r| {
            r.rule
                .clone()
                .unwrap()
                .selector
                .to_string()
                .trim()
                .to_string()
        })
        .collect();
    sibling_selectors.sort();
    assert_eq!(
        sibling_selectors,
        vec![".card".to_string(), ".table".to_string()]
    );
}

#[test]
fn insert_mutable_test() {
    let selector = parse_selector(&".btn".to_owned());
    let path = selector.to_css_db_path();
    let name = "font-size".to_owned();
    let value = "20px".to_owned();
    let mut tree = CSSDB::new();
    tree.insert_mut(selector, &path, &name, &value);
    let node = tree.get(&path).unwrap();

    assert_eq!(
        node.rule
            .as_ref()
            .unwrap()
            .properties
            .get(0)
            .unwrap()
            .to_string(),
        parse_property(&name, &value).to_string()
    )
}

#[test]
fn serialize() {
    let selector = parse_selector(&".btn".to_owned());
    let path = selector.to_css_db_path();
    let name = "font-size".to_owned();
    let value = "20px".to_owned();
    let mut tree = CSSDB::new();
    tree.insert_mut(selector, &path, &name, &value);
    assert_eq!(
        tree.serialize(),
        String::from(".btn {\n  font-size: 20px;\n}\n")
    );
}

pub trait Storage {
    fn to_css_db_path(&self) -> Vec<String>;
}

impl Storage for biome_css_syntax::AnyCssSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            CssBogusSelector(_) => todo!(),
            CssComplexSelector(s) => {
                let fields = s.as_fields();
                let left = fields.left.unwrap();
                let right = fields.right.unwrap();
                let mut parts = left.to_css_db_path();
                parts.push(" ".to_string());
                parts.extend(right.to_css_db_path());
                parts
            }
            CssCompoundSelector(selector) => selector.to_css_db_path(),
        }
    }
}

impl Storage for biome_css_syntax::CssCompoundSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        self.sub_selectors()
            .into_iter()
            .flat_map(|selector| selector.to_css_db_path())
            .collect()
    }
}

impl Storage for biome_css_syntax::AnyCssSubSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            CssAttributeSelector(_) => todo!(),
            CssBogusSubSelector(_) => todo!(),
            CssClassSelector(class) => {
                let name = class.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![format!(".{}", name)]
            }
            CssIdSelector(_) => todo!(),
            CssPseudoClassSelector(_) => todo!(),
            CssPseudoElementSelector(_) => todo!(),
        }
    }
}
