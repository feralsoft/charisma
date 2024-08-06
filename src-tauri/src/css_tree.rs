use crate::{parse_utils::parse_property, CharismaError};
use biome_css_syntax::{
    AnyCssAtRule, AnyCssDeclarationOrRuleBlock, AnyCssKeyframesSelector, AnyCssMediaCondition,
    AnyCssMediaInParens, AnyCssMediaQuery, AnyCssMediaTypeQuery, AnyCssPseudoClass,
    AnyCssPseudoClassNth, AnyCssPseudoClassNthSelector, AnyCssPseudoElement, AnyCssQueryFeature,
    AnyCssRelativeSelector, AnyCssRule, AnyCssRuleListBlock,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::{self, *},
    CssAttributeSelector, CssDeclarationOrRuleBlock, CssDeclarationWithSemicolon,
    CssFontFaceAtRule, CssKeyframesAtRule, CssKeyframesIdentSelector,
    CssKeyframesPercentageSelector, CssKeyframesSelectorList, CssMediaAtRule,
    CssMediaConditionInParens, CssMediaConditionQuery, CssMediaFeatureInParens,
    CssPseudoClassFunctionCompoundSelector, CssPseudoClassFunctionCompoundSelectorList,
    CssPseudoClassFunctionIdentifier, CssPseudoClassFunctionNth,
    CssPseudoClassFunctionRelativeSelectorList, CssPseudoClassFunctionSelector,
    CssPseudoClassFunctionSelectorList, CssPseudoClassFunctionValueList, CssPseudoClassNth,
    CssPseudoClassNthIdentifier, CssPseudoClassNthNumber, CssPseudoClassNthSelector,
    CssPseudoElementFunctionIdentifier, CssPseudoElementFunctionSelector, CssQueryFeaturePlain,
    CssRelativeSelector, CssRelativeSelectorList, CssSelectorList, CssSubSelectorList,
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

pub trait ToSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Result<Selector, CharismaError>;
}

impl ToSelector for CssSelectorList {
    fn to_selector(&self, _parent: Option<&Selector>) -> Result<Selector, CharismaError> {
        let path = self.to_css_tree_path()?;

        let list: Result<Vec<String>, _> = self
            .into_iter()
            .map(|s| {
                s.map(|s| s.to_string())
                    .map_err(|e| CharismaError::ParseError(e.to_string()))
            })
            .collect();

        Ok(Selector {
            string: list?.join(", ").trim().to_string(),
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
                Some(c) => get_combinator_type(c.kind())?,
                None => Combinator::Descendant,
            }
        };

        let path = [
            parent.map(|p| p.path.clone()).unwrap_or_default(),
            vec![Part::Combinator(combinator.clone())],
            selector
                .selector()
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .to_css_tree_path()?,
        ]
        .concat();

        Ok(Selector {
            string: self.to_string().trim().to_string(),
            path,
        })
    }
}

impl ToSelector for AnyCssSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Result<Selector, CharismaError> {
        Ok(Selector {
            string: self.to_string().trim().to_string(),
            path: [
                parent.map(|p| p.path.clone()).unwrap_or_default(),
                self.to_css_tree_path()?,
            ]
            .concat(),
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

        self.properties.push(Arc::new(new_property));
        self.properties.sort_by_key(|p| p.name.clone())
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
pub struct MediaQuery {
    pub path: Vec<Part>,
    pub string: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    Regular(RegularRule),
    Keyframes(Keyframes),
    Media(MediaQuery, RegularRule),
    // there's no good way to index @font-face since the selector doesn't include
    // something to use
    FontFace(Vec<FontFace>),
    Bogus(Vec<String>),
}

impl Rule {
    pub fn as_regular_rule(&self) -> Option<RegularRule> {
        match self {
            Rule::Regular(rule) => Some(rule.clone()),
            _ => None,
        }
    }

    pub fn as_mut_regular_rule(&mut self) -> Option<&mut RegularRule> {
        match self {
            Rule::Regular(rule) => Some(rule),
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

fn get_comments(str: &str) -> Result<Vec<String>, CharismaError> {
    let mut idx = 0;
    let mut comments: Vec<String> = vec![];
    while str[idx..].contains("/*") {
        match (str[idx..].find("/*"), str[idx..].find("*/")) {
            (Some(start), Some(end)) => {
                comments.push(str[(idx + start + 2)..(idx + end)].to_string());
                idx += end + 2;
            }
            (None, None) => {}
            _ => {
                return Err(CharismaError::ParseError(
                    "unexpected pattern during comment parsing".into(),
                ))
            }
        }
    }
    Ok(comments)
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
            .filter(|p| p.as_str() == path)
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
        )?);
        comments.extend(get_comments(
            block
                .r_curly_token()
                .map_err(|e| CharismaError::ParseError(e.to_string()))?
                .token_text()
                .text(),
        )?);

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
                biome_css_syntax::AnyCssDeclarationOrRule::CssBogus(_) => {
                    return Err(CharismaError::ParseError(property.to_string()))
                }
                biome_css_syntax::AnyCssDeclarationOrRule::CssDeclarationWithSemicolon(
                    property,
                ) => {
                    comments.extend(get_comments(&property.to_string())?);
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
                        biome_css_syntax::AnyCssKeyframesItem::CssBogusKeyframesItem(_) => {
                            return Err(CharismaError::NotSupported("bogus @keyframes item".into()))
                        }
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
                                    .fold(String::new(), |acc, cur| {
                                        format!("{} {}", acc.trim(), cur.trim())
                                    });
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
                )
            }
            AnyCssAtRule::CssBogusAtRule(_) => {
                Err(CharismaError::NotSupported("bogus at rule".into()))
            }
            AnyCssAtRule::CssCharsetAtRule(_) => {
                Err(CharismaError::NotSupported("char set".into()))
            }
            AnyCssAtRule::CssColorProfileAtRule(_) => {
                Err(CharismaError::NotSupported("color profile".into()))
            }
            AnyCssAtRule::CssContainerAtRule(_) => {
                Err(CharismaError::NotSupported("@container".into()))
            }
            AnyCssAtRule::CssCounterStyleAtRule(_) => {
                Err(CharismaError::NotSupported("counter".into()))
            }
            AnyCssAtRule::CssDocumentAtRule(_) => {
                Err(CharismaError::NotSupported("at document".into()))
            }
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
                        .and_then(|d| d.property())
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?;
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

                self.insert_font_face(FontFace { properties })
            }
            AnyCssAtRule::CssFontFeatureValuesAtRule(_) => {
                Err(CharismaError::NotSupported("font-features".into()))
            }
            AnyCssAtRule::CssFontPaletteValuesAtRule(_) => {
                Err(CharismaError::NotSupported("font palette".into()))
            }
            AnyCssAtRule::CssImportAtRule(_) => Err(CharismaError::NotSupported("@import".into())),
            AnyCssAtRule::CssLayerAtRule(_) => Err(CharismaError::NotSupported("@layer".into())),
            AnyCssAtRule::CssMediaAtRule(rule) => self.insert_media_at_rule(rule),
            AnyCssAtRule::CssNamespaceAtRule(_) => {
                Err(CharismaError::NotSupported("namespace".into()))
            }
            AnyCssAtRule::CssPageAtRule(_) => Err(CharismaError::NotSupported("page".into())),
            AnyCssAtRule::CssPropertyAtRule(_) => {
                Err(CharismaError::NotSupported("@property".into()))
            }
            AnyCssAtRule::CssScopeAtRule(_) => Err(CharismaError::NotSupported("@scope".into())),
            AnyCssAtRule::CssStartingStyleAtRule(_) => {
                Err(CharismaError::NotSupported("@starting-style".into()))
            }
            AnyCssAtRule::CssSupportsAtRule(_) => {
                Err(CharismaError::NotSupported("@supports".into()))
            }
        }
    }

    fn insert_media_at_rule(&mut self, at_rule: CssMediaAtRule) -> Result<(), CharismaError> {
        let path = at_rule.to_css_tree_path()?;
        println!("path = {:?}", path);
        let queries = at_rule
            .queries()
            .into_iter()
            .map(|q| q.unwrap().to_string())
            .collect::<String>();

        let block = at_rule
            .block()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        match block {
            AnyCssRuleListBlock::CssBogusBlock(_) => {
                return Err(CharismaError::ParseError(block.to_string()))
            }
            AnyCssRuleListBlock::CssRuleListBlock(ref b) => {
                for rule in b.rules() {
                    let rule = rule.as_css_qualified_rule().unwrap();
                    let selector = rule.prelude().to_selector(None)?;
                    match rule
                        .block()
                        .map_err(|e| CharismaError::ParseError(e.to_string()))?
                    {
                        AnyCssDeclarationOrRuleBlock::CssBogusBlock(_) => {
                            Err(CharismaError::ParseError(block.to_string()))
                        }
                        AnyCssDeclarationOrRuleBlock::CssDeclarationOrRuleBlock(block) => self
                            .load_rule(
                                Selector {
                                    string: format!("@media {} {}", queries, selector.string),
                                    path: [path.clone(), selector.path].concat(),
                                },
                                &block,
                            ),
                    }?
                }
            }
        }

        Err(CharismaError::NotSupported("@media".into()))
    }

    pub fn load(&mut self, css_path: &str) -> Vec<CharismaError> {
        let css = match fs::read_to_string(css_path) {
            Ok(css) => css,
            Err(_) => return vec![CharismaError::FileNotFound(css_path.to_string())],
        };
        let mut errors: Vec<CharismaError> = vec![];
        let ast = biome_css_parser::parse_css(&css, biome_css_parser::CssParserOptions::default());
        for rule in ast.tree().rules() {
            match rule {
                AnyCssRule::CssQualifiedRule(rule) => match rule.prelude().to_selector(None) {
                    Ok(selector) => {
                        let block = match rule.block() {
                            Ok(block) => match block.as_css_declaration_or_rule_block() {
                                Some(block) => block.clone(),
                                None => {
                                    errors.push(CharismaError::ParseError(block.to_string()));
                                    continue;
                                }
                            },
                            Err(e) => {
                                errors.push(CharismaError::ParseError(e.to_string()));
                                continue;
                            }
                        };
                        self.insert_empty_regular_rule(&selector);
                        if let Err(e) = self.load_rule(selector, &block) {
                            errors.push(e)
                        }
                    }
                    Err(e) => {
                        errors.push(e);
                        if let Err(e) = self.insert_bogus_rule(rule.to_string()) {
                            errors.push(e);
                        }
                    }
                },
                AnyCssRule::CssAtRule(at_rule) => {
                    if let Err(e) = self.load_at_rule(at_rule.rule().unwrap()) {
                        errors.push(e);
                        if let Err(e) = self.insert_bogus_rule(at_rule.to_string()) {
                            errors.push(e);
                        }
                    }
                }
                AnyCssRule::CssBogusRule(_) => {
                    if let Err(e) = self.insert_bogus_rule(rule.to_string()) {
                        errors.push(e);
                    }
                    errors.push(CharismaError::NotSupported(rule.to_string()))
                }
                AnyCssRule::CssNestedQualifiedRule(_) => {
                    errors.push(CharismaError::NotSupported(rule.to_string()))
                }
            };
        }
        self.current_path = Some(css_path.to_string());

        errors
    }

    pub fn serialize(&self) -> String {
        let rule = match &self.rule {
            Some(Rule::Regular(RegularRule {
                properties,
                selector,
            })) => {
                if properties.is_empty() {
                    String::new()
                } else {
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
            }
            Some(Rule::FontFace(fonts)) => fonts.iter().fold(String::new(), |mut out, f| {
                let _ = write!(
                    out,
                    "@font-face {{\n    {}}}\n",
                    f.properties
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<String>()
                );
                out
            }),
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
            Some(Rule::Bogus(rules)) => rules.concat(),
            Some(Rule::Media(q, rule)) => {
                if rule.properties.is_empty() {
                    String::new()
                } else {
                    format!(
                        "@media ({}) {{{}}}",
                        q.string,
                        format!(
                            "{} {{\n    {}\n}}\n",
                            rule.selector.string,
                            rule.properties
                                .iter()
                                .map(|p| p.to_string() + "\n    ")
                                .collect::<String>()
                                .trim()
                        )
                    )
                }
            }
            None => String::from(""),
        };

        let mut children: Vec<(&Part, &CssTree)> = self.children.iter().collect();
        children.sort_by_key(|(p, _)| p.to_owned());

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
    fn all_selectors_with_properties_aux(&self, selectors: &mut Vec<String>) {
        match self.rule.as_ref() {
            Some(Rule::Regular(rule)) => {
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
            Some(Rule::Bogus(_)) => {}
            Some(Rule::Media(_, _)) => {
                //
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
            Some(Rule::Regular(rule)) => {
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
            Some(Rule::Bogus(_)) => {}
            Some(Rule::Media(_, _)) => {}
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

    pub fn drain(&mut self) -> Result<(), CharismaError> {
        match &mut self.rule {
            Some(Rule::Regular(rule)) => {
                rule.properties.drain(0..);
                Ok(())
            }
            Some(Rule::Keyframes(_)) => Err(CharismaError::NotSupported(
                "can't drain @keyframes rule".to_string(),
            )),
            Some(Rule::FontFace(_)) => Err(CharismaError::NotSupported(
                "can't drain @font-face rule".to_string(),
            )),
            Some(Rule::Bogus(_)) => Err(CharismaError::NotSupported(
                "can't drain bogus rule".to_string(),
            )),
            Some(Rule::Media(_, _)) => Err(CharismaError::NotSupported(
                "can't drain media rule".to_string(),
            )),
            None => Err(CharismaError::NotSupported(
                "can't drain empty rule".to_string(),
            )),
        }
    }

    pub fn set_state(
        &mut self,
        path: &[Part],
        property_name: &str,
        property_value: &str,
        state: State,
    ) -> Result<(), CharismaError> {
        let rule = match self.get_mut(path).and_then(|t| t.rule.as_mut()) {
            Some(r) => r,
            None => return Err(CharismaError::RuleNotFound),
        };

        match rule {
            Rule::Regular(rule) => {
                rule.comment_all_with_name(property_name);
                if state == State::Valid {
                    rule.insert(Property {
                        name: property_name.to_string(),
                        value: property_value.to_string(),
                        state,
                    });
                }
                Ok(())
            }
            Rule::Keyframes(_) => Err(CharismaError::NotSupported(
                "can't edit @keyframes rule".into(),
            )),
            Rule::FontFace(_) => Err(CharismaError::NotSupported(
                "can't edit @font-face rule".into(),
            )),
            Rule::Media(_, _) => Err(CharismaError::NotSupported("can't edit @media rule".into())),
            Rule::Bogus(_) => Err(CharismaError::NotSupported("can't edit bogus rule".into())),
        }
    }

    pub fn delete(
        &mut self,
        path: &[Part],
        property_name: &str,
        property_value: &str,
    ) -> Result<(), CharismaError> {
        let rule = match self.get_mut(path).and_then(|t| t.rule.as_mut()) {
            Some(r) => r,
            None => return Err(CharismaError::RuleNotFound),
        };

        match rule {
            Rule::Regular(rule) => {
                rule.properties
                    .retain(|p| !(p.name == property_name && p.value == property_value));
                Ok(())
            }
            Rule::Keyframes(_) => Err(CharismaError::NotSupported(
                "can't delete property for @keyframes rule".into(),
            )),
            Rule::FontFace(_) => Err(CharismaError::NotSupported(
                "can't delete property for @font-face rule".into(),
            )),
            Rule::Bogus(_) => Err(CharismaError::NotSupported(
                "can't delete property for bogus rule".into(),
            )),
            Rule::Media(_, _) => Err(CharismaError::NotSupported(
                "can't delete property for @media rule".into(),
            )),
        }
    }

    fn insert_font_face(&mut self, fontface: FontFace) -> Result<(), CharismaError> {
        match self
            .get_mut(&[Part::AtRule(AtRulePart::Fontface)])
            .and_then(|t| t.rule.as_mut())
        {
            Some(Rule::FontFace(fonts)) => {
                fonts.push(fontface);
                Ok(())
            }
            Some(_) => Err(CharismaError::AssertionError(
                "should have a font here".into(),
            )),
            None => self.insert_raw(
                &[Part::AtRule(AtRulePart::Fontface)],
                Rule::FontFace(vec![fontface]),
            ),
        }
    }

    fn insert_raw(&mut self, path: &[Part], rule: Rule) -> Result<(), CharismaError> {
        match path {
            [] => match &mut self.rule {
                Some(_) => Err(CharismaError::AssertionError(
                    "failed to insert raw rule, rule already exists".into(),
                )),
                None => {
                    self.rule = Some(rule);
                    Ok(())
                }
            },
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_raw(parts, rule),
                None => {
                    let mut new_tree = CssTree::new();
                    new_tree.insert_raw(parts, rule)?;
                    self.children.insert(part.to_owned(), new_tree);
                    Ok(())
                }
            },
        }
    }

    fn insert_raw_regular_rule(
        &mut self,
        selector: Selector,
        path: &[Part],
        property: Property,
    ) -> Result<(), CharismaError> {
        match path {
            [] => match &mut self.rule {
                Some(Rule::Regular(rule)) => {
                    rule.insert(property);
                    Ok(())
                }
                Some(_) => Err(CharismaError::AssertionError("not a regular rule".into())),
                None => {
                    let mut rule = RegularRule::new(selector);
                    rule.insert(property);
                    self.rule = Some(Rule::Regular(rule));
                    Ok(())
                }
            },
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_raw_regular_rule(selector, parts, property),
                None => {
                    let mut new_tree = CssTree::new();
                    new_tree.insert_raw_regular_rule(selector, parts, property)?;
                    self.children.insert(part.to_owned(), new_tree);
                    Ok(())
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
        )
    }

    fn insert_empty_regular_rule_aux(&mut self, selector: Selector, path: &[Part]) {
        match path {
            [] => {
                match &mut self.rule {
                    Some(_) => {} // already exists
                    None => self.rule = Some(Rule::Regular(RegularRule::new(selector))),
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

    pub fn insert_bogus_rule(&mut self, rule: String) -> Result<(), CharismaError> {
        match self.children.get_mut(&Part::Bogus) {
            Some(t) => match t.rule.as_mut() {
                Some(r) => match r {
                    Rule::Bogus(rules) => {
                        rules.push(rule);
                        Ok(())
                    }
                    _ => Err(CharismaError::AssertionError(
                        "unexpected non-bogus rule at bogus location".into(),
                    )),
                },
                None => {
                    t.rule = Some(Rule::Bogus(vec![rule]));
                    Ok(())
                }
            },
            None => {
                let mut tree = CssTree::new();
                tree.rule = Some(Rule::Bogus(vec![rule]));
                self.children.insert(Part::Bogus, tree);
                Ok(())
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
        )
    }

    pub fn insert_regular_property(
        &mut self,
        selector: &Selector,
        property: &Property,
    ) -> Result<(), CharismaError> {
        self.insert_raw_regular_rule(selector.clone(), &selector.path, property.clone())
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AtRulePart {
    // @keyframes
    Keyframes,
    // @font-face
    Fontface,
    // @media
    Media,
    // eg. preferse-reduced-motion
    FeatureName(String),
    // eg. no-preference
    FeatureValue(String),
    // keyframe-name
    Name(String),
    // `from` or `to`
    Identifier(String),
    // 20%
    Percentage(i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Part {
    Combinator(Combinator),
    Pattern(Pattern),
    AtRule(AtRulePart),
    Comma,
    Bogus,
}

impl Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Part::Combinator(c) => c.fmt(f),
            Part::Pattern(p) => p.fmt(f),
            Part::AtRule(a) => a.fmt(f),
            Part::Comma => write!(f, ", "),
            Part::Bogus => panic!(),
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
            AtRulePart::Media => write!(f, "@media")?,
            AtRulePart::FeatureName(_) => todo!(),
            AtRulePart::FeatureValue(_) => todo!(),
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

impl CssTreePath for AnyCssSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            CssBogusSelector(_) => Err(CharismaError::ParseError(self.to_string())),
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
                let sign = match o.sign().map(|s| s.kind()) {
                    Ok(CssSyntaxKind::PLUS) => Ok(Sign::Plus),
                    Ok(CssSyntaxKind::MINUS) => Ok(Sign::Minus),
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
        Err(CharismaError::NotSupported(self.to_string()))
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
            AnyCssPseudoClassNthSelector::CssBogusSelector(_) => {
                Err(CharismaError::ParseError("bogus selector".into()))
            }
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
        Err(CharismaError::NotSupported(self.to_string()))
    }
}

impl CssTreePath for CssPseudoClassFunctionCompoundSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Err(CharismaError::NotSupported(self.to_string()))
    }
}

impl CssTreePath for CssPseudoClassFunctionCompoundSelectorList {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Err(CharismaError::NotSupported(self.to_string()))
    }
}

impl CssTreePath for CssPseudoClassFunctionIdentifier {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Err(CharismaError::NotSupported(self.to_string()))
    }
}

impl CssTreePath for CssPseudoClassFunctionSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Err(CharismaError::NotSupported(self.to_string()))
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
            AnyCssPseudoClass::CssBogusPseudoClass(_) => {
                Err(CharismaError::ParseError(self.to_string()))
            }
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
        let name = self
            .name()
            .and_then(|n| n.name())
            .and_then(|n| n.value_token())
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
            AnyCssKeyframesSelector::CssBogusSelector(_) => {
                Err(CharismaError::NotSupported("".into()))
            }
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

impl CssTreePath for AnyCssMediaTypeQuery {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssMediaTypeQuery::CssMediaAndTypeQuery(_) => todo!(),
            AnyCssMediaTypeQuery::CssMediaTypeQuery(_) => todo!(),
        }
    }
}

impl CssTreePath for CssMediaConditionInParens {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        self.condition()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .to_css_tree_path()
    }
}

impl CssTreePath for CssQueryFeaturePlain {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let name = self
            .name()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;
        let value = self
            .value()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?;

        Ok(vec![
            Part::AtRule(AtRulePart::FeatureName(name.to_string())),
            Part::AtRule(AtRulePart::FeatureValue(value.to_string())),
        ])
    }
}

impl CssTreePath for AnyCssQueryFeature {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssQueryFeature::CssQueryFeatureBoolean(_) => todo!(),
            AnyCssQueryFeature::CssQueryFeaturePlain(f) => f.to_css_tree_path(),
            AnyCssQueryFeature::CssQueryFeatureRange(_) => todo!(),
            AnyCssQueryFeature::CssQueryFeatureRangeInterval(_) => todo!(),
            AnyCssQueryFeature::CssQueryFeatureReverseRange(_) => todo!(),
        }
    }
}

impl CssTreePath for CssMediaFeatureInParens {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        self.feature()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .to_css_tree_path()
    }
}

impl CssTreePath for AnyCssMediaInParens {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssMediaInParens::CssMediaConditionInParens(c) => c.to_css_tree_path(),
            AnyCssMediaInParens::CssMediaFeatureInParens(c) => c.to_css_tree_path(),
        }
    }
}

impl CssTreePath for AnyCssMediaCondition {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssMediaCondition::AnyCssMediaInParens(m) => m.to_css_tree_path(),
            AnyCssMediaCondition::CssMediaAndCondition(_) => todo!(),
            AnyCssMediaCondition::CssMediaNotCondition(_) => todo!(),
            AnyCssMediaCondition::CssMediaOrCondition(_) => todo!(),
        }
    }
}

impl CssTreePath for CssMediaConditionQuery {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        self.condition()
            .map_err(|e| CharismaError::ParseError(e.to_string()))?
            .to_css_tree_path()
    }
}

impl CssTreePath for AnyCssMediaQuery {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssMediaQuery::AnyCssMediaTypeQuery(q) => q.to_css_tree_path(),
            AnyCssMediaQuery::CssBogusMediaQuery(_) => todo!(),
            AnyCssMediaQuery::CssMediaConditionQuery(q) => q.to_css_tree_path(),
        }
    }
}

impl CssTreePath for CssMediaAtRule {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        let queries = self
            .queries()
            .into_iter()
            .map(|r| r.map_err(|e| CharismaError::ParseError(e.to_string())))
            .collect::<Result<Vec<_>, _>>()?;

        let mut path: Vec<Part> = vec![Part::AtRule(AtRulePart::Media)];
        for sub_path in queries.iter().map(|q| q.to_css_tree_path()) {
            path.extend(sub_path?)
        }

        Ok(path)
    }
}

impl CssTreePath for AnyCssAtRule {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssAtRule::CssBogusAtRule(_) => Err(CharismaError::NotSupported(self.to_string())),
            AnyCssAtRule::CssCharsetAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssColorProfileAtRule(r) => {
                Err(CharismaError::NotSupported(r.to_string()))
            }
            AnyCssAtRule::CssContainerAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssCounterStyleAtRule(r) => {
                Err(CharismaError::NotSupported(r.to_string()))
            }
            AnyCssAtRule::CssDocumentAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssFontFaceAtRule(r) => r.to_css_tree_path(),
            AnyCssAtRule::CssFontFeatureValuesAtRule(r) => {
                Err(CharismaError::NotSupported(r.to_string()))
            }
            AnyCssAtRule::CssFontPaletteValuesAtRule(r) => {
                Err(CharismaError::NotSupported(r.to_string()))
            }
            AnyCssAtRule::CssImportAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssKeyframesAtRule(r) => r.to_css_tree_path(),
            AnyCssAtRule::CssLayerAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssMediaAtRule(r) => r.to_css_tree_path(),
            AnyCssAtRule::CssNamespaceAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssPageAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssPropertyAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssScopeAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
            AnyCssAtRule::CssStartingStyleAtRule(r) => {
                Err(CharismaError::NotSupported(r.to_string()))
            }
            AnyCssAtRule::CssSupportsAtRule(r) => Err(CharismaError::NotSupported(r.to_string())),
        }
    }
}

impl CssTreePath for CssPseudoElementFunctionIdentifier {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Err(CharismaError::NotSupported(self.to_string()))
    }
}

impl CssTreePath for CssPseudoElementFunctionSelector {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        Err(CharismaError::NotSupported(self.to_string()))
    }
}

impl CssTreePath for AnyCssPseudoElement {
    fn to_css_tree_path(&self) -> Result<Vec<Part>, CharismaError> {
        match self {
            AnyCssPseudoElement::CssBogusPseudoElement(_) => {
                Err(CharismaError::ParseError(self.to_string()))
            }
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
            AnyCssRelativeSelector::CssBogusSelector(_) => {
                Err(CharismaError::ParseError(self.to_string()))
            }
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
