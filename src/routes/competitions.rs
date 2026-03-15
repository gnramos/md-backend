use axum::{Router, routing::get};

use crate::{AppState, handlers};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/competitions/options",
            get(handlers::competitions::get_options),
        )
        .route(
            "/competitions/structures",
            get(handlers::competitions::get_structures),
        )
        .route(
            "/competitions/{id}/structure",
            get(handlers::competitions::get_structure_by_year),
        )
    /*
    .route("/competitions/{id}/results", get(todo!))
    */
}
