use serde::Deserialize;

use crate::shared::serde::CsvOptVec;

#[derive(Deserialize)]
pub struct InstitutionStructuresQuery {
    pub institution_ids: CsvOptVec<i32>,
}
