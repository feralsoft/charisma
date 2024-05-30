use html::Render;

mod html;

fn main() {
    let result = biome_css_parser::parse_css(
        ".test { --color: 20%; }",
        biome_css_parser::CssParserOptions::default(),
    );
    let output = result.tree().render_html();

    let expected_output = "<div data-kind=\"rule\"><div data-attr=\"selector\">".to_owned()
        + "<div data-kind=\"class\">"
        + &html::render_value("test".to_string())
        + "</div>"
        + "</div>"
        + "<div data-attr=\"properties\">"
        + "<div data-kind=\"property\">"
        + "<div data-attr=\"name\">"
        + &html::render_value("--color".to_owned())
        + "</div>"
        + "<div data-attr=\"value\">"
        + "<div data-kind=\"unit\" data-unit-type=\"percentage\">"
        + &html::render_value("20".to_owned())
        + "</div>"
        + "</div>"
        + "</div>"
        + "</div>"
        + "</div>";
    // println!("{}", expected_output);

    assert!(expected_output == output)
}
