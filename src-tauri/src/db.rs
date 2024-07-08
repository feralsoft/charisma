use crate::{
    parse_utils::{get_combinator_type, parse_property},
    properties,
};
use std::{collections::HashMap, fs, ops, rc::Rc};

use biome_css_syntax::{
    AnyCssPseudoClass, AnyCssPseudoElement, AnyCssRelativeSelector,
    AnyCssSelector::{self, *},
    AnyCssSubSelector::{self, *},
    CssAttributeSelector, CssDeclarationOrRuleBlock, CssDeclarationWithSemicolon,
    CssPseudoClassFunctionRelativeSelectorList, CssRelativeSelector,
};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Valid,
    Commented,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Specificity {
    // # of ids
    a: u64,
    // # of classes, attributes and pseudo-classes
    b: u64,
    // # of elements and pseudo-elements
    c: u64,
}

impl ops::Add<Specificity> for Specificity {
    type Output = Specificity;

    fn add(self, rhs: Specificity) -> Self::Output {
        Specificity::new(self.a + rhs.a, self.b + rhs.b, self.c + rhs.c)
    }
}

impl Specificity {
    pub fn new(a: u64, b: u64, c: u64) -> Self {
        Specificity { a, b, c }
    }

    // SPECIFICITY ALGORITHM FROM SPEC
    // count the number of ID selectors in the selector (= A)
    // count the number of class selectors, attributes selectors, and pseudo-classes in the selector (= B)
    // count the number of type selectors and pseudo-elements in the selector (= C)
    // ignore the universal selector (*)
    //
    // when calculating which selector wins, match components 1 by 1,
    // for eg.
    //   #my-id           => (1, 0, 0)
    //   .card:has(.name) => (0, 2, 0)
    //
    // #my-id will win!
    fn from_part(part: &String) -> Self {
        if part.starts_with("#") {
            Self::new(1, 0, 0)
        } else if part.starts_with(".") {
            Self::new(0, 1, 0)
        } else if part.starts_with("::") {
            Self::new(0, 0, 1)
        } else if part.starts_with(":") {
            Self::new(0, 1, 0)
        } else if part.chars().all(|c| c.is_alphabetic() || c == '-') {
            // element
            Self::new(0, 0, 1)
        } else {
            panic!("no specificity for {:?}", part)
        }
    }

    pub fn from_path(path: &[String]) -> Self {
        let mut specificity = Self::new(0, 0, 0);
        let mut iter = path.iter();

        // go until we hit ":has(" (or ":is(", ":not(")
        loop {
            match iter.next() {
                Some(part) => {
                    if part.contains("(") {
                        break;
                    }
                    if ![">", " ", "~", "+"].contains(&part.as_str()) {
                        specificity = specificity + Specificity::from_part(part)
                    }
                }
                None => return specificity,
            }
        }

        // check out the sub path until we get to ")"
        loop {
            match iter.next() {
                Some(part) => {
                    if part.as_str() == ")" {
                        break;
                    }
                    if ![">", " ", "~", "+"].contains(&part.as_str()) {
                        specificity = specificity + Specificity::from_part(part)
                    }
                }
                None => return specificity,
            }
        }

        match iter.cloned().collect::<Vec<_>>().as_slice() {
            [] => specificity,
            rest => specificity + Specificity::from_path(&rest),
        }
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.a
                .cmp(&other.a)
                .then_with(|| self.b.cmp(&other.b))
                .then_with(|| self.c.cmp(&other.c)),
        )
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
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
    pub specificity: Specificity,
}

pub trait ToSelectors {
    fn to_selectors(&self, parent: Option<&Selector>) -> Vec<Selector>;
}

impl ToSelectors for AnyCssRelativeSelector {
    fn to_selectors(&self, parent: Option<&Selector>) -> Vec<Selector> {
        let selector = self.as_css_relative_selector().unwrap();
        let selector = selector.selector().unwrap();
        let selector = selector.as_css_compound_selector().unwrap();
        assert!(selector.simple_selector().is_none());
        let separator = match selector.nesting_selector_token() {
            Some(combinator) => get_combinator_type(combinator.kind()),
            None => " ".to_string(),
        };

        selector
            .to_css_db_paths()
            .iter()
            .map(|path| {
                let path = [
                    parent.map(|p| p.path.clone()).unwrap_or(vec![]),
                    vec![separator.clone()],
                    path.clone(),
                ]
                .concat();
                Selector {
                    string: parent
                        .as_ref()
                        .map(|p| p.string.clone())
                        .unwrap_or("".to_string())
                        + &separator
                        + selector.to_string().trim(),
                    specificity: Specificity::from_path(&path),
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
                    string: parent
                        .as_ref()
                        .map(|p| p.string.clone())
                        .unwrap_or("".to_string())
                        + &path.join(""),
                    specificity: Specificity::from_path(&path),
                    path,
                }
            })
            .collect()
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
                        for selector in child.to_selectors(Some(&selector)) {
                            self.load_rule(selector, block);
                        }
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
                let block = rule.block().unwrap();
                let block = block.as_css_declaration_or_rule_block().unwrap();
                for selector in selector.unwrap().to_selectors(None) {
                    self.insert_empty(&selector);
                    self.load_rule(selector, block);
                }
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
        current_part_name: &str,
        super_paths: &mut Vec<Vec<String>>,
    ) {
        if !is_root {
            if let Some(path) = self
                .get(path)
                .and_then(|n| n.rule.as_ref().map(|r| r.selector.path.clone()))
            {
                super_paths.push(path)
            } else if current_part_name == ":has(" {
                if let Some(path) = self
                    .get(&[path, &[")".to_string()]].concat())
                    .and_then(|n| n.rule.as_ref().map(|r| r.selector.path.clone()))
                {
                    // body:has(button.active) is not getting detected as a superpath of
                    // button.active
                    // since given the path ["body", ":has(", "button", "active", ")"]
                    // when I'm at ["body", ":has"] & I get ["button", "active"]
                    // there is no rule there since we need to go to the ")"
                    super_paths.push(path)
                }
            }
        }
        for (name, t) in &self.children {
            t.super_paths_of_aux(path, false, &name, super_paths);
        }
    }

    // a super path is a path which contains the searched path
    pub fn super_paths_of(&self, path: &[String]) -> Vec<Vec<String>> {
        let mut super_paths: Vec<Vec<String>> = vec![];
        self.super_paths_of_aux(path, true, "", &mut super_paths);
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

    fn inheritable_properties(&self) -> HashMap<String, (Selector, Rc<Property>)> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| p.state == State::Valid)
                .filter(|p| properties::INHERITABLE_PROPERTIES.contains(&p.name().as_str()))
                .map(|p| (p.name(), (rule.selector.clone(), p.clone())))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        }
    }

    fn inherited_properties_for_aux(
        &self,
        path: &[String],
        inhertied_properties: &mut HashMap<String, (Selector, Rc<Property>)>,
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
    ) -> HashMap<String, (Selector, Rc<Property>)> {
        let tree = self.get(path).unwrap();
        let mut properties: HashMap<String, (Selector, Rc<Property>)> = HashMap::new();
        if !tree.is_root() {
            self.get_root()
                .inspect(|tree| properties.extend(tree.inheritable_properties()));
        }
        // go directly do my current path
        // eg. `body table td` ->
        // 1 - go to `body` & get all inherited properties
        // 2 - go to `body table` & get inherited properties (overwrite from `body`)
        self.inherited_properties_for_aux(path, &mut properties);
        // now we are going up super paths, in the case we are overwriting
        // a property, we need to compare specifcity to understand which property will rule
        for super_path in self.super_paths_of(path) {
            for (property_name, (super_path_selector, super_path_property)) in
                self.get(&super_path).unwrap().inheritable_properties()
            {
                if let Some((selector, _)) = properties.get(&property_name) {
                    if selector.specificity < super_path_selector.specificity {
                        properties
                            .insert(property_name, (super_path_selector, super_path_property));
                    }
                } else {
                    properties.insert(property_name, (super_path_selector, super_path_property));
                }
            }
        }

        for property in tree.valid_properties() {
            // wtf is this about?
            if property.value() != "inherit" {
                properties.remove(&property.name());
            }
        }

        properties
    }

    fn valid_vars_with_selector(&self) -> HashMap<String, (Selector, Rc<Property>)> {
        if let Some(rule) = &self.rule {
            rule.properties
                .iter()
                .filter(|p| p.is_var() && p.state == State::Valid)
                .map(|p| (p.name(), (rule.selector.clone(), p.clone())))
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
        inherited_vars: &mut HashMap<String, (Selector, Rc<Property>)>,
    ) {
        let inherited_vars_from_self = self.valid_vars_with_selector();
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
        inherited_properties: &HashMap<String, (Selector, Rc<Property>)>,
    ) -> HashMap<String, (Selector, Rc<Property>)> {
        let tree = self.get(path).unwrap();
        let mut vars: HashMap<String, (Selector, Rc<Property>)> = HashMap::new();
        if !tree.is_root() {
            self.get_root()
                .inspect(|tree| vars.extend(tree.valid_vars_with_selector()));
        }
        self.inherited_vars_for_aux(path, &mut vars);
        for super_path in self.super_paths_of(path) {
            for (var_name, (super_path_selector, super_path_property)) in
                self.get(&super_path).unwrap().valid_vars_with_selector()
            {
                if let Some((selector, _)) = vars.get(&var_name) {
                    if selector.specificity < super_path_selector.specificity {
                        vars.insert(var_name, (super_path_selector, super_path_property));
                    }
                } else {
                    vars.insert(var_name, (super_path_selector, super_path_property));
                }
            }
        }

        for name in tree.valid_vars_with_selector().keys() {
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
    fn to_css_db_paths(&self) -> Vec<Vec<String>>;
}

impl Storage for biome_css_syntax::AnyCssSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self {
            CssBogusSelector(_) => todo!(),
            CssComplexSelector(s) => {
                let fields = s.as_fields();
                let left = fields.left.unwrap();
                let right = fields.right.unwrap();
                let rhs_paths = right.to_css_db_paths();

                left.to_css_db_paths()
                    .iter()
                    .flat_map(|lhs| {
                        rhs_paths
                            .iter()
                            .map(|rhs| [lhs.clone(), vec![String::from(" ")], rhs.clone()].concat())
                    })
                    .collect()
            }
            CssCompoundSelector(selector) => selector.to_css_db_paths(),
        }
    }
}

impl Storage for biome_css_syntax::AnyCssSimpleSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self {
            biome_css_syntax::AnyCssSimpleSelector::CssTypeSelector(t) => {
                vec![vec![t
                    .ident()
                    .unwrap()
                    .value_token()
                    .unwrap()
                    .text_trimmed()
                    .to_string()]]
            }
            biome_css_syntax::AnyCssSimpleSelector::CssUniversalSelector(_) => todo!(),
        }
    }
}

impl Storage for biome_css_syntax::CssCompoundSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self.simple_selector() {
            Some(lhs) => {
                let lhs_paths = lhs.to_css_db_paths();

                if self.sub_selectors().into_iter().count() == 0 {
                    return lhs_paths;
                }

                self.sub_selectors()
                    .into_iter()
                    .flat_map(|selector| {
                        selector
                            .to_css_db_paths()
                            .iter()
                            .flat_map(|path| {
                                lhs_paths
                                    .iter()
                                    .map(|lhs| [lhs.clone(), path.clone()].concat())
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            }
            None => {
                let paths: Vec<_> = self
                    .sub_selectors()
                    .into_iter()
                    .flat_map(|selector| selector.to_css_db_paths())
                    .fold::<Vec<Vec<String>>, _>(vec![], |acc_paths, cur_path| {
                        // this breaks my mind, but it is appearing to work :sweat_smile:
                        if acc_paths.is_empty() {
                            vec![cur_path]
                        } else {
                            acc_paths
                                .iter()
                                .map(|lhs| [lhs.clone(), cur_path.clone()].concat())
                                .collect()
                        }
                    });

                paths
            }
        }
    }
}

impl Storage for CssPseudoClassFunctionRelativeSelectorList {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        let name = self.name_token().unwrap();
        let relative_selectors = self.relative_selectors();

        let path_of_paths: Vec<Vec<Vec<String>>> = relative_selectors
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

        path_of_paths
            .iter()
            .map(|paths| {
                // this will break when you have :has(:is(a, b))
                assert!(paths.len() == 1);
                let path = paths.first().unwrap();

                [
                    vec![format!(":{}(", name.text_trimmed())],
                    path.clone(),
                    vec![String::from(")")],
                ]
                .concat()
            })
            .collect()
    }
}

impl Storage for AnyCssPseudoClass {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self {
            AnyCssPseudoClass::CssBogusPseudoClass(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelector(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionCompoundSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionIdentifier(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionNth(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionRelativeSelectorList(s) => s.to_css_db_paths(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelector(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionSelectorList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassFunctionValueList(_) => todo!(),
            AnyCssPseudoClass::CssPseudoClassIdentifier(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![format!(":{}", name)]]
            }
        }
    }
}

impl Storage for CssAttributeSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        let name = self.name().unwrap();
        match self.matcher() {
            Some(matcher) => {
                assert!(matcher.modifier().is_none());
                let operator = matcher.operator().unwrap();
                let value = matcher.value().unwrap();

                // [data-kind="rule"] -> ['[data-kind]', '[data-kind="rule"]']
                // so that you can explore siblings along [data-kind]
                vec![vec![
                    format!("[{}]", name),
                    format!("[{}{}{}]", name, operator, value),
                ]]
            }
            None => {
                vec![vec![format!("[{}]", name)]]
            }
        }
    }
}

impl Storage for AnyCssPseudoElement {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self {
            AnyCssPseudoElement::CssBogusPseudoElement(_) => todo!(),
            AnyCssPseudoElement::CssPseudoElementFunctionIdentifier(_) => todo!(),
            AnyCssPseudoElement::CssPseudoElementFunctionSelector(_) => todo!(),
            AnyCssPseudoElement::CssPseudoElementIdentifier(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                vec![vec![format!("::{}", name.text_trimmed())]]
            }
        }
    }
}

impl Storage for CssRelativeSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        assert!(self.combinator().is_none());
        self.selector().unwrap().to_css_db_paths()
    }
}

impl Storage for AnyCssRelativeSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self {
            AnyCssRelativeSelector::CssBogusSelector(_) => todo!(),
            AnyCssRelativeSelector::CssRelativeSelector(selector) => selector.to_css_db_paths(),
        }
    }
}

impl Storage for AnyCssSubSelector {
    fn to_css_db_paths(&self) -> Vec<Vec<String>> {
        match self {
            CssAttributeSelector(attribute_selector) => attribute_selector.to_css_db_paths(),
            CssBogusSubSelector(_) => vec![],
            CssClassSelector(class) => {
                let name = class.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![format!(".{}", name)]]
            }
            CssIdSelector(id) => {
                let name = id.name().unwrap().value_token().unwrap();
                let name = name.text_trimmed();
                vec![vec![format!("#{}", name)]]
            }
            CssPseudoClassSelector(pseudo_class) => pseudo_class.class().unwrap().to_css_db_paths(),
            CssPseudoElementSelector(pseudo_element) => {
                pseudo_element.element().unwrap().to_css_db_paths()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse_utils::parse_selector;

    use super::*;

    fn selectors(str: &str) -> Vec<Selector> {
        let selector_list = parse_selector(str).unwrap();

        selector_list
            .into_iter()
            .flat_map(|s| s.unwrap().to_selectors(None))
            .collect()
    }

    fn one_selector(str: &str) -> Selector {
        let selectors = selectors(str);
        assert!(selectors.len() == 1);
        selectors.first().unwrap().clone()
    }

    #[test]
    fn id_has_greater_specificity_than_class() {
        let id_spec = Specificity::new(1, 0, 0); // #name
        let class_spec = Specificity::new(0, 2, 0); // .name.active
        assert!(id_spec > class_spec)
    }

    #[test]
    fn or_to_selectors() {
        let selectors = selectors("div, input");
        assert_eq!(selectors.len(), 2);
    }

    #[test]
    fn element_specificity() {
        let selectors = selectors("div, input");

        let div = selectors.iter().find(|s| s.string.contains("div")).unwrap();
        let input = selectors
            .iter()
            .find(|s| s.string.contains("input"))
            .unwrap();

        assert_eq!(
            Specificity::from_path(&div.path),
            Specificity::from_path(&input.path)
        );
    }

    #[test]
    fn class_specificity() {
        let selectors = selectors(".name, input");

        let name = selectors
            .iter()
            .find(|s| s.string.contains(".name"))
            .unwrap();
        let input = selectors
            .iter()
            .find(|s| s.string.contains("input"))
            .unwrap();

        assert!(Specificity::from_path(&name.path) > Specificity::from_path(&input.path));
    }

    #[test]
    fn id_specificity() {
        let selectors = selectors("#name, .input");

        let name = selectors
            .iter()
            .find(|s| s.string.contains("#name"))
            .unwrap();
        let input = selectors
            .iter()
            .find(|s| s.string.contains(".input"))
            .unwrap();

        assert!(Specificity::from_path(&name.path) > Specificity::from_path(&input.path));
    }

    #[test]
    fn complex_compare() {
        let selector = one_selector(".name:has(.you)"); // (0, 2, 0)

        assert_eq!(
            Specificity::from_path(&selector.path),
            Specificity { a: 0, b: 2, c: 0 }
        );
    }

    #[test]
    fn complex_compare_with_id() {
        let selector = one_selector("#name:has(.you)"); // (1, 1, 0)

        assert_eq!(
            Specificity::from_path(&selector.path),
            Specificity { a: 1, b: 1, c: 0 }
        );
    }

    #[test]
    fn complex_compare_with_id_against_class() {
        let s1 = one_selector("#name:has(.you)"); // (1, 1, 0)
        let s2 = one_selector(".name:has(.you)"); // (0, 2, 0)

        assert!(Specificity::from_path(&s1.path) > Specificity::from_path(&s2.path));
    }
}
