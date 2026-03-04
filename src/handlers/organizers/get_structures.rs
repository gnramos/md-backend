use axum::{Json, extract::{Query, State}, response::IntoResponse};

use crate::{AppState, dtos::filters::input::CompetitionOptionsQuery, services};

pub async fn get_structures(
    State(state): State<AppState>,
    Query(filter): Query<CompetitionOptionsQuery>
) -> impl IntoResponse {
    services::organizers::get_structures(&state.repo, filter.organizer_ids)
        .await
        .map(|structures| Json(structures))
}