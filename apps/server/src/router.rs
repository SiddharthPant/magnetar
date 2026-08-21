use crate::handlers::{hello, root};
use axum::Router;

pub fn build_app() -> Router {
    root::routes().nest("/hello", hello::routes())
}
