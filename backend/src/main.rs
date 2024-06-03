#[macro_use]
extern crate rocket;

use html::Render;
use rocket::{http::ContentType, tokio::fs::read_to_string};
use sha1::Digest;
use storage::Storage;

mod html;
mod storage;

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
    let rule = biome_css_parser::parse_css(
        ".btn { font-size: 20px; }",
        biome_css_parser::CssParserOptions::default(),
    )
    .tree()
    .rules()
    .into_iter()
    .next()
    .unwrap();
    let path = rule
        .as_css_qualified_rule()
        .unwrap()
        .prelude()
        .into_iter()
        .next()
        .unwrap()
        .unwrap()
        .to_path();
    let mut hasher = sha1::Sha1::new();
    hasher.update(b".btn");
    let btn = hasher.finalize();

    assert!(path == vec![format!("{:X}", btn)]);

    rocket::build().mount("/", routes![index])
}
