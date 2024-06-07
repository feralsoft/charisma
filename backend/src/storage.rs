use std::collections::{HashMap, HashSet};

use biome_css_syntax::{
    AnyCssDeclarationOrRule,
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
    selector: AnyCssSelector,
    properties: Vec<CssDeclarationWithSemicolon>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DBTree {
    children: HashMap<String, DBTree>,
    rule: Option<Rule>,
}

impl DBTree {
    fn new() -> DBTree {
        DBTree {
            children: HashMap::new(),
            rule: None,
        }
    }

    fn serialize(&self) -> String {
        let rule = match &self.rule {
            Some(Rule {
                properties,
                selector,
            }) => format!(
                "{} {{ {} }}",
                selector,
                properties.iter().map(|p| p.to_string()).collect::<String>()
            ),
            None => String::from(""),
        };

        format!(
            "{}\n{}",
            rule,
            self.children
                .values()
                .map(|node| node.serialize())
                .collect::<String>()
        )
    }

    fn inherited_properties_for_aux(
        &self,
        path: &[String],
        inhertied_properties: &mut HashMap<String, CssDeclarationWithSemicolon>,
    ) {
        let inherited_properties_from_self: HashMap<String, CssDeclarationWithSemicolon> =
            if let Some(rule) = &self.rule {
                rule.properties
                    .iter()
                    .filter(|p| INHERITABLE_PROPERTIES.contains(&name_of_item(p).as_str()))
                    .map(|p| (name_of_item(p), p.clone()))
                    .collect::<HashMap<_, _>>()
            } else {
                HashMap::new()
            };
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

    fn inherited_properties_for(
        &self,
        path: &[String],
    ) -> HashMap<String, CssDeclarationWithSemicolon> {
        let mut properties: HashMap<String, CssDeclarationWithSemicolon> = HashMap::new();
        self.inherited_properties_for_aux(path, &mut properties);
        properties
    }

    fn siblings_for(&self, path: &[String]) -> Vec<Rule> {
        assert!(path.len() > 0);
        let (last_part, parent_path) = path.split_last().unwrap();
        let root = self.get(parent_path);
        assert!(root.is_some());
        let root = root.unwrap();
        root.children
            .iter()
            .filter(|(part, _)| *part != last_part)
            .filter_map(|(_, tree)| tree.rule.clone())
            .collect()
    }

    fn delete_mut(&mut self, path: &[String], property_name: &String) {
        let tree = self.get_mut(path).unwrap();
        assert!(
            tree.rule.is_some(),
            "can't delete property from rule that doesn't exist"
        );
        let rule = tree.rule.as_mut().unwrap();
        rule.properties
            .retain(|p| &name_of_item(p) != property_name);
    }

    fn insert_mut(
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
                    let mut new_tree = DBTree::new();
                    new_tree.insert_mut(selector, parts, name, value);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    fn get(&self, path: &[String]) -> Option<&DBTree> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => match self.children.get(part) {
                Some(child) => child.get(parts),
                None => None,
            },
        }
    }

    fn get_mut(&mut self, path: &[String]) -> Option<&mut DBTree> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(child) => child.get_mut(parts),
                None => None,
            },
        }
    }
}

fn parse_selector(str: &String) -> AnyCssSelector {
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
fn color_is_inherited() {
    let mut tree = DBTree::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_path_parts();
    tree.insert_mut(s1, &s1_path, &"color".to_string(), &"red".to_string());
    let s2 = parse_selector(&".card .btn".to_owned());
    let s2_path = s2.to_path_parts();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());
    let inherited_properties = tree.inherited_properties_for(&s2_path);
    assert_eq!(inherited_properties.contains_key("color"), true);
}

#[test]
fn display_is_not_inherited() {
    let mut tree = DBTree::new();
    let s1 = parse_selector(&".card".to_owned());
    let s1_path = s1.to_path_parts();
    tree.insert_mut(s1, &s1_path, &"display".to_string(), &"flex".to_string());
    let s2 = parse_selector(&".card .btn".to_owned());
    let s2_path = s2.to_path_parts();
    tree.insert_mut(s2, &s2_path, &"font-size".to_string(), &"20px".to_string());
    let inherited_properties = tree.inherited_properties_for(&s2_path);
    assert_eq!(inherited_properties.contains_key("display"), false);
}

#[test]
fn delete() {
    let s1 = parse_selector(&".btn".to_owned());
    let s1_path = s1.to_path_parts();
    let s2 = parse_selector(&".card".to_owned());
    let s2_path = s2.to_path_parts();
    let mut tree = DBTree::new();
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
            .collect::<HashSet<_>>(),
        HashSet::from(["color: red;".to_string()])
    );
}

#[test]
fn siblings() {
    let s1 = parse_selector(&".btn".to_owned());
    let s1_path = s1.to_path_parts();
    let s2 = parse_selector(&".card".to_owned());
    let s2_path = s2.to_path_parts();
    let s3 = parse_selector(&".table".to_owned());
    let s3_path = s3.to_path_parts();
    let mut tree = DBTree::new();
    tree.insert_mut(s1, &s1_path, &"font-size".to_owned(), &"20px".to_owned());
    tree.insert_mut(s2, &s2_path, &"color".to_owned(), &"red".to_owned());
    tree.insert_mut(s3, &s3_path, &"display".to_owned(), &"flex".to_owned());
    let s1_siblings = tree.siblings_for(&s1_path);
    let sibling_selectors: HashSet<String> = s1_siblings
        .iter()
        .map(|r| r.selector.to_string().trim().to_string())
        .collect();
    assert_eq!(
        sibling_selectors,
        HashSet::from([".card".to_string(), ".table".to_string()])
    );
}

#[test]
fn insert_mutable_test() {
    let selector = parse_selector(&".btn".to_owned());
    let path = selector.to_path_parts();
    let name = "font-size".to_owned();
    let value = "20px".to_owned();
    let mut tree = DBTree::new();
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
    let path = selector.to_path_parts();
    let name = "font-size".to_owned();
    let value = "20px".to_owned();
    let mut tree = DBTree::new();
    tree.insert_mut(selector, &path, &name, &value);
    assert_eq!(
        tree.serialize(),
        String::from("\n.btn  { font-size: 20px;  }\n")
    );
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

pub trait Storage {
    fn to_path_parts(&self) -> Vec<String>;
}

impl Storage for biome_css_syntax::AnyCssSelector {
    fn to_path_parts(&self) -> Vec<String> {
        match self {
            CssBogusSelector(_) => todo!(),
            CssComplexSelector(s) => {
                let fields = s.as_fields();
                let left = fields.left.unwrap();
                let right = fields.right.unwrap();
                let mut parts = left.to_path_parts();
                parts.push(" ".to_string());
                parts.extend(right.to_path_parts());
                parts
            }
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
                return vec![format!(".{}", name)];
            }
            CssIdSelector(_) => todo!(),
            CssPseudoClassSelector(_) => todo!(),
            CssPseudoElementSelector(_) => todo!(),
        }
    }
}
