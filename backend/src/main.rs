#[macro_use]
extern crate rocket;
use html::Render;
use rocket::http::ContentType;

mod html;

#[get("/")]
fn index() -> (ContentType, String) {
    (
        ContentType::HTML,
        biome_css_parser::parse_css(
            ".btn { font-size: 20px; }",
            biome_css_parser::CssParserOptions::default(),
        )
        .tree()
        .render_html(),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
