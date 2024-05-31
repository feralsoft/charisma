#[macro_use]
extern crate rocket;

use html::Render;
use rocket::{http::ContentType, tokio::fs::read_to_string};

mod html;

async fn css() -> String {
    read_to_string("src/index.css").await.unwrap()
}

#[get("/")]
async fn index() -> (ContentType, String) {
    let code_html = biome_css_parser::parse_css(
        ".btn { font-size: 20px; }",
        biome_css_parser::CssParserOptions::default(),
    )
    .tree()
    .render_html();
    (
        ContentType::HTML,
        format!(
            "<style>{}</style>
            <div class=\"--editor\" spellcheck=\"false\">{}</div>",
            css().await,
            code_html
        ),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
