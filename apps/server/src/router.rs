use crate::handlers::{hello, home};
use axum::Router;
use axum::routing::get;

pub fn build_app() -> Router {
    Router::new().route("/", get(home)).merge(hello::routes())
}
