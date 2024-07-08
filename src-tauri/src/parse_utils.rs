use biome_css_syntax::{CssDeclarationWithSemicolon, CssSelectorList, CssSyntaxKind};

pub fn parse_selector(str: &str) -> Option<CssSelectorList> {
    // eh heck, `url::form_urlencoded::byte_serialize` encodes ' ' as '+'
    // this is gonna get real fucked when we get sibling selectors..
    assert!(!str.contains("+"));

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

fn parse_one(rule: String) -> Option<biome_css_syntax::CssQualifiedRule> {
    let rules = biome_css_parser::parse_css(&rule, biome_css_parser::CssParserOptions::default())
        .tree()
        .rules();
    assert!((&rules).into_iter().len() == 1);
    let rule = rules.into_iter().next()?;

    Some(rule.as_css_qualified_rule()?.to_owned())
}

pub fn parse_property(property_str: &str) -> Option<CssDeclarationWithSemicolon> {
    let dummy_rule = parse_one(format!(".a {{ {} }}", property_str))?;
    let block = dummy_rule.block().ok()?;
    let block = block.as_css_declaration_or_rule_block()?;
    assert!(block.items().into_iter().len() == 1);
    let item = block.items().into_iter().next()?;
    item.as_css_declaration_with_semicolon()
        .map(|item| item.to_owned())
}

pub fn get_combinator_type(token_kind: CssSyntaxKind) -> String {
    match token_kind {
        CssSyntaxKind::CSS_SPACE_LITERAL => "descendent".to_string(),
        CssSyntaxKind::R_ANGLE => "direct-descendant".to_string(),
        _ => todo!(),
    }
}
