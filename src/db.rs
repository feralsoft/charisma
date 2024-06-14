use crate::parse_utils::{name_of_item, parse_property, parse_selector};
use std::{collections::HashMap, fs, rc::Rc};

use biome_css_syntax::{
    AnyCssPseudoClass, AnyCssPseudoElement,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::*,
    CssAttributeSelector, CssDeclarationWithSemicolon,
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

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selector: AnyCssSelector,
    pub properties: Vec<Rc<CssDeclarationWithSemicolon>>,
}

impl Rule {
    pub fn new(selector: AnyCssSelector) -> Self {
        Rule {
            selector,
            properties: vec![],
        }
    }

    pub fn insert(&mut self, property: CssDeclarationWithSemicolon) {
        self.properties.push(Rc::new(property))
    }
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
                let property = property
                    .as_css_declaration_with_semicolon()
                    .unwrap()
                    .to_owned();
                self.insert(selector.to_owned(), &selector.to_css_db_path(), property);
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
                    .map(|p| p.to_string().trim().to_string() + "\n  ")
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

    fn inheritable_properties(&self) -> HashMap<String, Rc<CssDeclarationWithSemicolon>> {
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
        inhertied_properties: &mut HashMap<String, Rc<CssDeclarationWithSemicolon>>,
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

    pub fn get_root(&self) -> Option<&Self> {
        self.get(&[":root".to_string()])
    }

    pub fn inherited_properties_for(
        &self,
        path: &[String],
    ) -> HashMap<String, Rc<CssDeclarationWithSemicolon>> {
        let mut properties: HashMap<String, Rc<CssDeclarationWithSemicolon>> = HashMap::new();
        self.inherited_properties_for_aux(path, &mut properties);
        self.get_root()
            .inspect(|tree| properties.extend(tree.inheritable_properties()));
        for super_path in self.super_pathes_of(path) {
            properties.extend(self.get(&super_path).unwrap().inheritable_properties());
        }
        properties
    }

    fn vars(&self) -> HashMap<String, Rc<CssDeclarationWithSemicolon>> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                // .inspect(|p| println!("PROP = {:?}", p))
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
        inherited_vars: &mut HashMap<String, Rc<CssDeclarationWithSemicolon>>,
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
    ) -> HashMap<String, Rc<CssDeclarationWithSemicolon>> {
        let mut vars: HashMap<String, Rc<CssDeclarationWithSemicolon>> = HashMap::new();
        self.inherited_vars_for_aux(path, &mut vars);
        self.get_root().inspect(|tree| vars.extend(tree.vars()));
        for super_path in self.super_pathes_of(path) {
            vars.extend(self.get(&super_path).unwrap().vars());
        }
        vars
    }

    pub fn siblings_with_subpath(&self, path: &[String], subpath: &[String]) -> Vec<&CSSDB> {
        assert!(path.len() > 0);
        let (last_part, parent_path) = path.split_last().unwrap();
        let root = self.get(parent_path);
        assert!(root.is_some());
        let root = root.unwrap();
        root.children
            .iter()
            .filter(|(part, _)| *part != last_part)
            .map(|(_, tree)| tree)
            .filter_map(|tree| tree.get(&subpath))
            .filter(|tree| tree.rule.is_some())
            .collect()
    }

    pub fn delete(&mut self, path: &[String], property_name: &String) {
        let tree = self.get_mut(path).unwrap();
        assert!(
            tree.rule.is_some(),
            "can't delete property from rule that doesn't exist"
        );
        let rule = tree.rule.as_mut().unwrap();
        rule.properties
            .retain(|p| &name_of_item(p) != property_name);
    }

    pub fn insert(
        &mut self,
        selector: AnyCssSelector,
        path: &[String],
        property: CssDeclarationWithSemicolon,
    ) {
        match path {
            [] => {
                match &mut self.rule {
                    Some(rule) => rule.insert(property),
                    None => {
                        let mut rule = Rule::new(selector);
                        rule.insert(property);
                        self.rule = Some(rule)
                    }
                };
            }
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert(selector, parts, property),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert(selector, parts, property);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn get(&self, path: &[String]) -> Option<&CSSDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get(part).and_then(|c| c.get(parts)),
        }
    }

    pub fn get_mut(&mut self, path: &[String]) -> Option<&mut CSSDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get_mut(part).and_then(|c| c.get_mut(parts)),
        }
    }
}

fn is_var(property: &CssDeclarationWithSemicolon) -> bool {
    let decl = property.as_fields().declaration.unwrap();
    let property = decl.as_fields().property.unwrap();
    let property = property.as_css_generic_property().unwrap();
    let name = property.as_fields().name.unwrap();
    let name = name.as_css_identifier().unwrap();
    let name = name.value_token().unwrap();
    name.text_trimmed().starts_with("--")
}

#[test]
fn one_level_super_path() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(".card");
    let s1_path = s1.to_css_db_path();
    tree.insert(s1, &s1_path, parse_property("color: red"));
    let s2 = parse_selector(".container .card");
    let s2_path = s2.to_css_db_path();
    tree.insert(s2, &s2_path, parse_property("font-size: 20px"));

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
    let s1 = parse_selector(".card");
    let s1_path = s1.to_css_db_path();
    tree.insert(s1, &s1_path, parse_property("color: red;"));
    let s2 = parse_selector(".main .container .card");
    let s2_path = s2.to_css_db_path();
    tree.insert(s2, &s2_path, parse_property("font-size: 20px"));

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
    let s1 = parse_selector(".card");
    let s1_path = s1.to_css_db_path();
    tree.insert(s1, &s1_path, parse_property("color: red"));
    let s2 = parse_selector(".main .container");
    let s2_path = s2.to_css_db_path();
    tree.insert(s2, &s2_path, parse_property("font-size: 20px"));

    let paths = tree.super_pathes_of(&s1_path);
    assert_eq!(paths, vec![] as Vec<Vec<String>>);
}

#[test]
fn var_is_inherited() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(".card");
    let s1_path = s1.to_css_db_path();
    tree.insert(s1, &s1_path, parse_property("--var: red;"));
    let s2 = parse_selector(".card .btn");
    let s2_path = s2.to_css_db_path();
    tree.insert(s2, &s2_path, parse_property("font-size: var(--var);"));
    let inhertied_vars = tree.inherited_vars_for(&s2_path);
    assert_eq!(inhertied_vars.contains_key("--var"), true);
}

#[test]
fn color_is_inherited() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(".card");
    let s1_path = s1.to_css_db_path();
    tree.insert(s1, &s1_path, parse_property("color: red"));
    let s2 = parse_selector(".card .btn");
    let s2_path = s2.to_css_db_path();
    tree.insert(s2, &s2_path, parse_property("font-size: 20px"));
    let inherited_properties = tree.inherited_properties_for(&s2_path);
    assert_eq!(inherited_properties.contains_key("color"), true);
}

#[test]
fn display_is_not_inherited() {
    let mut tree = CSSDB::new();
    let s1 = parse_selector(".card");
    let s1_path = s1.to_css_db_path();
    tree.insert(s1, &s1_path, parse_property("display: flex"));
    let s2 = parse_selector(".card .btn");
    let s2_path = s2.to_css_db_path();
    tree.insert(s2, &s2_path, parse_property("font-size: 20px"));
    let inherited_properties = tree.inherited_properties_for(&s2_path);
    assert_eq!(inherited_properties.contains_key("display"), false);
}

#[test]
fn delete() {
    let s1 = parse_selector(".btn");
    let s1_path = s1.to_css_db_path();
    let s2 = parse_selector(".card");
    let s2_path = s2.to_css_db_path();
    let mut tree = CSSDB::new();
    tree.insert(s1, &s1_path, parse_property("font-size: 20px"));
    tree.insert(s2, &s2_path, parse_property("color: red"));
    tree.delete(&s1_path, &"font-size".to_owned());

    assert_eq!(
        tree.children
            .values()
            .filter_map(|p| p.rule.as_ref())
            .flat_map(|rule| rule
                .properties
                .iter()
                .map(|p| p.to_string().trim().to_string())
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        vec!["color: red".to_string()]
    );
}

#[test]
fn insert_mutable_test() {
    let selector = parse_selector(".btn");
    let path = selector.to_css_db_path();
    let mut tree = CSSDB::new();
    tree.insert(selector, &path, parse_property("font-size: 20px;"));
    let node = tree.get(&path).unwrap();

    assert_eq!(
        node.rule
            .as_ref()
            .unwrap()
            .properties
            .get(0)
            .unwrap()
            .to_string()
            .trim(),
        "font-size: 20px;"
    )
}

#[test]
fn serialize() {
    let selector = parse_selector(".btn");
    let path = selector.to_css_db_path();
    let mut tree = CSSDB::new();
    tree.insert(selector, &path, parse_property("font-size: 20px;"));
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

impl Storage for AnyCssPseudoClass {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            AnyCssPseudoClass::CssBogusPseudoClass(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelector(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionIdentifier(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionNth(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionRelativeSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelector(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionValueList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassIdentifier(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![format!(":{}", name)]
            }
        }
    }
}

impl Storage for CssAttributeSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        let name = self.name().unwrap();
        match self.matcher() {
            Some(matcher) => {
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().unwrap();
                let value = matcher.value().unwrap();

                // [data-kind="rule"] -> ['[data-kind]', '[data-kind="rule"]']
                // so that you can explore siblings along [data-kind]
                vec![
                    format!("[{}]", name),
                    format!("[{}{}{}]", name, operator, value),
                ]
            }
            None => {
                vec![format!("[{}]", name)]
            }
        }
    }
}

impl Storage for AnyCssPseudoElement {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            AnyCssPseudoElement::CssBogusPseudoElement(_) => todo!(),
            AnyCssPseudoElement::CssPseudoElementFunctionIdentifier(_) => todo!(),
            AnyCssPseudoElement::CssPseudoElementFunctionSelector(_) => todo!(),
            AnyCssPseudoElement::CssPseudoElementIdentifier(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                vec![format!("::{}", name.text_trimmed())]
            }
        }
    }
}

impl Storage for biome_css_syntax::AnyCssSubSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            CssAttributeSelector(attribute_selector) => attribute_selector.to_css_db_path(),
            CssBogusSubSelector(s) => {
                println!("{:?}", s);
                todo!()
            }
            CssClassSelector(class) => {
                let name = class.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![format!(".{}", name)]
            }
            CssIdSelector(_) => todo!(),
            CssPseudoClassSelector(pseudo_class) => pseudo_class.class().unwrap().to_css_db_path(),
            CssPseudoElementSelector(pseudo_element) => {
                pseudo_element.element().unwrap().to_css_db_path()
            }
        }
    }
}
