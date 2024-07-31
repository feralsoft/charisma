use biome_css_syntax::{CssDeclarationWithSemicolon, CssSelectorList};

use crate::CharismaError;

pub fn parse_selector(str: &str) -> Result<CssSelectorList, CharismaError> {
    let rule = biome_css_parser::parse_css(
        format!("{} {{}}", str).as_str(),
        biome_css_parser::CssParserOptions::default(),
    )
    .tree()
    .rules()
    .into_iter()
    .next();

    let rule = match rule.and_then(|r| r.as_css_qualified_rule().cloned()) {
        Some(r) => r,
        None => return Err(CharismaError::ParseError("invalid selector".into())),
    };

    Ok(rule.prelude())
}

pub fn parse_one(rule: &str) -> Result<biome_css_syntax::CssQualifiedRule, CharismaError> {
    let rules = biome_css_parser::parse_css(rule, biome_css_parser::CssParserOptions::default())
        .tree()
        .rules();
    if (&rules).into_iter().count() != 1 {
        return Err(CharismaError::ParseError("no rule found".into()));
    }
    let rule = match rules.into_iter().next() {
        Some(r) => r,
        None => {
            return Err(CharismaError::ParseError("no rule found".into()));
        }
    };

    match rule.as_css_qualified_rule() {
        Some(r) => Ok(r.clone()),
        None => Err(CharismaError::ParseError("invalid rule".into())),
    }
}

pub fn parse_property(property_str: &str) -> Result<CssDeclarationWithSemicolon, CharismaError> {
    let dummy_rule = parse_one(&format!(".a {{ {} }}", property_str))?;
    let block = dummy_rule
        .block()
        .map_err(|e| CharismaError::ParseError(e.to_string()))?;
    let block = match block.as_css_declaration_or_rule_block() {
        Some(b) => b,
        None => return Err(CharismaError::ParseError("not a valid rule".into())),
    };
    assert!(block.items().into_iter().len() == 1);
    let item = match block.items().into_iter().next() {
        Some(item) => item,
        None => return Err(CharismaError::ParseError("no property found".into())),
    };
    match item.as_css_declaration_with_semicolon() {
        Some(item) => Ok(item.clone()),
        None => Err(CharismaError::ParseError("not valid decl".into())),
    }
}
