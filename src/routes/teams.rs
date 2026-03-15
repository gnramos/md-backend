use axum::{Router, routing::get};

use crate::{AppState, handlers};

pub fn router() -> Router<AppState> {
    Router::new().route("/teams/options", get(handlers::institutions::get_options))
}
