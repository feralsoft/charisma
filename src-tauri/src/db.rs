use crate::parse_utils::parse_property;
use std::{collections::HashMap, fs, sync::Arc};

use biome_css_syntax::{
    AnyCssAtRule, AnyCssKeyframesSelector, AnyCssPseudoClass, AnyCssPseudoClassNth,
    AnyCssPseudoClassNthSelector, AnyCssPseudoElement, AnyCssRelativeSelector, AnyCssRule,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::{self, *},
    CssAttributeSelector, CssDeclarationOrRuleBlock, CssDeclarationWithSemicolon,
    CssKeyframesAtRule, CssKeyframesPercentageSelector, CssKeyframesSelectorList,
    CssPseudoClassFunctionCompoundSelector, CssPseudoClassFunctionCompoundSelectorList,
    CssPseudoClassFunctionIdentifier, CssPseudoClassFunctionNth,
    CssPseudoClassFunctionRelativeSelectorList, CssPseudoClassFunctionSelector,
    CssPseudoClassFunctionSelectorList, CssPseudoClassFunctionValueList, CssPseudoClassNth,
    CssPseudoClassNthIdentifier, CssPseudoClassNthNumber, CssPseudoClassNthSelector,
    CssPseudoElementFunctionIdentifier, CssPseudoElementFunctionSelector, CssRelativeSelector,
    CssSyntaxKind, CssUniversalSelector,
};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Valid,
    Commented,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub state: State,
    pub name: String,
    pub value: String,
}

impl Property {
    fn to_commented(&self) -> Self {
        Property {
            state: State::Commented,
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    pub string: String,
    pub path: Vec<Part>,
}

pub trait ToSelectors {
    fn to_selectors(&self, parent: Option<&Selector>) -> Vec<Selector>;
}

fn path_to_string(path: &Vec<Part>) -> String {
    let condensed_path = path
        .iter()
        .fold::<Vec<Part>, _>(vec![], |new_path, part| match part {
            Part::Pattern(Pattern::AttributeMatch(name, _, _)) => {
                if let Some(Part::Pattern(Pattern::Attribute(new_name))) = new_path.last() {
                    assert!(new_name == name);
                } else {
                    panic!();
                };
                let mut new_path: Vec<Part> =
                    new_path.iter().take(new_path.len() - 1).cloned().collect();
                new_path.push(part.clone());
                new_path
            }
            _ => [new_path, vec![part.clone()]].concat(),
        });

    return condensed_path
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join("");
}

impl ToSelectors for AnyCssRelativeSelector {
    fn to_selectors(&self, parent: Option<&Selector>) -> Vec<Selector> {
        let selector = self.as_css_relative_selector().unwrap();
        // this fucking sucks.. I would assume `.combinator` would do this, but it doesn't

        let combinator = if selector.to_string().trim().starts_with("&") {
            Combinator::And
        } else {
            selector
                .combinator()
                .map(|c| get_combinator_type(c.kind()))
                .unwrap_or(Combinator::Descendant)
        };
        let selector = selector.selector().unwrap();

        selector
            .to_css_db_paths()
            .iter()
            .map(|path| {
                let path = [
                    parent.map(|p| p.path.clone()).unwrap_or(vec![]),
                    vec![Part::Combinator(combinator.clone())],
                    path.clone(),
                ]
                .concat();

                Selector {
                    string: path_to_string(&path),
                    path,
                }
            })
            .collect()
    }
}

impl ToSelectors for AnyCssSelector {
    fn to_selectors(&self, parent: Option<&Selector>) -> Vec<Selector> {
        self.to_css_db_paths()
            .iter()
            .map(|path| {
                let path = [
                    parent.map(|p| p.path.clone()).unwrap_or(vec![]),
                    path.clone(),
                ]
                .concat();
                Selector {
                    string: path_to_string(&path),
                    path,
                }
            })
            .collect()
    }
}

impl Property {
    pub fn to_string(&self) -> String {
        let property_str = format!("{}: {};", self.name, self.value);
        match self.state {
            State::Valid => property_str,
            State::Commented => format!("/* {} */", property_str),
        }
    }

    pub fn name(node: &CssDeclarationWithSemicolon) -> String {
        let decl = node.declaration().unwrap();
        let property = decl.property().unwrap();
        let property = property.as_css_generic_property().unwrap();
        let name = property.name().unwrap();
        let name = name.as_css_identifier().unwrap();
        let name = name.value_token().unwrap();
        name.text_trimmed().to_string()
    }

    pub fn value(node: &CssDeclarationWithSemicolon) -> String {
        let decl = node.declaration().unwrap();
        let property = decl.property().unwrap();
        let property = property.as_css_generic_property().unwrap();
        property
            .value()
            .into_iter()
            .map(|item| item.to_string())
            .collect::<String>()
            .trim()
            .to_string()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegularRule {
    pub selector: Selector,
    pub properties: Vec<Arc<Property>>,
}

impl RegularRule {
    pub fn new(selector: Selector) -> Self {
        RegularRule {
            selector,
            properties: vec![],
        }
    }

    pub fn comment_all_with_name(&mut self, name: &str) {
        self.properties = self
            .properties
            .iter()
            .map(|p| {
                if &p.name == name {
                    Arc::new(p.to_commented())
                } else {
                    p.clone()
                }
            })
            .collect();
    }

    pub fn insert(&mut self, new_property: Property) {
        if new_property.state == State::Valid {
            self.properties = self
                .properties
                .iter()
                // if we are literally re-entering the same property, just ignore it
                // ^ this is important if we are loading in a huge css file
                .filter(|existing_property| {
                    !(existing_property.name == new_property.name
                        && existing_property.value == new_property.value)
                })
                // if its the same name, but different value, comment out the other ones
                .map(|p| {
                    if p.name == new_property.name {
                        Arc::new(p.to_commented())
                    } else {
                        p.clone()
                    }
                })
                .collect();
        }

        self.properties.push(Arc::new(new_property))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    path: Vec<Part>,
    properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframes {
    name: String,
    frames: Vec<Frame>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    RegularRule(RegularRule),
    AtRule(Keyframes),
}

impl Rule {
    pub fn as_regular_rule(&self) -> Option<RegularRule> {
        match self {
            Rule::RegularRule(rule) => Some(rule.clone()),
            Rule::AtRule(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSSDB {
    children: HashMap<Part, CSSDB>,
    current_path: Option<String>,
    pub rule: Option<Rule>,
}

fn get_comments(str: &str) -> Vec<String> {
    let mut idx = 0;
    let mut comments: Vec<String> = vec![];
    while str[idx..].contains("/*") {
        match (str[idx..].find("/*"), str[idx..].find("*/")) {
            (Some(start), Some(end)) => {
                comments.push(str[(idx + start + 2)..(idx + end)].to_string());
                idx += end + 2;
            }
            (None, None) => {}
            _ => panic!("unexpected pattern"),
        }
    }
    comments
}

impl CSSDB {
    pub fn new() -> CSSDB {
        CSSDB {
            children: HashMap::new(),
            rule: None,
            current_path: None,
        }
    }

    pub fn is_loaded(&self, path: &str) -> bool {
        return self
            .current_path
            .as_ref()
            .map(|p| p.as_str() == path)
            .is_some();
    }

    fn load_rule(&mut self, selector: Selector, block: &CssDeclarationOrRuleBlock) {
        let mut comments: Vec<String> = vec![];
        comments.extend(get_comments(
            block.l_curly_token().unwrap().token_text().text(),
        ));
        comments.extend(get_comments(
            block.r_curly_token().unwrap().token_text().text(),
        ));

        for property in block.items() {
            match property {
                biome_css_syntax::AnyCssDeclarationOrRule::AnyCssRule(rule) => {
                    let rule = rule.as_css_nested_qualified_rule().unwrap();
                    let block = rule.block().unwrap();
                    let block = block.as_css_declaration_or_rule_block().unwrap();
                    for child in rule.prelude() {
                        let child = child.unwrap();
                        for selector in child.to_selectors(Some(&selector)) {
                            self.load_rule(selector, block);
                        }
                    }
                }
                biome_css_syntax::AnyCssDeclarationOrRule::CssBogus(_) => panic!(),
                biome_css_syntax::AnyCssDeclarationOrRule::CssDeclarationWithSemicolon(
                    property,
                ) => {
                    comments.extend(get_comments(&property.to_string()));
                    self.insert_regular_rule(&selector, &property);
                }
            }
        }

        for property in comments.iter().filter_map(|str| parse_property(&str)) {
            self.insert_regular_rule_commented(&selector, property);
        }
    }

    fn load_at_rule(&mut self, at_rule: AnyCssAtRule) {
        let at_rule_paths = at_rule.to_css_db_paths();
        assert!(at_rule_paths.len() == 1);
        let at_rule_path = at_rule_paths.first().unwrap();
        match at_rule {
            AnyCssAtRule::CssKeyframesAtRule(rule) => {
                let name = rule.name().unwrap();
                let block = rule.block().unwrap();
                let block = block.as_css_keyframes_block().unwrap();
                let mut frames: Vec<Frame> = vec![];
                for item in block.items() {
                    let frame = match item {
                        biome_css_syntax::AnyCssKeyframesItem::CssBogusKeyframesItem(_) => todo!(),
                        biome_css_syntax::AnyCssKeyframesItem::CssKeyframesItem(item) => {
                            let paths = item.selectors().to_css_db_paths();
                            assert!(paths.len() == 1);
                            let path = paths.first().unwrap().to_owned();

                            let block = item.block().unwrap();
                            let block = block.as_css_declaration_list_block().unwrap();
                            let mut properties: Vec<Property> = vec![];
                            for property in block.declarations() {
                                let property = property.declaration().unwrap().property().unwrap();
                                let property = property.as_css_generic_property().unwrap();
                                let name = property.name().unwrap().to_string().trim().to_string();
                                let value = property
                                    .value()
                                    .into_iter()
                                    .map(|value| value.to_string())
                                    .reduce(|acc, cur| format!("{} {}", acc.trim(), cur.trim()))
                                    .unwrap();
                                properties.push(Property {
                                    state: State::Valid,
                                    name,
                                    value,
                                })
                            }
                            Frame { path, properties }
                        }
                    };
                    frames.push(frame)
                }

                self.insert_raw(
                    at_rule_path,
                    Rule::AtRule(Keyframes {
                        name: name.to_string().trim().to_string(),
                        frames,
                    }),
                )
            }
            AnyCssAtRule::CssBogusAtRule(_) => todo!(),
            AnyCssAtRule::CssCharsetAtRule(_) => todo!(),
            AnyCssAtRule::CssColorProfileAtRule(_) => todo!(),
            AnyCssAtRule::CssContainerAtRule(_) => todo!(),
            AnyCssAtRule::CssCounterStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssDocumentAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFaceAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFeatureValuesAtRule(_) => todo!(),
            AnyCssAtRule::CssFontPaletteValuesAtRule(_) => todo!(),
            AnyCssAtRule::CssImportAtRule(_) => panic!(),
            AnyCssAtRule::CssLayerAtRule(_) => todo!(),
            AnyCssAtRule::CssMediaAtRule(_) => todo!(),
            AnyCssAtRule::CssNamespaceAtRule(_) => todo!(),
            AnyCssAtRule::CssPageAtRule(_) => todo!(),
            AnyCssAtRule::CssPropertyAtRule(_) => todo!(),
            AnyCssAtRule::CssScopeAtRule(_) => todo!(),
            AnyCssAtRule::CssStartingStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssSupportsAtRule(_) => todo!(),
        }
    }

    pub fn load(&mut self, css_path: &str) {
        let css = fs::read_to_string(css_path).unwrap();
        let ast = biome_css_parser::parse_css(&css, biome_css_parser::CssParserOptions::default());
        for rule in ast.tree().rules() {
            match rule {
                AnyCssRule::CssQualifiedRule(rule) => {
                    for selector in rule.prelude() {
                        let block = rule.block().unwrap();
                        let block = block.as_css_declaration_or_rule_block().unwrap();
                        for selector in selector.unwrap().to_selectors(None) {
                            self.insert_empty_regular_rule(&selector);
                            self.load_rule(selector, block);
                        }
                    }
                }
                AnyCssRule::CssAtRule(at_rule) => self.load_at_rule(at_rule.rule().unwrap()),
                AnyCssRule::CssBogusRule(_) => todo!(),
                AnyCssRule::CssNestedQualifiedRule(_) => todo!(),
            };
        }
        self.current_path = Some(css_path.to_string());
    }

    pub fn serialize(&self) -> String {
        let rule = match &self.rule {
            Some(Rule::RegularRule(RegularRule {
                properties,
                selector,
            })) => {
                format!(
                    "{} {{\n    {}\n}}\n",
                    selector.string,
                    properties
                        .iter()
                        .map(|p| p.to_string() + "\n    ")
                        .collect::<String>()
                        .trim()
                )
            }
            Some(Rule::AtRule(Keyframes { name, frames })) => {
                format!(
                    "@keyframes {} {{\n    {}\n}}\n",
                    name,
                    frames
                        .iter()
                        .map(|p| format!(
                            "{} {{\n        {}\n    }}\n    ",
                            // ugh this is so bad
                            p.path.last().unwrap().to_string(),
                            p.properties
                                .iter()
                                .map(|p| p.to_string() + "\n        ")
                                .collect::<String>()
                                .trim()
                        ))
                        .collect::<String>()
                        .trim()
                )
            }
            None => String::from(""),
        };

        let mut children: Vec<(&Part, &CSSDB)> = self.children.iter().collect();
        children.sort_by_key(|(p, _)| p.to_string());

        format!(
            "{}{}",
            rule,
            children
                .iter()
                .map(|(_, t)| t.serialize())
                .collect::<String>()
        )
    }

    fn all_selectors_with_properties_aux(&self, selectors: &mut Vec<String>) {
        if let Some(Rule::RegularRule(rule)) = self.rule.as_ref() {
            if !rule.properties.is_empty() {
                selectors.push(rule.selector.string.to_owned())
            }
        }
        for (_, tree) in &self.children {
            tree.all_selectors_with_properties_aux(selectors);
        }
    }

    pub fn all_selectors_with_properties(&self) -> Vec<String> {
        let mut selectors: Vec<String> = vec![];
        self.all_selectors_with_properties_aux(&mut selectors);
        selectors
    }

    pub fn drain(&mut self) {
        match &mut self.rule {
            Some(Rule::RegularRule(rule)) => rule.properties.drain(0..),
            Some(Rule::AtRule(_)) => panic!("sdfwfjl"),
            None => todo!(),
        };
    }

    pub fn set_state(
        &mut self,
        path: &[Part],
        property_name: &str,
        property_value: &str,
        state: State,
    ) {
        let tree = self.get_mut(path).unwrap();
        assert!(
            tree.rule.is_some(),
            "can't delete property from rule that doesn't exist"
        );

        let rule = tree.rule.as_mut().unwrap();
        match rule {
            Rule::RegularRule(rule) => {
                rule.comment_all_with_name(property_name);
                if state == State::Valid {
                    rule.insert(Property {
                        name: property_name.to_string(),
                        value: property_value.to_string(),
                        state,
                    });
                }
            }
            Rule::AtRule(_) => panic!(),
        }
    }

    pub fn delete(&mut self, path: &[Part], property_name: &str, property_value: &str) {
        let tree = self.get_mut(path).unwrap();
        assert!(
            tree.rule.is_some(),
            "can't delete property from rule that doesn't exist"
        );
        let rule = tree.rule.as_mut().unwrap();
        match rule {
            Rule::RegularRule(rule) => {
                rule.properties
                    .retain(|p| !(&p.name == property_name && &p.value == property_value));
            }
            Rule::AtRule(_) => panic!(),
        }
    }

    fn insert_raw(&mut self, path: &[Part], rule: Rule) {
        match path {
            [] => match &mut self.rule {
                Some(_) => panic!(),
                None => self.rule = Some(rule),
            },
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_raw(parts, rule),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert_raw(parts, rule);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    fn insert_raw_regular_rule(&mut self, selector: Selector, path: &[Part], property: Property) {
        match path {
            [] => {
                match &mut self.rule {
                    Some(Rule::RegularRule(rule)) => rule.insert(property),
                    Some(Rule::AtRule(_)) => panic!(),
                    None => {
                        let mut rule = RegularRule::new(selector);
                        rule.insert(property);
                        self.rule = Some(Rule::RegularRule(rule))
                    }
                };
            }
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_raw_regular_rule(selector, parts, property),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert_raw_regular_rule(selector, parts, property);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn insert_regular_rule_commented(
        &mut self,
        selector: &Selector,
        property: CssDeclarationWithSemicolon,
    ) {
        self.insert_raw_regular_rule(
            selector.clone(),
            &selector.path,
            Property {
                name: Property::name(&property),
                value: Property::value(&property),
                state: State::Commented,
            },
        )
    }

    fn insert_empty_regular_rule_aux(&mut self, selector: Selector, path: &[Part]) {
        match path {
            [] => {
                match &mut self.rule {
                    Some(_) => {} // already exists
                    None => self.rule = Some(Rule::RegularRule(RegularRule::new(selector))),
                };
            }
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_empty_regular_rule_aux(selector, parts),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert_empty_regular_rule_aux(selector, parts);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn insert_empty_regular_rule(&mut self, selector: &Selector) {
        self.insert_empty_regular_rule_aux(selector.clone(), &selector.path);
    }

    pub fn insert_regular_rule(
        &mut self,
        selector: &Selector,
        property: &CssDeclarationWithSemicolon,
    ) {
        self.insert_raw_regular_rule(
            selector.clone(),
            &selector.path,
            Property {
                name: Property::name(property),
                value: Property::value(property),
                state: State::Valid,
            },
        )
    }

    pub fn get(&self, path: &[Part]) -> Option<&CSSDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get(part).and_then(|c| c.get(parts)),
        }
    }

    pub fn get_mut(&mut self, path: &[Part]) -> Option<&mut CSSDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get_mut(part).and_then(|c| c.get_mut(parts)),
        }
    }
}

pub trait DBPath {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>>;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Combinator {
    // " "
    Descendant,
    // ">"
    DirectDescendant,
    // "&"
    And,
    // +
    Plus,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Pattern {
    // [data-kind]
    Attribute(String),
    // [data-kind=rule]
    AttributeMatch(String, String, String),
    // .name
    Class(String),
    // #name
    Id(String),
    // div
    Element(String),
    // ::before
    PseudoElement(String),
    // :active
    PseudoClass(String),
    // :has(
    PseudoClassWithSelectorList(String),
    // )
    CloseSelectorList,
    // *
    Star,
    // 2 -- part of nth selectors
    Number(i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AtRulePart {
    // @keyframes
    Keyframes,
    // keyframe-name
    Name(String),
    // 20%
    Percentage(i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Combinator(Combinator),
    Pattern(Pattern),
    AtRule(AtRulePart),
}

impl ToString for Part {
    fn to_string(&self) -> String {
        match self {
            Part::Combinator(c) => c.to_string(),
            Part::Pattern(p) => p.to_string(),
            Part::AtRule(a) => a.to_string(),
        }
    }
}

impl ToString for Combinator {
    fn to_string(&self) -> String {
        match self {
            Combinator::Descendant => String::from(" "),
            Combinator::DirectDescendant => String::from(" > "),
            Combinator::And => String::from(""),
            Combinator::Plus => String::from(" + "),
        }
    }
}

impl ToString for Pattern {
    fn to_string(&self) -> String {
        match self {
            Pattern::Attribute(name) => format!("[{}]", name),
            Pattern::AttributeMatch(name, matcher, value) => {
                format!("[{}{}{}]", name, matcher, value)
            }
            Pattern::Class(name) => format!(".{}", name),
            Pattern::Id(name) => format!("#{}", name),
            Pattern::Element(name) => String::from(name),
            Pattern::PseudoElement(name) => format!("::{}", name),
            Pattern::PseudoClass(name) => format!(":{}", name),
            Pattern::PseudoClassWithSelectorList(name) => format!(":{}(", name),
            Pattern::CloseSelectorList => String::from(")"),
            Pattern::Star => String::from("*"),
            Pattern::Number(num) => num.to_string(),
        }
    }
}

impl ToString for AtRulePart {
    fn to_string(&self) -> String {
        match self {
            AtRulePart::Keyframes => String::from("@keyframes"),
            AtRulePart::Name(name) => name.clone(),
            AtRulePart::Percentage(num) => format!("{}%", num),
        }
    }
}

pub fn get_combinator_type(token_kind: CssSyntaxKind) -> Combinator {
    match token_kind {
        CssSyntaxKind::CSS_SPACE_LITERAL => Combinator::Descendant,
        CssSyntaxKind::R_ANGLE => Combinator::DirectDescendant,
        CssSyntaxKind::PLUS => Combinator::Plus,
        _ => panic!("unexpected token = {:?}", token_kind),
    }
}

impl DBPath for biome_css_syntax::AnyCssSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            CssBogusSelector(_) => panic!(),
            CssComplexSelector(s) => {
                let left = s.left().unwrap();
                let right = s.right().unwrap();
                let rhs_paths = right.to_css_db_paths();
                let combinator =
                    Part::Combinator(get_combinator_type(s.combinator().unwrap().kind()));

                left.to_css_db_paths()
                    .iter()
                    .flat_map(|lhs| {
                        rhs_paths.iter().map(|rhs| {
                            [lhs.clone(), vec![combinator.clone()], rhs.clone()].concat()
                        })
                    })
                    .collect()
            }
            CssCompoundSelector(selector) => selector.to_css_db_paths(),
        }
    }
}

impl DBPath for CssUniversalSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        vec![vec![Part::Pattern(Pattern::Star)]]
    }
}

impl DBPath for biome_css_syntax::AnyCssSimpleSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            biome_css_syntax::AnyCssSimpleSelector::CssTypeSelector(t) => {
                vec![vec![Part::Pattern(Pattern::Element(
                    t.ident()
                        .unwrap()
                        .value_token()
                        .unwrap()
                        .text_trimmed()
                        .to_string(),
                ))]]
            }
            biome_css_syntax::AnyCssSimpleSelector::CssUniversalSelector(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for biome_css_syntax::CssCompoundSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self.simple_selector() {
            Some(lhs) => {
                let lhs_paths = lhs.to_css_db_paths();

                if self.sub_selectors().into_iter().count() == 0 {
                    return lhs_paths;
                }

                // sub selectors are like ".btn.help" -> ".btn", ".help"
                let rhs_paths = self
                    .sub_selectors()
                    .into_iter()
                    .map(|selector| selector.to_css_db_paths())
                    .fold::<Vec<Vec<Part>>, _>(vec![], |acc_paths, cur_paths| {
                        if acc_paths.is_empty() {
                            cur_paths
                        } else {
                            acc_paths
                                .iter()
                                .flat_map(|lhs| {
                                    cur_paths
                                        .iter()
                                        .map(|rhs| [lhs.clone(), rhs.clone()].concat())
                                })
                                .collect()
                        }
                    });

                lhs_paths
                    .iter()
                    .flat_map(|lhs_path| {
                        rhs_paths
                            .iter()
                            .map(|rhs_path| [lhs_path.clone(), rhs_path.clone()].concat())
                    })
                    .collect()
            }
            None => self
                .sub_selectors()
                .into_iter()
                .map(|selector| selector.to_css_db_paths())
                .fold::<Vec<Vec<Part>>, _>(vec![], |acc_paths, cur_paths| {
                    if acc_paths.is_empty() {
                        cur_paths
                    } else {
                        acc_paths
                            .iter()
                            .flat_map(|lhs| {
                                cur_paths
                                    .iter()
                                    .map(|rhs| [lhs.clone(), rhs.clone()].concat())
                            })
                            .collect()
                    }
                }),
        }
    }
}

impl DBPath for CssPseudoClassFunctionRelativeSelectorList {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let name = self.name_token().unwrap();
        let relative_selectors = self.relative_selectors();

        let list_of_paths: Vec<Vec<Vec<Part>>> = relative_selectors
            .clone()
            .into_iter()
            .map(|s| s.unwrap())
            .map(|s| s.to_css_db_paths())
            .collect();

        // eg. body:has(button.active) -> ["body", ":has(", "button.active", ")"]
        // this encoding allows us to navigate siblings of "button.active"
        // ... although ... now I'm wondering .. can't we just encode it like
        // ["body", ":has(", "button", ".active", ")"]
        // ["body", ":has(", "button" ")"]
        // ... what would be the consequence of this?
        // idfk, let's try it :)

        list_of_paths
            .iter()
            .map(|paths| {
                // this will break when you have :has(:is(a, b))
                assert!(paths.len() == 1);
                let path = paths.first().unwrap();

                [
                    vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(
                        name.text_trimmed().to_string(),
                    ))],
                    path.clone(),
                    vec![Part::Pattern(Pattern::CloseSelectorList)],
                ]
                .concat()
            })
            .collect()
    }
}

impl DBPath for CssPseudoClassNth {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassNthIdentifier {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassNthNumber {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        assert!(self.sign().is_none());
        let number = self.value().unwrap();
        let number = number.value_token().unwrap();
        let number = number.text_trimmed();
        let number: i32 = number.parse().unwrap();

        vec![vec![Part::Pattern(Pattern::Number(number))]]
    }
}

impl DBPath for AnyCssPseudoClassNth {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            AnyCssPseudoClassNth::CssPseudoClassNth(s) => s.to_css_db_paths(),
            AnyCssPseudoClassNth::CssPseudoClassNthIdentifier(s) => s.to_css_db_paths(),
            AnyCssPseudoClassNth::CssPseudoClassNthNumber(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for CssPseudoClassNthSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        assert!(self.of_selector().is_none());
        self.nth().unwrap().to_css_db_paths()
    }
}

impl DBPath for AnyCssPseudoClassNthSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => todo!(),
            AnyCssPseudoClassNthSelector::CssPseudoClassNthSelector(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for CssPseudoClassFunctionNth {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let name = self.name().unwrap().text_trimmed().to_string();
        let paths = self.selector().unwrap().to_css_db_paths();
        assert!(paths.len() == 1);
        let path = paths.first().unwrap().clone();

        vec![[
            vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(name))],
            path,
            vec![Part::Pattern(Pattern::CloseSelectorList)],
        ]
        .concat()]
    }
}

impl DBPath for CssPseudoClassFunctionValueList {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionCompoundSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionCompoundSelectorList {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionIdentifier {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionSelectorList {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let name = self.name().unwrap().text_trimmed().to_string();

        let list_of_paths: Vec<Vec<Vec<Part>>> = self
            .selectors()
            .into_iter()
            .map(|result| result.unwrap())
            .map(|list| list.to_css_db_paths())
            .collect();

        list_of_paths
            .iter()
            .flat_map(|paths| {
                paths.iter().map(|path| {
                    [
                        vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(
                            name.clone(),
                        ))],
                        path.clone(),
                        vec![Part::Pattern(Pattern::CloseSelectorList)],
                    ]
                    .concat()
                })
            })
            .collect()
    }
}

impl DBPath for AnyCssPseudoClass {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            AnyCssPseudoClass::CssBogusPseudoClass(_) => panic!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelector(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelectorList(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionIdentifier(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionNth(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionRelativeSelectorList(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelector(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelectorList(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionValueList(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassIdentifier(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![Part::Pattern(Pattern::PseudoClass(name.to_string()))]]
            }
        }
    }
}

impl DBPath for CssAttributeSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let name = self.name().unwrap();
        let name = name
            .name()
            .unwrap()
            .value_token()
            .unwrap()
            .text_trimmed()
            .to_string();
        match self.matcher() {
            Some(matcher) => {
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().unwrap();
                let value = matcher.value().unwrap();

                // [data-kind="rule"] -> ['[data-kind]', '[data-kind="rule"]']
                // so that you can explore siblings along [data-kind]
                vec![vec![
                    Part::Pattern(Pattern::Attribute(name.clone())),
                    Part::Pattern(Pattern::AttributeMatch(
                        name.clone(),
                        operator.to_string(),
                        value.to_string(),
                    )),
                ]]
            }
            None => {
                vec![vec![Part::Pattern(Pattern::Attribute(name))]]
            }
        }
    }
}

impl DBPath for CssKeyframesAtRule {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let name = self.name().unwrap();
        let name = name.as_css_custom_identifier().unwrap();
        let name = name.value_token().unwrap();

        vec![vec![
            Part::AtRule(AtRulePart::Keyframes),
            Part::AtRule(AtRulePart::Name(name.text_trimmed().to_string())),
        ]]
    }
}

impl DBPath for CssKeyframesPercentageSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let selector = self.selector().unwrap();
        let num: i32 = selector
            .as_fields()
            .value_token
            .unwrap()
            .text_trimmed()
            .parse()
            .unwrap();
        vec![vec![Part::AtRule(AtRulePart::Percentage(num))]]
    }
}

impl DBPath for CssKeyframesSelectorList {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        self.into_iter()
            .map(|s| s.unwrap())
            .map(|s| match s {
                AnyCssKeyframesSelector::CssBogusSelector(_) => todo!(),
                AnyCssKeyframesSelector::CssKeyframesIdentSelector(_) => todo!(),
                AnyCssKeyframesSelector::CssKeyframesPercentageSelector(pct) => {
                    pct.to_css_db_paths()
                }
            })
            .map(|paths| {
                assert!(paths.len() == 1);
                paths.first().unwrap().clone()
            })
            .collect()
    }
}

impl DBPath for AnyCssAtRule {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            AnyCssAtRule::CssBogusAtRule(_) => todo!(),
            AnyCssAtRule::CssCharsetAtRule(_) => todo!(),
            AnyCssAtRule::CssColorProfileAtRule(_) => todo!(),
            AnyCssAtRule::CssContainerAtRule(_) => todo!(),
            AnyCssAtRule::CssCounterStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssDocumentAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFaceAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFeatureValuesAtRule(_) => todo!(),
            AnyCssAtRule::CssFontPaletteValuesAtRule(_) => todo!(),
            AnyCssAtRule::CssImportAtRule(_) => todo!(),
            AnyCssAtRule::CssKeyframesAtRule(r) => r.to_css_db_paths(),
            AnyCssAtRule::CssLayerAtRule(_) => todo!(),
            AnyCssAtRule::CssMediaAtRule(_) => todo!(),
            AnyCssAtRule::CssNamespaceAtRule(_) => todo!(),
            AnyCssAtRule::CssPageAtRule(_) => todo!(),
            AnyCssAtRule::CssPropertyAtRule(_) => todo!(),
            AnyCssAtRule::CssScopeAtRule(_) => todo!(),
            AnyCssAtRule::CssStartingStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssSupportsAtRule(_) => todo!(),
        }
    }
}

impl DBPath for CssPseudoElementFunctionIdentifier {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for CssPseudoElementFunctionSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        todo!()
    }
}

impl DBPath for AnyCssPseudoElement {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            AnyCssPseudoElement::CssBogusPseudoElement(_) => panic!(),
            AnyCssPseudoElement::CssPseudoElementFunctionIdentifier(s) => s.to_css_db_paths(),
            AnyCssPseudoElement::CssPseudoElementFunctionSelector(s) => s.to_css_db_paths(),
            AnyCssPseudoElement::CssPseudoElementIdentifier(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![Part::Pattern(Pattern::PseudoElement(
                    name.to_string(),
                ))]]
            }
        }
    }
}

impl DBPath for CssRelativeSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        let paths = self.selector().unwrap().to_css_db_paths();
        if let Some(combinator) = self.combinator() {
            // prepend combinator to all the paths
            paths
                .iter()
                .map(|path| {
                    [
                        vec![Part::Combinator(get_combinator_type(combinator.kind()))],
                        path.clone(),
                    ]
                    .concat()
                })
                .collect()
        } else {
            paths
        }
    }
}

impl DBPath for AnyCssRelativeSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => panic!(),
            AnyCssRelativeSelector::CssRelativeSelector(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for AnyCssSubSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<Part>> {
        match self {
            CssAttributeSelector(attribute_selector) => attribute_selector.to_css_db_paths(),
            CssBogusSubSelector(_) => vec![],
            CssClassSelector(class) => {
                let name = class.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![Part::Pattern(Pattern::Class(name.to_string()))]]
            }
            CssIdSelector(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![Part::Pattern(Pattern::Id(name.to_owned()))]]
            }
            CssPseudoClassSelector(pseudo_class) => pseudo_class.class().unwrap().to_css_db_paths(),
            CssPseudoElementSelector(pseudo_element) => {
                pseudo_element.element().unwrap().to_css_db_paths()
            }
        }
    }
}
