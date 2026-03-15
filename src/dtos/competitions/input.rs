use serde::Deserialize;

use crate::shared::serde::CsvOptVec;

// For the main competitions endpoint: "/competitions/structures"
#[derive(Deserialize)]
pub struct CompetitionStructuresQuery {
    pub competition_ids: CsvOptVec<i32>,
}

// For any endpoint that searches for data of a single competition by its id
#[derive(Deserialize)]
pub struct CompetitionPath {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct CompetitionStructureQuery {
    pub year: Option<i32>,
}
