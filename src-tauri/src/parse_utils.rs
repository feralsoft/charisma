use biome_css_syntax::{CssDeclarationWithSemicolon, CssSelectorList};

pub fn parse_selector(str: &str) -> Option<CssSelectorList> {
    let rule = biome_css_parser::parse_css(
        format!("{} {{}}", str).as_str(),
        biome_css_parser::CssParserOptions::default(),
    )
    .tree()
    .rules()
    .into_iter()
    .next()?;

    Some(rule.as_css_qualified_rule()?.prelude())
}

pub fn parse_one(rule: &str) -> Option<biome_css_syntax::CssQualifiedRule> {
    let rules = biome_css_parser::parse_css(rule, biome_css_parser::CssParserOptions::default())
        .tree()
        .rules();
    if (&rules).into_iter().count() != 1 {
        return None;
    }
    let rule = rules.into_iter().next()?;

    Some(rule.as_css_qualified_rule()?.to_owned())
}

pub fn parse_property(property_str: &str) -> Option<CssDeclarationWithSemicolon> {
    let dummy_rule = parse_one(&format!(".a {{ {} }}", property_str))?;
    let block = dummy_rule.block().ok()?;
    let block = block.as_css_declaration_or_rule_block()?;
    assert!(block.items().into_iter().len() == 1);
    let item = block.items().into_iter().next()?;
    item.as_css_declaration_with_semicolon()
        .map(|item| item.to_owned())
}
