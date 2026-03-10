use crate::shared::types::LocationType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Location {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub location_type: LocationType,
    pub name: String,
}
