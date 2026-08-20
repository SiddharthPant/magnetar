use askama::Template;
use axum::{Router, extract, response::IntoResponse, routing::get};

use crate::response::HtmlTemplate;

pub fn routes() -> Router {
    Router::new().route("/hello/{name}", get(hello))
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

async fn hello(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    HtmlTemplate(template)
}
