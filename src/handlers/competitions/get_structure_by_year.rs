use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};

use crate::{
    AppState,
    dtos::competitions::input::{CompetitionPath, CompetitionStructureQuery},
    services,
};

pub async fn get_structure_by_year(
    State(state): State<AppState>,
    Path(path): Path<CompetitionPath>,
    Query(query): Query<CompetitionStructureQuery>,
) -> impl IntoResponse {
    services::competitions::get_structure_by_year(&state.repo, path.id, query.year)
        .await
        .map(|structure| Json(structure))
}
