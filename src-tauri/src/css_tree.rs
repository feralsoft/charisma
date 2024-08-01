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
    CssRelativeSelectorList, CssSelectorList, CssSubSelectorList, CssSyntaxKind,
    CssUniversalSelector,
};
use std::{collections::HashMap, fmt::Display, fs, sync::Arc};
use std::{fmt::Write, string::ParseError};

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

pub trait ToSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Result<Selector, CharismaError>;
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

impl ToSelector for CssSelectorList {
    fn to_selector(&self, _parent: Option<&Selector>) -> Result<Selector, CharismaError> {
        let path = self.to_css_tree_path()?;

        Ok(Selector {
            string: path_to_string(&path),
            path,
        })
    }
}

impl ToSelector for AnyCssRelativeSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Result<Selector, CharismaError> {
        let selector = self.as_css_relative_selector().unwrap();
        // this fucking sucks.. I would assume `.combinator` would do this, but it doesn't

        let combinator = if selector.to_string().trim().starts_with('&') {
            Combinator::And
        } else {
            match selector.combinator() {
                Some(c) => get_combinator_type(c.kind()),
                None => Ok(Combinator::Descendant),
            }?
        };
        let selector = selector.selector().unwrap();

        let path = [
            parent.map(|p| p.path.clone()).unwrap_or_default(),
            vec![Part::Combinator(combinator.clone())],
            selector.to_css_tree_path()?,
        ]
        .concat();

        Ok(Selector {
            string: path_to_string(&path),
            path,
        })
    }
}

impl ToSelector for AnyCssSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Result<Selector, CharismaError> {
        let path = [
            parent.map(|p| p.path.clone()).unwrap_or_default(),
            self.to_css_tree_path()?,
        ]
        .concat();

        Ok(Selector {
            string: path_to_string(&path),
            path,
        })
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
        let decl = node
            .declaration()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let property = decl
            .property()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let property = property.as_css_generic_property().unwrap();
        let name = property
            .name()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let name = name.as_css_identifier().unwrap();
        let name = name
            .value_token()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        Ok(name.text_trimmed().to_string())
    }

    pub fn value(node: &CssDeclarationWithSemicolon) -> Result<String, CharismaError> {
        let decl = node
            .declaration()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let property = decl
            .property()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
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

    pub fn remove(&mut self, property_to_remove: &Property) {
        assert!(self
            .properties
            .iter()
            .any(|p| p.name == property_to_remove.name && p.value == property_to_remove.value));

        self.properties = self
            .properties
            .iter()
            .filter(|existing_property| {
                !(existing_property.name == property_to_remove.name
                    && existing_property.value == property_to_remove.value)
            })
            .cloned()
            .collect::<Vec<_>>();
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

    pub fn as_mut_regular_rule(&mut self) -> Option<&mut RegularRule> {
        match self {
            Rule::RegularRule(rule) => Some(rule),
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
pub struct CssTree {
    children: HashMap<Part, CssTree>,
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

impl CssTree {
    pub fn new() -> CssTree {
        CssTree {
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
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .token_text()
                .text(),
        ));
        comments.extend(get_comments(
            block
                .r_curly_token()
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .token_text()
                .text(),
        ));

        for property in block.items() {
            match property {
                biome_css_syntax::AnyCssDeclarationOrRule::AnyCssRule(rule) => {
                    let rule = rule.as_css_nested_qualified_rule().unwrap();
                    let block = rule
                        .block()
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                    let block = block.as_css_declaration_or_rule_block().unwrap();
                    for child in rule.prelude() {
                        let child = child.map_err(|e| CharismaError::ParseError(e.to_string()))?;
                        let selector = child.to_selector(Some(&selector))?;
                        self.load_rule(selector, block)?;
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

        for property in comments.iter().map(|str| parse_property(str)) {
            self.insert_regular_rule_commented(&selector, property?)?;
        }

        Ok(())
    }

    fn load_at_rule(&mut self, at_rule: AnyCssAtRule) -> Result<(), CharismaError> {
        let at_rule_path = at_rule.to_css_tree_path()?;
        match at_rule {
            AnyCssAtRule::CssKeyframesAtRule(rule) => {
                let name = rule
                    .name()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let block = rule
                    .block()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let block = block.as_css_keyframes_block().unwrap();
                let mut frames: Vec<Frame> = vec![];
                for item in block.items() {
                    let frame = match item {
                        biome_css_syntax::AnyCssKeyframesItem::CssBogusKeyframesItem(_) => todo!(),
                        biome_css_syntax::AnyCssKeyframesItem::CssKeyframesItem(item) => {
                            let path = item.selectors().to_css_tree_path()?;

                            let block = item
                                .block()
                                .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                            let block = block.as_css_declaration_list_block().unwrap();
                            let mut properties: Vec<Property> = vec![];
                            for property in block.declarations() {
                                let property = property
                                    .declaration()
                                    .map_err(|e| CharismaError::ParseError(e.to_string()))?
                                    .property()
                                    .unwrap();
                                let property = property.as_css_generic_property().unwrap();
                                let name = property
                                    .name()
                                    .map_err(|e| CharismaError::ParseError(e.to_string()))?
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
                    &at_rule_path,
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
                let block = rule
                    .block()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let block = block.as_css_declaration_list_block().unwrap();
                let path = rule.to_css_tree_path()?;
                assert!(path.len() == 1);
                let mut properties: Vec<Property> = vec![];
                for property in block.declarations() {
                    let property = property
                        .declaration()
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?
                        .property()
                        .unwrap();
                    let property = property.as_css_generic_property().unwrap();
                    let name = property
                        .name()
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?
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
                    let selector = rule.prelude().to_selector(None)?;
                    let block = rule.block().unwrap();
                    let block = block.as_css_declaration_or_rule_block().unwrap();
                    self.insert_empty_regular_rule(&selector);
                    self.load_rule(selector, block)?;
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

        let mut children: Vec<(&Part, &CssTree)> = self.children.iter().collect();
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
                    let mut new_tree = CssTree::new();
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
                    let mut new_tree = CssTree::new();
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
                    let mut new_tree = CssTree::new();
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
                self.children.insert(keyframes_part.clone(), CssTree::new());
                self.children.get_mut(&keyframes_part).unwrap()
            }
        };
        match tree.children.get(&name_part) {
            Some(_) => {} // already there
            None => {
                let mut dst = CssTree::new();
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

    pub fn insert_regular_property(
        &mut self,
        selector: &Selector,
        property: &Property,
    ) -> Result<(), CharismaError> {
        self.insert_raw_regular_rule(selector.clone(), &selector.path, property.clone());
        Ok(())
    }

    pub fn get(&self, path: &[Part]) -> Option<&CssTree> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get(part).and_then(|c| c.get(parts)),
        }
    }

    pub fn get_mut(&mut self, path: &[Part]) -> Option<&mut CssTree> {
        match path {
            [] => Some(self),
            [part, parts @ ..] => self.children.get_mut(part).and_then(|c| c.get_mut(parts)),
        }
    }
}

pub trait CssTreePath {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError>;
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
    // borked
    Bogus,
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
    Comma,
}

impl Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Combinator(c) => c.fmt(f),
            Part::Pattern(p) => p.fmt(f),
            Part::AtRule(a) => a.fmt(f),
            Part::Comma => write!(f, ", "),
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
            Combinator::Bogus => return Err(std::fmt::Error),
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

pub fn get_combinator_type(token_kind: CssSyntaxKind) -> Result<Combinator, CharismaError> {
    match token_kind {
        CssSyntaxKind::CSS_SPACE_LITERAL => Ok(Combinator::Descendant),
        CssSyntaxKind::R_ANGLE => Ok(Combinator::DirectDescendant),
        CssSyntaxKind::PLUS => Ok(Combinator::Plus),
        _ => Err(CharismaError::ParseError(format!(
            "unexpected token = {:?}",
            token_kind
        ))),
    }
}

impl CssTreePath for biome_css_syntax::AnyCssSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            CssBogusSelector(_) => panic!(),
            CssComplexSelector(s) => {
                let left = s
                    .left()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;

                let right = s
                    .right()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;

                let combinator = Part::Combinator(get_combinator_type(
                    s.combinator()
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?
                        .kind(),
                )?);

                let lhs_path = left.to_css_tree_path()?;
                let rhs_path = right.to_css_tree_path()?;

                Ok([lhs_path, vec![combinator], rhs_path].concat())
            }
            CssCompoundSelector(selector) => selector.to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssUniversalSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Ok(vec![Part::Pattern(Pattern::Star)])
    }
}

impl CssTreePath for biome_css_syntax::AnyCssSimpleSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            biome_css_syntax::AnyCssSimpleSelector::CssTypeSelector(t) => {
                Ok(vec![Part::Pattern(Pattern::Element(
                    t.ident()
                        .and_then(|id| id.value_token())
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?
                        .text_trimmed()
                        .to_string(),
                ))])
            }
            biome_css_syntax::AnyCssSimpleSelector::CssUniversalSelector(s) => s.to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssSubSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        self.into_iter()
            .map(|item| item.to_css_tree_path())
            .collect::<Result<Vec<Vec<Part>>, CharismaError>>()
            .map(|items| items.concat())
    }
}

impl CssTreePath for biome_css_syntax::CssCompoundSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self.simple_selector() {
            Some(lhs) => {
                let lhs_path = lhs.to_css_tree_path()?;
                let sub_selector_path = self.sub_selectors().to_css_tree_path()?;
                Ok([lhs_path, sub_selector_path].concat())
            }
            None => self.sub_selectors().to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssRelativeSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let list = self.into_iter().collect::<Vec<_>>();
        match list.as_slice() {
            [] => Ok(vec![]),
            [item] => item
                .as_ref()
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .to_css_tree_path(),
            items => {
                panic!("wtf is this = {:?}", items)
            }
        }
    }
}

impl CssTreePath for CssPseudoClassFunctionRelativeSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let name = self
            .name_token()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;

        Ok([
            vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(
                name.text_trimmed().to_string(),
            ))],
            self.relative_selectors().to_css_tree_path()?,
            vec![Part::Pattern(Pattern::CloseSelectorList)],
        ]
        .concat())
    }
}

impl CssTreePath for CssPseudoClassNth {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        assert!(self.sign().is_none()); // don't know what this is
        assert!(self.symbol_token().unwrap().to_string().trim() == "n");
        let value = self.value().unwrap();
        let value = value
            .to_string()
            .parse::<i32>()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        match self.offset() {
            Some(o) => {
                let sign = match o.sign().unwrap().kind() {
                    CssSyntaxKind::PLUS => Ok(Sign::Plus),
                    CssSyntaxKind::MINUS => Ok(Sign::Minus),
                    _ => Err(CharismaError::ParseError("can't determine nth sign".into())),
                }?;

                let offset: i32 = o.value().unwrap().to_string().trim().parse().unwrap();
                Ok(vec![Part::Pattern(Pattern::NthWithOffset(
                    value, sign, offset,
                ))])
            }
            None => Ok(vec![Part::Pattern(Pattern::Nth(value))]),
        }
    }
}

impl CssTreePath for CssPseudoClassNthIdentifier {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssPseudoClassNthNumber {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        assert!(self.sign().is_none());
        let number = self
            .value()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let number = number
            .value_token()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let number = number.text_trimmed();
        let number = number
            .parse::<i32>()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        Ok(vec![Part::Pattern(Pattern::Number(number))])
    }
}

impl CssTreePath for AnyCssPseudoClassNth {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssPseudoClassNth::CssPseudoClassNth(s) => s.to_css_tree_path(),
            AnyCssPseudoClassNth::CssPseudoClassNthIdentifier(s) => s.to_css_tree_path(),
            AnyCssPseudoClassNth::CssPseudoClassNthNumber(s) => s.to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssPseudoClassNthSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        assert!(self.of_selector().is_none());
        self.nth()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .to_css_tree_path()
    }
}

impl CssTreePath for AnyCssPseudoClassNthSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => todo!(),
            AnyCssPseudoClassNthSelector::CssPseudoClassNthSelector(s) => s.to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssPseudoClassFunctionNth {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let name = self
            .name()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .text_trimmed()
            .to_string();
        let path = self
            .selector()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .to_css_tree_path()?;

        Ok([
            vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(name))],
            path,
            vec![Part::Pattern(Pattern::CloseSelectorList)],
        ]
        .concat())
    }
}

impl CssTreePath for CssPseudoClassFunctionValueList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssPseudoClassFunctionCompoundSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssPseudoClassFunctionCompoundSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssPseudoClassFunctionIdentifier {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssPseudoClassFunctionSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let list = self
            .into_iter()
            .map(|item| {
                item.as_ref()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))
                    .and_then(|item| item.to_css_tree_path())
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(list.iter().fold(vec![], |acc, path| {
            if acc.is_empty() {
                path.clone()
            } else {
                [acc, vec![Part::Comma], path.clone()].concat()
            }
        }))
    }
}

impl CssTreePath for CssPseudoClassFunctionSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let name = self
            .name()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .text_trimmed()
            .to_string();

        Ok([
            vec![Part::Pattern(Pattern::PseudoClassWithSelectorList(
                name.clone(),
            ))],
            self.selectors().to_css_tree_path()?,
            vec![Part::Pattern(Pattern::CloseSelectorList)],
        ]
        .concat())
    }
}

impl CssTreePath for AnyCssPseudoClass {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssPseudoClass::CssBogusPseudoClass(_) => panic!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelector(s) => s.to_css_tree_path(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelectorList(s) => {
                s.to_css_tree_path()
            }
            AnyCssPseudoClass::CssPseudoClassFunctionIdentifier(s) => s.to_css_tree_path(),
            AnyCssPseudoClass::CssPseudoClassFunctionNth(s) => s.to_css_tree_path(),
            AnyCssPseudoClass::CssPseudoClassFunctionRelativeSelectorList(s) => {
                s.to_css_tree_path()
            }
            AnyCssPseudoClass::CssPseudoClassFunctionSelector(s) => s.to_css_tree_path(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelectorList(s) => s.to_css_tree_path(),
            AnyCssPseudoClass::CssPseudoClassFunctionValueList(s) => s.to_css_tree_path(),
            AnyCssPseudoClass::CssPseudoClassIdentifier(id) => {
                let name = id
                    .name()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?
                    .value_token()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let name = name.text_trimmed();
                Ok(vec![Part::Pattern(Pattern::PseudoClass(name.to_string()))])
            }
        }
    }
}

impl CssTreePath for CssAttributeSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let name = self.name().unwrap();
        let name = name
            .name()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .value_token()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .text_trimmed()
            .to_string();
        match self.matcher() {
            Some(matcher) => {
                assert!(matcher.modifier().is_none());
                let operator = matcher
                    .operator()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;

                let value = matcher
                    .value()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                // [data-kind="rule"] -> ['[data-kind]', '[data-kind="rule"]']
                // so that you can explore siblings along [data-kind]
                Ok(vec![
                    Part::Pattern(Pattern::Attribute(name.clone())),
                    Part::Pattern(Pattern::AttributeMatch(
                        name.clone(),
                        operator.to_string(),
                        value.to_string(),
                    )),
                ])
            }
            None => Ok(vec![Part::Pattern(Pattern::Attribute(name))]),
        }
    }
}

impl CssTreePath for CssKeyframesAtRule {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let name = self
            .name()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let name = name.as_css_custom_identifier().unwrap();
        let name = name
            .value_token()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        Ok(vec![
            Part::AtRule(AtRulePart::Keyframes),
            Part::AtRule(AtRulePart::Name(name.text_trimmed().to_string())),
        ])
    }
}

impl CssTreePath for CssKeyframesIdentSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let selector = self
            .selector()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        Ok(vec![Part::AtRule(AtRulePart::Identifier(
            selector.text_trimmed().to_string(),
        ))])
    }
}

impl CssTreePath for CssKeyframesPercentageSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let selector = self
            .selector()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let num = selector
            .value_token()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .text_trimmed()
            .parse::<i32>()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        Ok(vec![Part::AtRule(AtRulePart::Percentage(num))])
    }
}

impl CssTreePath for CssKeyframesSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let list: Vec<_> = self.into_iter().collect();
        assert!(list.len() == 1);
        let item = list.first().unwrap().as_ref();
        let item = item.map_err(|e| CharismaError::ParseError(e.to_string()))?;

        match item {
            AnyCssKeyframesSelector::CssBogusSelector(_) => todo!(),
            AnyCssKeyframesSelector::CssKeyframesIdentSelector(id) => id.to_css_tree_path(),
            AnyCssKeyframesSelector::CssKeyframesPercentageSelector(s) => s.to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssFontFaceAtRule {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Ok(vec![Part::AtRule(AtRulePart::Fontface)])
    }
}

impl CssTreePath for AnyCssAtRule {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssAtRule::CssBogusAtRule(_) => todo!(),
            AnyCssAtRule::CssCharsetAtRule(_) => todo!(),
            AnyCssAtRule::CssColorProfileAtRule(_) => todo!(),
            AnyCssAtRule::CssContainerAtRule(_) => todo!(),
            AnyCssAtRule::CssCounterStyleAtRule(_) => todo!(),
            AnyCssAtRule::CssDocumentAtRule(_) => todo!(),
            AnyCssAtRule::CssFontFaceAtRule(r) => r.to_css_tree_path(),
            AnyCssAtRule::CssFontFeatureValuesAtRule(_) => todo!(),
            AnyCssAtRule::CssFontPaletteValuesAtRule(_) => todo!(),
            AnyCssAtRule::CssImportAtRule(_) => todo!(),
            AnyCssAtRule::CssKeyframesAtRule(r) => r.to_css_tree_path(),
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

impl CssTreePath for CssPseudoElementFunctionIdentifier {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for CssPseudoElementFunctionSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        todo!()
    }
}

impl CssTreePath for AnyCssPseudoElement {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssPseudoElement::CssBogusPseudoElement(_) => panic!(),
            AnyCssPseudoElement::CssPseudoElementFunctionIdentifier(s) => s.to_css_tree_path(),
            AnyCssPseudoElement::CssPseudoElementFunctionSelector(s) => s.to_css_tree_path(),
            AnyCssPseudoElement::CssPseudoElementIdentifier(id) => {
                let name = id
                    .name()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?
                    .value_token()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let name = name.text_trimmed();
                Ok(vec![Part::Pattern(Pattern::PseudoElement(
                    name.to_string(),
                ))])
            }
        }
    }
}

impl CssTreePath for CssRelativeSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let path = self
            .selector()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .to_css_tree_path()?;
        if let Some(combinator) = self.combinator() {
            // prepend combinator to all the paths
            Ok([
                vec![Part::Combinator(get_combinator_type(combinator.kind())?)],
                path.clone(),
            ]
            .concat())
        } else {
            Ok(path)
        }
    }
}

impl CssTreePath for AnyCssRelativeSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => panic!(),
            AnyCssRelativeSelector::CssRelativeSelector(s) => s.to_css_tree_path(),
        }
    }
}

impl CssTreePath for AnyCssSubSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            CssAttributeSelector(attribute_selector) => attribute_selector.to_css_tree_path(),
            CssBogusSubSelector(_) => {
                Err(CharismaError::ParseError("bogus sub selector".to_string()))
            }
            CssClassSelector(class) => {
                let name = class
                    .name()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?
                    .value_token()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let name = name.text_trimmed();
                Ok(vec![Part::Pattern(Pattern::Class(name.to_string()))])
            }
            CssIdSelector(id) => {
                let name = id
                    .name()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?
                    .value_token()
                    .map_err(|e| CharismaError::ParseError(e.to_string()))?;
                let name = name.text_trimmed();
                Ok(vec![Part::Pattern(Pattern::Id(name.to_owned()))])
            }
            CssPseudoClassSelector(pseudo_class) => pseudo_class
                .class()
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .to_css_tree_path(),
            CssPseudoElementSelector(pseudo_element) => pseudo_element
                .element()
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .to_css_tree_path(),
        }
    }
}
