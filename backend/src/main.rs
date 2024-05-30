fn main() {
    let result = biome_css_parser::parse_css(
        ".btn { font-size: 20px; }",
        biome_css_parser::CssParserOptions::default(),
    );

    // let base_rule = result.tree().rules().into_iter().next().unwrap();

    // let btn_selector = base_rule
    //     .as_css_qualified_rule()
    //     .unwrap()
    //     .prelude()
    //     .into_iter()
    //     .next()
    //     .unwrap()
    //     .unwrap()
    //     .as_css_compound_selector()
    //     .unwrap()
    //     .as_fields()
    //     .sub_selectors;

    // println!("{:?}", base_rule)
}
