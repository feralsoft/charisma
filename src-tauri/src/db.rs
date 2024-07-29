use crate::{parse_utils::parse_property, CharismaError};
use biome_css_syntax::{
    AnyCssAtRule, AnyCssKeyframesSelector, AnyCssPseudoClass, AnyCssPseudoClassNth,
    AnyCssPseudoClassNthSelector, AnyCssPseudoElement, AnyCssRelativeSelector, AnyCssRule,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::{self, *},
    CssAttributeSelector, CssDeclarationOrRuleBlock, CssDeclarationWithSemicolon,
    CssFontFaceAtRule, CssKeyframesAtRule, CssKeyframesIdentSelector,
    CssKeyframesPercentageSelector, CssKeyframesSelectorList,
    CssPseudoClassFunctionCompoundSelector, CssPseudoClassFunctionCompoundSelectorList,
    CssPseudoClassFunctionIdentifier, CssPseudoClassFunctionNth,
    CssPseudoClassFunctionRelativeSelectorList, CssPseudoClassFunctionSelector,
    CssPseudoClassFunctionSelectorList, CssPseudoClassFunctionValueList, CssPseudoClassNth,
    CssPseudoClassNthIdentifier, CssPseudoClassNthNumber, CssPseudoClassNthSelector,
    CssPseudoElementFunctionIdentifier, CssPseudoElementFunctionSelector, CssRelativeSelector,
    CssSyntaxKind, CssUniversalSelector,
};
use std::fmt::Write;
use std::{collections::HashMap, fmt::Display, fs, sync::Arc};

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
    fn to_selectors(&self, parent: Option<&Selector>) -> Result<Vec<Selector>, CharismaError>;
}

fn path_to_string(path: &[Part]) -> String {
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

    condensed_path
        .iter()
        .map(|p| p.to_string())
        .collect::<String>()
}

impl ToSelectors for AnyCssRelativeSelector {
    fn to_selectors(&self, parent: Option<&Selector>) -> Result<Vec<Selector>, CharismaError> {
        let selector = self.as_css_relative_selector().unwrap();
        // this fucking sucks.. I would assume `.combinator` would do this, but it doesn't

        let combinator = if selector.to_string().trim().starts_with('&') {
            Combinator::And
        } else {
            selector
                .combinator()
                .map(|c| get_combinator_type(c.kind()))
                .unwrap_or(Combinator::Descendant)
        };
        let selector = selector.selector().unwrap();

        Ok(selector
            .to_css_db_paths()?
            .iter()
            .map(|path| {
                let path = [
                    parent.map(|p| p.path.clone()).unwrap_or_default(),
                    vec![Part::Combinator(combinator.clone())],
                    path.clone(),
                ]
                .concat();

                Selector {
                    string: path_to_string(&path),
                    path,
                }
            })
            .collect())
    }
}

impl ToSelectors for AnyCssSelector {
    fn to_selectors(&self, parent: Option<&Selector>) -> Result<Vec<Selector>, CharismaError> {
        Ok(self
            .to_css_db_paths()?
            .iter()
            .map(|path| {
                let path = [
                    parent.map(|p| p.path.clone()).unwrap_or_default(),
                    path.clone(),
                ]
                .concat();
                Selector {
                    string: path_to_string(&path),
                    path,
                }
            })
            .collect())
    }
}

impl Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            State::Valid => write!(f, "{}: {};", self.name, self.value),
            State::Commented => write!(f, "/* {}: {}; */", self.name, self.value),
        }
    }
}

impl Property {
    pub fn name(node: &CssDeclarationWithSemicolon) -> Result<String, CharismaError> {
        let decl = node.declaration().map_err(|_| CharismaError::ParseError)?;
        let property = decl.property().map_err(|_| CharismaError::ParseError)?;
        let property = property.as_css_generic_property().unwrap();
        let name = property.name().map_err(|_| CharismaError::ParseError)?;
        let name = name.as_css_identifier().unwrap();
        let name = name.value_token().map_err(|_| CharismaError::ParseError)?;
        Ok(name.text_trimmed().to_string())
    }

    pub fn value(node: &CssDeclarationWithSemicolon) -> Result<String, CharismaError> {
        let decl = node.declaration().map_err(|_| CharismaError::ParseError)?;
        let property = decl.property().map_err(|_| CharismaError::ParseError)?;
        let property = property.as_css_generic_property().unwrap();
        Ok(property
            .value()
            .into_iter()
            .map(|item| item.to_string())
            .collect::<String>()
            .trim()
            .to_string())
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
                if p.name == name {
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
    pub path: Vec<Part>,
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframes {
    pub name: String,
    pub frames: Vec<Frame>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontFace {
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    RegularRule(RegularRule),
    Keyframes(Keyframes),
    // there's no good way to index @font-face since the selector doesn't include
    // something to use
    FontFace(Vec<FontFace>),
}

impl Rule {
    pub fn as_regular_rule(&self) -> Option<RegularRule> {
        match self {
            Rule::RegularRule(rule) => Some(rule.clone()),
            _ => None,
        }
    }

    pub fn as_keyframes(&self) -> Option<Keyframes> {
        match self {
            Rule::Keyframes(rule) => Some(rule.clone()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssDB {
    children: HashMap<Part, CssDB>,
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

impl CssDB {
    pub fn new() -> CssDB {
        CssDB {
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

    fn load_rule(
        &mut self,
        selector: Selector,
        block: &CssDeclarationOrRuleBlock,
    ) -> Result<(), CharismaError> {
        let mut comments: Vec<String> = vec![];
        comments.extend(get_comments(
            block
                .l_curly_token()
                .map_err(|_| CharismaError::ParseError)?
                .token_text()
                .text(),
        ));
        comments.extend(get_comments(
            block
                .r_curly_token()
                .map_err(|_| CharismaError::ParseError)?
                .token_text()
                .text(),
        ));

        for property in block.items() {
            match property {
                biome_css_syntax::AnyCssDeclarationOrRule::AnyCssRule(rule) => {
                    let rule = rule.as_css_nested_qualified_rule().unwrap();
                    let block = rule.block().map_err(|_| CharismaError::ParseError)?;
                    let block = block.as_css_declaration_or_rule_block().unwrap();
                    for child in rule.prelude() {
                        let child = child.map_err(|_| CharismaError::ParseError)?;
                        for selector in child.to_selectors(Some(&selector))? {
                            self.load_rule(selector, block)?;
                        }
                    }
                }
                biome_css_syntax::AnyCssDeclarationOrRule::CssBogus(_) => panic!(),
                biome_css_syntax::AnyCssDeclarationOrRule::CssDeclarationWithSemicolon(
                    property,
                ) => {
                    comments.extend(get_comments(&property.to_string()));
                    self.insert_regular_rule(&selector, &property)?;
                }
            }
        }

        for property in comments.iter().filter_map(|str| parse_property(str)) {
            self.insert_regular_rule_commented(&selector, property)?;
        }

        Ok(())
    }

    fn load_at_rule(&mut self, at_rule: AnyCssAtRule) -> Result<(), CharismaError> {
        let at_rule_paths = at_rule.to_css_db_paths()?;
        assert!(at_rule_paths.len() == 1);
        let at_rule_path = at_rule_paths.first().unwrap();
        match at_rule {
            AnyCssAtRule::CssKeyframesAtRule(rule) => {
                let name = rule.name().map_err(|_| CharismaError::ParseError)?;
                let block = rule.block().map_err(|_| CharismaError::ParseError)?;
                let block = block.as_css_keyframes_block().unwrap();
                let mut frames: Vec<Frame> = vec![];
                for item in block.items() {
                    let frame = match item {
                        biome_css_syntax::AnyCssKeyframesItem::CssBogusKeyframesItem(_) => todo!(),
                        biome_css_syntax::AnyCssKeyframesItem::CssKeyframesItem(item) => {
                            let paths = item.selectors().to_css_db_paths()?;
                            assert!(paths.len() == 1);
                            let path = paths.first().unwrap().to_owned();

                            let block = item.block().map_err(|_| CharismaError::ParseError)?;
                            let block = block.as_css_declaration_list_block().unwrap();
                            let mut properties: Vec<Property> = vec![];
                            for property in block.declarations() {
                                let property = property
                                    .declaration()
                                    .map_err(|_| CharismaError::ParseError)?
                                    .property()
                                    .unwrap();
                                let property = property.as_css_generic_property().unwrap();
                                let name = property
                                    .name()
                                    .map_err(|_| CharismaError::ParseError)?
                                    .to_string()
                                    .trim()
                                    .to_string();
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
                    Rule::Keyframes(Keyframes {
                        name: name.to_string().trim().to_string(),
                        frames,
                    }),
                );

                Ok(())
            }
            AnyCssAtRule::CssBogusAtRule(_) => todo!(),
            AnyCssAtRule::CssCharsetAtRule(_) => todo!(),
            AnyCssAtRule::CssColorProfileAtRule(_) => todo!(),
            AnyCssAtRule::CssContainerAtRule(_) => todo!(),
            AnyCssAtRule::CssCounterStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssDocumentAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFaceAtRule(rule) => {
                let block = rule.block().map_err(|_| CharismaError::ParseError)?;
                let block = block.as_css_declaration_list_block().unwrap();
                let path = rule.to_css_db_paths()?;
                assert!(path.len() == 1);
                let mut properties: Vec<Property> = vec![];
                for property in block.declarations() {
                    let property = property
                        .declaration()
                        .map_err(|_| CharismaError::ParseError)?
                        .property()
                        .unwrap();
                    let property = property.as_css_generic_property().unwrap();
                    let name = property
                        .name()
                        .map_err(|_| CharismaError::ParseError)?
                        .to_string()
                        .trim()
                        .to_string();
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
                    });
                }

                self.insert_font_face(FontFace { properties });

                Ok(())
            }
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

    pub fn load(&mut self, css_path: &str) -> Result<(), CharismaError> {
        let css = fs::read_to_string(css_path).unwrap();
        let ast = biome_css_parser::parse_css(&css, biome_css_parser::CssParserOptions::default());
        for rule in ast.tree().rules() {
            match rule {
                AnyCssRule::CssQualifiedRule(rule) => {
                    for selector in rule.prelude() {
                        let block = rule.block().unwrap();
                        let block = block.as_css_declaration_or_rule_block().unwrap();
                        for selector in selector.unwrap().to_selectors(None)? {
                            self.insert_empty_regular_rule(&selector);
                            self.load_rule(selector, block)?;
                        }
                    }
                }
                AnyCssRule::CssAtRule(at_rule) => self.load_at_rule(at_rule.rule().unwrap())?,
                AnyCssRule::CssBogusRule(_) => todo!(),
                AnyCssRule::CssNestedQualifiedRule(_) => todo!(),
            };
        }
        self.current_path = Some(css_path.to_string());

        Ok(())
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
            Some(Rule::FontFace(fonts)) => fonts
                .iter()
                .map(|f| {
                    format!(
                        "@font-face {{\n    {}}}\n",
                        f.properties
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<String>()
                    )
                })
                .collect::<String>(),
            Some(Rule::Keyframes(Keyframes { name, frames })) => {
                format!(
                    "@keyframes {} {{\n    {}\n}}\n",
                    name,
                    frames
                        .iter()
                        .fold(String::new(), |mut out, p| {
                            let _ = write!(
                                out,
                                "{} {{\n        {}\n    }}\n    ",
                                // ugh this is so bad
                                p.path.last().unwrap(),
                                p.properties
                                    .iter()
                                    .fold(String::new(), |mut out, p| {
                                        let _ = write!(out, "{}\n        ", p);
                                        out
                                    })
                                    .trim()
                            );
                            out
                        })
                        .trim()
                )
            }
            None => String::from(""),
        };

        let mut children: Vec<(&Part, &CssDB)> = self.children.iter().collect();
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

    // listen here kid
    // we are going to stretch the definition of "selector"
    // a wee-lil-bit so that "@keyframes animation_name"
    // is also a selector
    //
    // if that doesn't sit well with you, I can understand that
    // but us men have to get some work done around here
    fn all_selectors_with_properties_aux(&self, selectors: &mut Vec<String>) {
        match self.rule.as_ref() {
            Some(Rule::RegularRule(rule)) => {
                if !rule.properties.is_empty() {
                    selectors.push(rule.selector.string.to_owned());
                }
            }
            Some(Rule::Keyframes(rule)) => {
                if !rule.frames.is_empty() {
                    selectors.push(format!("@keyframes {}", rule.name));
                }
            }
            Some(Rule::FontFace(_fonts)) => {
                // TODO: what do we show here?
            }
            None => {}
        }
        for tree in self.children.values() {
            tree.all_selectors_with_properties_aux(selectors);
        }
    }

    fn recursive_search_for_property_aux(
        &self,
        q: &[&str],
        properties: &mut Vec<(Arc<Property>, Selector)>,
    ) {
        match self.rule.as_ref() {
            Some(Rule::RegularRule(rule)) => {
                for property in rule.properties.iter() {
                    if q.iter()
                        .all(|q| property.name.contains(q) || property.value.contains(q))
                    {
                        properties.push((property.clone(), rule.selector.clone()))
                    }
                }
            }
            Some(Rule::Keyframes(_)) => {
                // TODO!
            }
            Some(Rule::FontFace(_)) => {
                // TODO!
            }
            None => {}
        }
        for child in self.children.values() {
            child.recursive_search_for_property_aux(q, properties)
        }
    }

    pub fn recursive_search_for_property(&self, q: &[&str]) -> Vec<(Arc<Property>, Selector)> {
        let mut properties: Vec<(Arc<Property>, Selector)> = vec![];
        self.recursive_search_for_property_aux(q, &mut properties);
        properties
    }

    pub fn all_selectors_with_properties(&self) -> Vec<String> {
        let mut selectors: Vec<String> = vec![];
        self.all_selectors_with_properties_aux(&mut selectors);
        selectors
    }

    pub fn drain(&mut self) {
        match &mut self.rule {
            Some(Rule::RegularRule(rule)) => rule.properties.drain(0..),
            Some(Rule::Keyframes(_)) => panic!("drain"),
            Some(Rule::FontFace(_)) => panic!("drain"),
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
        let rule = self.get_mut(path).and_then(|t| t.rule.as_mut()).unwrap();

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
            Rule::Keyframes(_) => panic!(),
            Rule::FontFace(_) => panic!(),
        }
    }

    pub fn delete(&mut self, path: &[Part], property_name: &str, property_value: &str) {
        let rule = self.get_mut(path).and_then(|t| t.rule.as_mut()).unwrap();

        match rule {
            Rule::RegularRule(rule) => {
                rule.properties
                    .retain(|p| !(p.name == property_name && p.value == property_value));
            }
            Rule::Keyframes(_) => panic!(),
            Rule::FontFace(_) => panic!(),
        }
    }

    fn insert_font_face(&mut self, fontface: FontFace) {
        match self
            .get_mut(&[Part::AtRule(AtRulePart::Fontface)])
            .and_then(|t| t.rule.as_mut())
        {
            Some(Rule::FontFace(fonts)) => fonts.push(fontface),
            Some(_) => panic!("should have a font here"),
            None => self.insert_raw(
                &[Part::AtRule(AtRulePart::Fontface)],
                Rule::FontFace(vec![fontface]),
            ),
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
                    let mut new_tree = CssDB::new();
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
                    Some(Rule::Keyframes(_)) => panic!(),
                    Some(Rule::FontFace(_)) => panic!(),
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
                    let mut new_tree = CssDB::new();
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
    ) -> Result<(), CharismaError> {
        self.insert_raw_regular_rule(
            selector.clone(),
            &selector.path,
            Property {
                name: Property::name(&property)?,
                value: Property::value(&property)?,
                state: State::Commented,
            },
        );
        Ok(())
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
                    let mut new_tree = CssDB::new();
                    new_tree.insert_empty_regular_rule_aux(selector, parts);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn insert_empty_regular_rule(&mut self, selector: &Selector) {
        self.insert_empty_regular_rule_aux(selector.clone(), &selector.path);
    }

    pub fn insert_empty_keyframes_rule(&mut self, name: String) {
        let keyframes_part = Part::AtRule(AtRulePart::Keyframes);
        let name_part = Part::AtRule(AtRulePart::Name(name.clone()));
        let tree = match self.children.get_mut(&keyframes_part) {
            Some(tree) => tree,
            None => {
                self.children.insert(keyframes_part.clone(), CssDB::new());
                self.children.get_mut(&keyframes_part).unwrap()
            }
        };
        match tree.children.get(&name_part) {
            Some(_) => {} // already there
            None => {
                let mut dst = CssDB::new();
                dst.rule = Some(Rule::Keyframes(Keyframes {
                    name,
                    frames: vec![],
                }));
                tree.children.insert(name_part, dst);
            }
        }
    }

    pub fn insert_regular_rule(
        &mut self,
        selector: &Selector,
        property: &CssDeclarationWithSemicolon,
    ) -> Result<(), CharismaError> {
        self.insert_raw_regular_rule(
            selector.clone(),
            &selector.path,
            Property {
                name: Property::name(property)?,
                value: Property::value(property)?,
                state: State::Valid,
            },
        );

        Ok(())
    }

    pub fn get(&self, path: &[Part]) -> Option<&CssDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get(part).and_then(|c| c.get(parts)),
        }
    }

    pub fn get_mut(&mut self, path: &[Part]) -> Option<&mut CssDB> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get_mut(part).and_then(|c| c.get_mut(parts)),
        }
    }
}

pub trait DBPath {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError>;
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
pub enum Sign {
    Plus,
    Minus,
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

    // 3n
    Nth(i32),

    // 3n + 1
    NthWithOffset(i32, Sign, i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AtRulePart {
    // @keyframes
    Keyframes,
    // @font-face
    Fontface,
    // keyframe-name
    Name(String),
    // `from` or `to`
    Identifier(String),
    // 20%
    Percentage(i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    Combinator(Combinator),
    Pattern(Pattern),
    AtRule(AtRulePart),
}

impl Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Combinator(c) => c.fmt(f),
            Part::Pattern(p) => p.fmt(f),
            Part::AtRule(a) => a.fmt(f),
        }
    }
}

impl Display for Combinator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Combinator::Descendant => write!(f, " ")?,
            Combinator::DirectDescendant => write!(f, " > ")?,
            Combinator::And => write!(f, "")?,
            Combinator::Plus => write!(f, " + ")?,
        };
        Ok(())
    }
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sign::Plus => write!(f, "+"),
            Sign::Minus => write!(f, "-"),
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Attribute(name) => write!(f, "[{}]", name)?,
            Pattern::AttributeMatch(name, matcher, value) => {
                write!(f, "[{}{}{}]", name, matcher, value)?
            }
            Pattern::Class(name) => write!(f, ".{}", name)?,
            Pattern::Id(name) => write!(f, "#{}", name)?,
            Pattern::Element(name) => write!(f, "{}", name)?,
            Pattern::PseudoElement(name) => write!(f, "::{}", name)?,
            Pattern::PseudoClass(name) => write!(f, ":{}", name)?,
            Pattern::PseudoClassWithSelectorList(name) => write!(f, ":{}(", name)?,
            Pattern::CloseSelectorList => write!(f, ")")?,
            Pattern::Star => write!(f, "*")?,
            Pattern::Number(num) => write!(f, "{}", num)?,
            Pattern::Nth(n) => write!(f, "{}n", n)?,
            Pattern::NthWithOffset(n, s, o) => write!(f, "{}n {} {}", n, s, o)?,
        };
        Ok(())
    }
}

impl Display for AtRulePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtRulePart::Keyframes => write!(f, "@keyframes")?,
            AtRulePart::Fontface => write!(f, "@font-face")?,
            AtRulePart::Name(name) => write!(f, "{}", name)?,
            AtRulePart::Percentage(num) => write!(f, "{}%", num)?,
            AtRulePart::Identifier(id) => write!(f, "{}", id)?,
        }
        Ok(())
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
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            CssBogusSelector(_) => panic!(),
            CssComplexSelector(s) => {
                let left = s.left().map_err(|_| CharismaError::ParseError)?;
                let right = s.right().map_err(|_| CharismaError::ParseError)?;
                let rhs_paths = right.to_css_db_paths()?;
                let combinator = Part::Combinator(get_combinator_type(
                    s.combinator()
                        .map_err(|_| CharismaError::ParseError)?
                        .kind(),
                ));

                Ok(left
                    .to_css_db_paths()?
                    .iter()
                    .flat_map(|lhs| {
                        rhs_paths.iter().map(|rhs| {
                            [lhs.clone(), vec![combinator.clone()], rhs.clone()].concat()
                        })
                    })
                    .collect())
            }
            CssCompoundSelector(selector) => selector.to_css_db_paths(),
        }
    }
}

impl DBPath for CssUniversalSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        Ok(vec![vec![Part::Pattern(Pattern::Star)]])
    }
}

impl DBPath for biome_css_syntax::AnyCssSimpleSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            biome_css_syntax::AnyCssSimpleSelector::CssTypeSelector(t) => {
                Ok(vec![vec![Part::Pattern(Pattern::Element(
                    t.ident()
                        .and_then(|id| id.value_token())
                        .map_err(|_| CharismaError::ParseError)?
                        .text_trimmed()
                        .to_string(),
                ))]])
            }
            biome_css_syntax::AnyCssSimpleSelector::CssUniversalSelector(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for biome_css_syntax::CssCompoundSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self.simple_selector() {
            Some(lhs) => {
                let lhs_paths = lhs.to_css_db_paths()?;

                if self.sub_selectors().into_iter().count() == 0 {
                    return Ok(lhs_paths);
                }

                let sub_selector_paths = self
                    .sub_selectors()
                    .into_iter()
                    .map(|selector| selector.to_css_db_paths())
                    .collect::<Result<Vec<_>, _>>()?;

                // sub selectors are like ".btn.help" -> ".btn", ".help"
                let rhs_paths = sub_selector_paths.iter().fold::<Vec<Vec<Part>>, _>(
                    vec![],
                    |acc_paths, cur_paths| {
                        if acc_paths.is_empty() {
                            cur_paths.clone()
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
                    },
                );

                Ok(lhs_paths
                    .iter()
                    .flat_map(|lhs_path| {
                        rhs_paths
                            .iter()
                            .map(|rhs_path| [lhs_path.clone(), rhs_path.clone()].concat())
                    })
                    .collect())
            }
            None => {
                let sub_selector_paths = self
                    .sub_selectors()
                    .into_iter()
                    .map(|selector| selector.to_css_db_paths())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(sub_selector_paths.iter().fold::<Vec<Vec<Part>>, _>(
                    vec![],
                    |acc_paths, cur_paths| {
                        if acc_paths.is_empty() {
                            cur_paths.clone()
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
                    },
                ))
            }
        }
    }
}

impl DBPath for CssPseudoClassFunctionRelativeSelectorList {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let name = self.name_token().map_err(|_| CharismaError::ParseError)?;
        let relative_selectors = self
            .relative_selectors()
            .into_iter()
            .map(|s| s.map_err(|_| CharismaError::ParseError))
            .collect::<Result<Vec<_>, _>>()?;

        let list_of_paths: Vec<Vec<Vec<Part>>> = relative_selectors
            .into_iter()
            .map(|s| s.to_css_db_paths())
            .collect::<Result<_, _>>()?;

        // eg. body:has(button.active) -> ["body", ":has(", "button.active", ")"]
        // this encoding allows us to navigate siblings of "button.active"
        // ... although ... now I'm wondering .. can't we just encode it like
        // ["body", ":has(", "button", ".active", ")"]
        // ["body", ":has(", "button" ")"]
        // ... what would be the consequence of this?
        // idfk, let's try it :)

        Ok(list_of_paths
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
            .collect())
    }
}

impl DBPath for CssPseudoClassNth {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        assert!(self.sign().is_none()); // don't know what this is
        assert!(self.symbol_token().unwrap().to_string().trim() == "n");
        let value = self.value().unwrap();
        let value = value
            .to_string()
            .parse::<i32>()
            .map_err(|_| CharismaError::ParseError)?;
        match self.offset() {
            Some(o) => {
                let sign = match o.sign().unwrap().kind() {
                    CssSyntaxKind::PLUS => Sign::Plus,
                    CssSyntaxKind::MINUS => Sign::Minus,
                    _ => panic!(),
                };

                let offset: i32 = o.value().unwrap().to_string().trim().parse().unwrap();
                Ok(vec![vec![Part::Pattern(Pattern::NthWithOffset(
                    value, sign, offset,
                ))]])
            }
            None => Ok(vec![vec![Part::Pattern(Pattern::Nth(value))]]),
        }
    }
}

impl DBPath for CssPseudoClassNthIdentifier {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoClassNthNumber {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        assert!(self.sign().is_none());
        let number = self.value().map_err(|_| CharismaError::ParseError)?;
        let number = number
            .value_token()
            .map_err(|_| CharismaError::ParseError)?;
        let number = number.text_trimmed();
        let number: i32 = number.parse().map_err(|_| CharismaError::ParseError)?;

        Ok(vec![vec![Part::Pattern(Pattern::Number(number))]])
    }
}

impl DBPath for AnyCssPseudoClassNth {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            AnyCssPseudoClassNth::CssPseudoClassNth(s) => s.to_css_db_paths(),
            AnyCssPseudoClassNth::CssPseudoClassNthIdentifier(s) => s.to_css_db_paths(),
            AnyCssPseudoClassNth::CssPseudoClassNthNumber(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for CssPseudoClassNthSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        assert!(self.of_selector().is_none());
        self.nth()
            .map_err(|_| CharismaError::ParseError)?
            .to_css_db_paths()
    }
}

impl DBPath for AnyCssPseudoClassNthSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => todo!(),
            AnyCssPseudoClassNthSelector::CssPseudoClassNthSelector(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for CssPseudoClassFunctionNth {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let name = self
            .name()
            .map_err(|_| CharismaError::ParseError)?
            .text_trimmed()
            .to_string();
        let paths = self
            .selector()
            .map_err(|_| CharismaError::ParseError)?
            .to_css_db_paths()?;
        assert!(paths.len() == 1);
        let path = paths.first().unwrap().clone();

        Ok(vec![[
            vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(name))],
            path,
            vec![Part::Pattern(Pattern::CloseSelectorList)],
        ]
        .concat()])
    }
}

impl DBPath for CssPseudoClassFunctionValueList {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionCompoundSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionCompoundSelectorList {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionIdentifier {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoClassFunctionSelectorList {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let name = self
            .name()
            .map_err(|_| CharismaError::ParseError)?
            .text_trimmed()
            .to_string();

        let list_of_paths: Vec<Vec<Vec<Part>>> = self
            .selectors()
            .into_iter()
            .map(|result| {
                result
                    .map_err(|_| CharismaError::ParseError)
                    .and_then(|l| l.to_css_db_paths())
            })
            .collect::<Result<_, _>>()?;

        Ok(list_of_paths
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
            .collect())
    }
}

impl DBPath for AnyCssPseudoClass {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
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
                let name = id
                    .name()
                    .map_err(|_| CharismaError::ParseError)?
                    .value_token()
                    .map_err(|_| CharismaError::ParseError)?;
                let name = name.text_trimmed();
                Ok(vec![vec![Part::Pattern(Pattern::PseudoClass(
                    name.to_string(),
                ))]])
            }
        }
    }
}

impl DBPath for CssAttributeSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let name = self.name().unwrap();
        let name = name
            .name()
            .map_err(|_| CharismaError::ParseError)?
            .value_token()
            .map_err(|_| CharismaError::ParseError)?
            .text_trimmed()
            .to_string();
        match self.matcher() {
            Some(matcher) => {
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().map_err(|_| CharismaError::ParseError)?;
                let value = matcher.value().map_err(|_| CharismaError::ParseError)?;

                // [data-kind="rule"] -> ['[data-kind]', '[data-kind="rule"]']
                // so that you can explore siblings along [data-kind]
                Ok(vec![vec![
                    Part::Pattern(Pattern::Attribute(name.clone())),
                    Part::Pattern(Pattern::AttributeMatch(
                        name.clone(),
                        operator.to_string(),
                        value.to_string(),
                    )),
                ]])
            }
            None => Ok(vec![vec![Part::Pattern(Pattern::Attribute(name))]]),
        }
    }
}

impl DBPath for CssKeyframesAtRule {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let name = self.name().map_err(|_| CharismaError::ParseError)?;
        let name = name.as_css_custom_identifier().unwrap();
        let name = name.value_token().map_err(|_| CharismaError::ParseError)?;

        Ok(vec![vec![
            Part::AtRule(AtRulePart::Keyframes),
            Part::AtRule(AtRulePart::Name(name.text_trimmed().to_string())),
        ]])
    }
}

impl DBPath for CssKeyframesIdentSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let selector = self.selector().map_err(|_| CharismaError::ParseError)?;
        Ok(vec![vec![Part::AtRule(AtRulePart::Identifier(
            selector.text_trimmed().to_string(),
        ))]])
    }
}

impl DBPath for CssKeyframesPercentageSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let selector = self.selector().map_err(|_| CharismaError::ParseError)?;
        let num: i32 = selector
            .value_token()
            .map_err(|_| CharismaError::ParseError)?
            .text_trimmed()
            .parse()
            .map_err(|_| CharismaError::ParseError)?;
        Ok(vec![vec![Part::AtRule(AtRulePart::Percentage(num))]])
    }
}

impl DBPath for CssKeyframesSelectorList {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        self.into_iter()
            .map(|s| s.unwrap())
            .map(|s| match s {
                AnyCssKeyframesSelector::CssBogusSelector(_) => todo!(),
                AnyCssKeyframesSelector::CssKeyframesIdentSelector(id) => id.to_css_db_paths(),
                AnyCssKeyframesSelector::CssKeyframesPercentageSelector(pct) => {
                    pct.to_css_db_paths()
                }
            })
            .map(|paths| -> Result<Vec<Part>, CharismaError> {
                // assert!(paths?.len() == 1);
                Ok(paths?.first().unwrap().clone())
            })
            .collect::<Result<_, _>>()
    }
}

impl DBPath for CssFontFaceAtRule {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        Ok(vec![vec![Part::AtRule(AtRulePart::Fontface)]])
    }
}

impl DBPath for AnyCssAtRule {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            AnyCssAtRule::CssBogusAtRule(_) => todo!(),
            AnyCssAtRule::CssCharsetAtRule(_) => todo!(),
            AnyCssAtRule::CssColorProfileAtRule(_) => todo!(),
            AnyCssAtRule::CssContainerAtRule(_) => todo!(),
            AnyCssAtRule::CssCounterStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssDocumentAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFaceAtRule(r) => r.to_css_db_paths(),
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
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for CssPseudoElementFunctionSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        todo!()
    }
}

impl DBPath for AnyCssPseudoElement {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            AnyCssPseudoElement::CssBogusPseudoElement(_) => panic!(),
            AnyCssPseudoElement::CssPseudoElementFunctionIdentifier(s) => s.to_css_db_paths(),
            AnyCssPseudoElement::CssPseudoElementFunctionSelector(s) => s.to_css_db_paths(),
            AnyCssPseudoElement::CssPseudoElementIdentifier(id) => {
                let name = id
                    .name()
                    .map_err(|_| CharismaError::ParseError)?
                    .value_token()
                    .map_err(|_| CharismaError::ParseError)?;
                let name = name.text_trimmed();
                Ok(vec![vec![Part::Pattern(Pattern::PseudoElement(
                    name.to_string(),
                ))]])
            }
        }
    }
}

impl DBPath for CssRelativeSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        let paths = self
            .selector()
            .map_err(|_| CharismaError::ParseError)?
            .to_css_db_paths()?;
        if let Some(combinator) = self.combinator() {
            // prepend combinator to all the paths
            Ok(paths
                .iter()
                .map(|path| {
                    [
                        vec![Part::Combinator(get_combinator_type(combinator.kind()))],
                        path.clone(),
                    ]
                    .concat()
                })
                .collect())
        } else {
            Ok(paths)
        }
    }
}

impl DBPath for AnyCssRelativeSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => panic!(),
            AnyCssRelativeSelector::CssRelativeSelector(s) => s.to_css_db_paths(),
        }
    }
}

impl DBPath for AnyCssSubSelector {
    fn to_css_db_paths(&self) -> Result<Vec<Vec<Part>>, CharismaError> {
        match self {
            CssAttributeSelector(attribute_selector) => attribute_selector.to_css_db_paths(),
            CssBogusSubSelector(_) => Err(CharismaError::ParseError),
            CssClassSelector(class) => {
                let name = class
                    .name()
                    .map_err(|_| CharismaError::ParseError)?
                    .value_token()
                    .map_err(|_| CharismaError::ParseError)?;
                let name = name.text_trimmed();
                Ok(vec![vec![Part::Pattern(Pattern::Class(name.to_string()))]])
            }
            CssIdSelector(id) => {
                let name = id
                    .name()
                    .map_err(|_| CharismaError::ParseError)?
                    .value_token()
                    .map_err(|_| CharismaError::ParseError)?;
                let name = name.text_trimmed();
                Ok(vec![vec![Part::Pattern(Pattern::Id(name.to_owned()))]])
            }
            CssPseudoClassSelector(pseudo_class) => pseudo_class
                .class()
                .map_err(|_| CharismaError::ParseError)?
                .to_css_db_paths(),
            CssPseudoElementSelector(pseudo_element) => pseudo_element
                .element()
                .map_err(|_| CharismaError::ParseError)?
                .to_css_db_paths(),
        }
    }
}
