use crate::{parse_utils::parse_property, properties};
use std::{collections::HashMap, fs, rc::Rc};

use biome_css_syntax::{
    AnyCssPseudoClass, AnyCssPseudoElement, AnyCssRelativeSelector,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::{self, *},
    CssAttributeSelector, CssDeclarationOrRuleBlock, CssDeclarationWithSemicolon,
    CssRelativeSelector,
};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Valid,
    Commented,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub state: State,
    pub node: CssDeclarationWithSemicolon,
}

impl Property {
    fn to_commented(&self) -> Self {
        Property {
            state: State::Commented,
            node: self.node.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    pub string: String,
    pub path: Vec<String>,
}

pub trait ToSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Selector;
}

impl ToSelector for AnyCssRelativeSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Selector {
        let sel = self.as_css_relative_selector().unwrap();
        let sel = sel.selector().unwrap();
        let sel = sel.as_css_compound_selector().unwrap();
        assert!(sel.simple_selector().is_none());
        let sep = match sel.nesting_selector_token() {
            Some(_) => todo!(),
            None => " ".to_string(),
        };

        Selector {
            string: parent
                .as_ref()
                .map(|p| p.string.clone())
                .unwrap_or("".to_string())
                + &sep
                + sel.to_string().trim(),
            path: [
                parent.map(|p| p.path.clone()).unwrap_or(vec![]),
                vec![sep],
                sel.to_css_db_path(),
            ]
            .concat(),
        }
    }
}
impl ToSelector for AnyCssSelector {
    fn to_selector(&self, parent: Option<&Selector>) -> Selector {
        Selector {
            string: parent
                .as_ref()
                .map(|p| p.string.clone())
                .unwrap_or("".to_string())
                + self.to_string().trim(),
            path: [
                parent.map(|p| p.path.clone()).unwrap_or(vec![]),
                self.to_css_db_path(),
            ]
            .concat(),
        }
    }
}

impl Property {
    pub fn to_string(&self) -> String {
        let property_str = format!("{}: {};", self.name(), self.value());
        match self.state {
            State::Valid => property_str,
            State::Commented => format!("/* {} */", property_str),
        }
    }

    pub fn name(&self) -> String {
        let decl = self.node.declaration().unwrap();
        let property = decl.property().unwrap();
        let property = property.as_css_generic_property().unwrap();
        let name = property.name().unwrap();
        let name = name.as_css_identifier().unwrap();
        let name = name.value_token().unwrap();
        name.text_trimmed().to_string()
    }

    pub fn value(&self) -> String {
        let decl = self.node.declaration().unwrap();
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

    pub fn is_var(&self) -> bool {
        let decl = self.node.as_fields().declaration.unwrap();
        let property = decl.as_fields().property.unwrap();
        let property = property.as_css_generic_property().unwrap();
        let name = property.as_fields().name.unwrap();
        let name = name.as_css_identifier().unwrap();
        let name = name.value_token().unwrap();
        name.text_trimmed().starts_with("--")
    }

    pub fn var_ref(&self) -> Option<String> {
        if self.state == State::Commented {
            return None;
        }
        let decl = self.node.declaration().unwrap();
        let property = decl.property().unwrap();
        let property = property.as_css_generic_property().unwrap();
        let value = property.value().into_iter().next().unwrap();
        match value.as_any_css_value().unwrap() {
            biome_css_syntax::AnyCssValue::AnyCssFunction(f) => {
                let items = f.as_css_function().unwrap().items();
                let item = items.into_iter().next().unwrap().unwrap();
                match item.any_css_expression().unwrap() {
                    biome_css_syntax::AnyCssExpression::CssListOfComponentValuesExpression(
                        items,
                    ) => {
                        let item = items.css_component_value_list().into_iter().next().unwrap();
                        match item {
                            biome_css_syntax::AnyCssValue::CssDashedIdentifier(name) => {
                                Some(name.to_string().trim().to_string())
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selector: Selector,
    pub properties: Vec<Rc<Property>>,
}

impl Rule {
    pub fn new(selector: Selector) -> Self {
        Rule {
            selector,
            properties: vec![],
        }
    }

    pub fn comment_all_with_name(&mut self, name: &str) {
        self.properties = self
            .properties
            .iter()
            .map(|p| {
                if &p.name() == name {
                    Rc::new(p.to_commented())
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
                    !(existing_property.name() == new_property.name()
                        && existing_property.value() == new_property.value())
                })
                // if its the same name, but different value, comment out the other ones
                .map(|p| {
                    if p.name() == new_property.name() {
                        Rc::new(p.to_commented())
                    } else {
                        p.clone()
                    }
                })
                .collect();
        }

        self.properties.push(Rc::new(new_property))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CSSDB {
    children: HashMap<String, CSSDB>,
    pub rule: Option<Rule>,
}

fn get_comments(str: &str) -> Vec<String> {
    let mut idx = 0;
    let mut comments: Vec<String> = vec![];
    while str[idx..].contains('*') {
        assert!(str.chars().skip(idx).filter(|c| c == &'*').count() >= 2);
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
        }
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
                        self.load_rule(child.to_selector(Some(&selector)), block);
                    }
                }
                biome_css_syntax::AnyCssDeclarationOrRule::CssBogus(_) => todo!(),
                biome_css_syntax::AnyCssDeclarationOrRule::CssDeclarationWithSemicolon(
                    property,
                ) => {
                    comments.extend(get_comments(&property.to_string()));
                    self.insert(&selector, &property);
                }
            }
        }

        for property in comments.iter().filter_map(|str| parse_property(&str)) {
            self.insert_commented(&selector, property);
        }
    }

    pub fn load(&mut self, css_path: &str) {
        let css = fs::read_to_string(css_path).unwrap();
        let ast = biome_css_parser::parse_css(&css, biome_css_parser::CssParserOptions::default());
        for rule in ast.tree().rules() {
            let rule = rule.as_css_qualified_rule().unwrap();
            for selector in rule.prelude().into_iter() {
                let selector = selector.unwrap().to_selector(None);
                let block = rule.block().unwrap();
                let block = block.as_css_declaration_or_rule_block().unwrap();
                self.insert_empty(&selector);
                self.load_rule(selector, block);
            }
        }
    }

    pub fn serialize(&self) -> String {
        let rule = match &self.rule {
            Some(Rule {
                properties,
                selector,
            }) => {
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

    fn super_paths_of_aux(
        &self,
        path: &[String],
        is_root: bool,
        super_paths: &mut Vec<Vec<String>>,
    ) {
        if !is_root {
            if let Some(path) = self
                .get(path)
                .and_then(|n| n.rule.as_ref().map(|r| r.selector.path.clone()))
            {
                super_paths.push(path)
            }
        }
        for (_, t) in &self.children {
            t.super_paths_of_aux(path, false, super_paths);
        }
    }
    pub fn super_paths_of(&self, path: &[String]) -> Vec<Vec<String>> {
        let mut super_paths: Vec<Vec<String>> = vec![];
        self.super_paths_of_aux(path, true, &mut super_paths);
        super_paths
    }

    fn all_selectors_with_properties_aux(&self, selectors: &mut Vec<String>) {
        if let Some(rule) = self.rule.as_ref() {
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
        if let Some(rule) = self.rule.as_mut() {
            rule.properties.drain(0..);
        }
    }

    fn inheritable_properties(&self) -> HashMap<String, (String, Rc<Property>)> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| p.state == State::Valid)
                .filter(|p| properties::INHERITABLE_PROPERTIES.contains(&p.name().as_str()))
                .map(|p| (p.name(), (rule.selector.string.clone(), p.clone())))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        }
    }

    fn inherited_properties_for_aux(
        &self,
        path: &[String],
        inhertied_properties: &mut HashMap<String, (String, Rc<Property>)>,
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

    pub fn is_root(&self) -> bool {
        self.rule
            .as_ref()
            .map(|rule| rule.selector.string == ":root")
            .unwrap_or(false)
    }

    pub fn inherited_properties_for(
        &self,
        path: &[String],
    ) -> HashMap<String, (String, Rc<Property>)> {
        let tree = self.get(path).unwrap();
        let mut properties: HashMap<String, (String, Rc<Property>)> = HashMap::new();
        if !tree.is_root() {
            self.get_root()
                .inspect(|tree| properties.extend(tree.inheritable_properties()));
        }
        self.inherited_properties_for_aux(path, &mut properties);
        for super_path in self.super_paths_of(path) {
            properties.extend(self.get(&super_path).unwrap().inheritable_properties());
        }

        for property in tree.valid_properties() {
            if property.value() != "inherit" {
                properties.remove(&property.name());
            }
        }

        properties
    }

    fn valid_vars_with_selector_str(&self) -> HashMap<String, (String, Rc<Property>)> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| p.is_var() && p.state == State::Valid)
                .map(|p| (p.name(), (rule.selector.string.clone(), p.clone())))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        }
    }

    fn valid_var_lookup_ids(&self) -> Vec<String> {
        if let Some(rule) = &self.rule {
            // here lies the most beautiful code anyone has every seen /s
            rule.properties.iter().filter_map(|p| p.var_ref()).collect()
        } else {
            vec![]
        }
    }

    fn valid_properties(&self) -> Vec<Property> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| !p.is_var() && p.state == State::Valid)
                .map(|p| p.as_ref().clone())
                .collect()
        } else {
            vec![]
        }
    }

    fn inherited_vars_for_aux(
        &self,
        path: &[String],
        inherited_vars: &mut HashMap<String, (String, Rc<Property>)>,
    ) {
        let inherited_vars_from_self = self.valid_vars_with_selector_str();
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
        inherited_properties: &HashMap<String, (String, Rc<Property>)>,
    ) -> HashMap<String, (String, Rc<Property>)> {
        let tree = self.get(path).unwrap();
        let mut vars: HashMap<String, (String, Rc<Property>)> = HashMap::new();
        if !tree.is_root() {
            self.get_root()
                .inspect(|tree| vars.extend(tree.valid_vars_with_selector_str()));
        }
        self.inherited_vars_for_aux(path, &mut vars);
        for super_path in self.super_paths_of(path) {
            vars.extend(
                self.get(&super_path)
                    .unwrap()
                    .valid_vars_with_selector_str(),
            );
        }
        for name in tree.valid_vars_with_selector_str().keys() {
            vars.remove(name);
        }
        let var_references_in_rule = tree.valid_var_lookup_ids();
        let var_references_in_inherited_properties: Vec<String> = inherited_properties
            .iter()
            .filter_map(|(_, (_, p))| p.var_ref())
            .collect();

        vars.retain(|key, _| {
            var_references_in_rule.contains(key)
                || var_references_in_inherited_properties.contains(key)
        });
        vars
    }

    pub fn set_state(
        &mut self,
        path: &[String],
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
        rule.comment_all_with_name(property_name);
        if state == State::Valid {
            rule.insert(Property {
                node: parse_property(&format!("{}: {};", property_name, property_value)).unwrap(),
                state,
            });
        }
    }

    pub fn delete(&mut self, path: &[String], property_name: &str, property_value: &str) {
        let tree = self.get_mut(path).unwrap();
        assert!(
            tree.rule.is_some(),
            "can't delete property from rule that doesn't exist"
        );
        let rule = tree.rule.as_mut().unwrap();
        rule.properties
            .retain(|p| !(&p.name() == property_name && &p.value() == property_value));
    }

    fn insert_raw(&mut self, selector: Selector, path: &[String], property: Property) {
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
                Some(tree) => tree.insert_raw(selector, parts, property),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert_raw(selector, parts, property);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn insert_commented(&mut self, selector: &Selector, property: CssDeclarationWithSemicolon) {
        self.insert_raw(
            selector.clone(),
            &selector.path,
            Property {
                node: property,
                state: State::Commented,
            },
        )
    }

    fn insert_empty_aux(&mut self, selector: Selector, path: &[String]) {
        match path {
            [] => {
                match &mut self.rule {
                    Some(_) => {} // already exists
                    None => self.rule = Some(Rule::new(selector)),
                };
            }
            [part, parts @ ..] => match self.children.get_mut(part) {
                Some(tree) => tree.insert_empty_aux(selector, parts),
                None => {
                    let mut new_tree = CSSDB::new();
                    new_tree.insert_empty_aux(selector, parts);
                    self.children.insert(part.to_owned(), new_tree);
                }
            },
        }
    }

    pub fn insert_empty(&mut self, selector: &Selector) {
        self.insert_empty_aux(selector.clone(), &selector.path);
    }

    pub fn insert(&mut self, selector: &Selector, property: &CssDeclarationWithSemicolon) {
        self.insert_raw(
            selector.clone(),
            &selector.path,
            Property {
                node: property.clone(),
                state: State::Valid,
            },
        )
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

impl Storage for biome_css_syntax::AnyCssSimpleSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            biome_css_syntax::AnyCssSimpleSelector::CssTypeSelector(t) => {
                vec![t
                    .ident()
                    .unwrap()
                    .value_token()
                    .unwrap()
                    .text_trimmed()
                    .to_string()]
            }
            biome_css_syntax::AnyCssSimpleSelector::CssUniversalSelector(_) => todo!(),
        }
    }
}

impl Storage for biome_css_syntax::CssCompoundSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        [
            self.simple_selector()
                .map(|s| s.to_css_db_path())
                .unwrap_or(vec![]),
            self.sub_selectors()
                .into_iter()
                .flat_map(|selector| selector.to_css_db_path())
                .collect(),
        ]
        .concat()
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

impl Storage for CssRelativeSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        assert!(self.combinator().is_none());
        self.selector().unwrap().to_css_db_path()
    }
}

impl Storage for AnyCssRelativeSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => todo!(),
            AnyCssRelativeSelector::CssRelativeSelector(selector) => selector.to_css_db_path(),
        }
    }
}

impl Storage for AnyCssSubSelector {
    fn to_css_db_path(&self) -> Vec<String> {
        match self {
            CssAttributeSelector(attribute_selector) => attribute_selector.to_css_db_path(),
            CssBogusSubSelector(_) => vec![],
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
