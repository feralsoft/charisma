use std::{
    collections::HashMap,
    fs::{self},
    io,
};

use biome_css_syntax::{
    AnyCssDeclarationOrRule,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::*,
    CssDeclarationWithSemicolon,
};

use sha1::Digest;

fn hash(string: String) -> String {
    let mut hasher = sha1::Sha1::new();
    hasher.update(string.as_bytes());
    let h = hasher.finalize();
    format!("{:x}", h)
}

fn name_of_item(item: &AnyCssDeclarationOrRule) -> String {
    let decl = item
        .as_css_declaration_with_semicolon()
        .unwrap()
        .declaration();
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

pub fn siblings_of(selector: AnyCssSelector, idx: usize) -> Vec<String> {
    let part_paths = selector.to_path_parts();
    assert!((0..part_paths.len()).contains(&idx));
    let parent_dir_path = part_paths[0..idx].join("/");
    let mut out: Vec<String> = vec![];
    for dir in fs::read_dir(format!("db/{}", parent_dir_path)).unwrap() {
        let dir = dir.unwrap();
        assert!(dir.file_type().unwrap().is_dir());
        let file_name = dir.file_name().into_string().unwrap();
        let path = format!("{}/{}", parent_dir_path, file_name);
        if path == format!("{}/{}", parent_dir_path, part_paths[idx]) {
            continue;
        }
        out.push(path);
    }
    out
}

fn parse_selector(str: &'static str) -> AnyCssSelector {
    let rule = biome_css_parser::parse_css(
        format!("{} {{}}", str).as_str(),
        biome_css_parser::CssParserOptions::default(),
    )
    .tree()
    .rules()
    .into_iter()
    .next()
    .unwrap();

    rule.as_css_qualified_rule()
        .unwrap()
        .prelude()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
}

#[test]
fn test_siblings() {
    let s1 = parse_selector(".btn");

    delete_property(&s1, "color".to_owned());
    store_property(&s1, "color".to_owned(), "red".to_owned());

    let s2 = parse_selector(".container");

    delete_property(&s2, "display".to_owned());
    store_property(&s2, "display".to_owned(), "flex".to_owned());

    let siblings = siblings_of(s2, 0);

    assert_eq!(siblings, vec!["/".to_owned() + &hash(".btn".to_owned())],);
}

fn value_of_item(item: &AnyCssDeclarationOrRule) -> String {
    let decl = item
        .as_css_declaration_with_semicolon()
        .unwrap()
        .declaration();
    let property = decl.unwrap().property().unwrap();
    let property = property.as_css_generic_property().unwrap();
    let value_list = property.value();
    assert!((&value_list).into_iter().len() == 1);
    let value_node = value_list.into_iter().next().unwrap();
    value_node.as_any_css_value().unwrap().to_string()
}

fn parse_one(rule: String) -> biome_css_syntax::CssQualifiedRule {
    let rules = biome_css_parser::parse_css(&rule, biome_css_parser::CssParserOptions::default())
        .tree()
        .rules();
    assert!((&rules).into_iter().len() == 1);
    let rule = rules.into_iter().next().unwrap();

    rule.as_css_qualified_rule().unwrap().to_owned()
}

fn parse_property(name: &String, value: &String) -> CssDeclarationWithSemicolon {
    let property_str = format!("{}: {};", name, value);
    let dummy_rule = parse_one(format!(".a {{ {} }}", property_str));
    let block = dummy_rule.block().unwrap();
    let block = block.as_css_declaration_or_rule_block();
    assert!(block.unwrap().items().into_iter().len() == 1);
    let item = block.unwrap().items().into_iter().next().unwrap();
    item.as_css_declaration_with_semicolon().unwrap().to_owned()
}

pub fn delete_property(selector: &biome_css_syntax::AnyCssSelector, name: String) {
    let db_path = selector.to_path_parts().join("/");
    let path = format!("db/{}/index.css", db_path);
    match fs::read_to_string(&path) {
        Ok(rule) => {
            let rule = parse_one(rule);
            let block = rule.block().unwrap();
            let block = block.as_css_declaration_or_rule_block().unwrap();
            let mut sorted_properties = block
                .items()
                .into_iter()
                .filter(|i| name_of_item(i) != name)
                .map(|i| i.to_string().trim().to_string())
                .collect::<Vec<String>>();
            sorted_properties.sort();
            let new_rule = format!(
                "{} {{\n  {}\n}}",
                selector.to_string().trim(),
                sorted_properties.join("\n  ")
            );
            fs::write(path, new_rule).unwrap();
        }
        Err(_) => panic!("should never delete a property of a non-existent rule"),
    }
}

pub fn store_property(selector: &biome_css_syntax::AnyCssSelector, name: String, value: String) {
    let db_path = selector.to_path_parts().join("/");
    let path = format!("db/{}/index.css", db_path);
    match fs::read_to_string(&path) {
        Ok(rule) => {
            let rule = parse_one(rule);
            parse_property(&name, &value); // make sure we can parse it
            let block = rule.block().unwrap();
            let block = block.as_css_declaration_or_rule_block().unwrap();
            let selector = rule.prelude().into_iter().next().unwrap().unwrap();
            assert!(selector.to_path_parts().join("/") == db_path);

            assert!(!block.items().into_iter().any(|i| name_of_item(&i) == name));
            let mut properties: HashMap<String, String> = block
                .items()
                .into_iter()
                .map(|i| (name_of_item(&i), value_of_item(&i)))
                .collect();
            properties.insert(name, value);
            let mut sorted_properties: Vec<_> = properties
                .iter()
                .map(|(name, value)| format!("{}: {};", name, value))
                .collect();
            sorted_properties.sort();

            let new_rule = format!(
                "{} {{\n  {}\n}}\n",
                selector.to_string().trim(),
                sorted_properties.join("\n  ")
            );
            fs::write(path, new_rule).unwrap();
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let new_property = parse_property(&name, &value);
            let rule = format!("{} {{\n  {}\n}}\n", selector, new_property);
            fs::create_dir(format!("db/{}", db_path)).unwrap();
            fs::write(path, rule).unwrap();
        }
        Err(_) => panic!("unknown error"),
    }
}

pub trait Storage {
    fn to_path_parts(&self) -> Vec<String>;
}

impl Storage for biome_css_syntax::AnyCssSelector {
    fn to_path_parts(&self) -> Vec<String> {
        match self {
            CssBogusSelector(_) => todo!(),
            CssComplexSelector(_) => todo!(),
            CssCompoundSelector(selector) => selector.to_path_parts(),
        }
    }
}

impl Storage for biome_css_syntax::CssCompoundSelector {
    fn to_path_parts(&self) -> Vec<String> {
        self.sub_selectors()
            .into_iter()
            .flat_map(|selector| selector.to_path_parts())
            .collect()
    }
}

impl Storage for biome_css_syntax::AnyCssSubSelector {
    fn to_path_parts(&self) -> Vec<String> {
        match self {
            CssAttributeSelector(_) => todo!(),
            CssBogusSubSelector(_) => todo!(),
            CssClassSelector(class) => {
                let name = class.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                return vec![hash(format!(".{}", name))];
            }
            CssIdSelector(_) => todo!(),
            CssPseudoClassSelector(_) => todo!(),
            CssPseudoElementSelector(_) => todo!(),
        }
    }
}
