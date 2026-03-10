use crate::shared::types::GenderCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Competition {
    pub id: i32,
    pub organizer_id: i32,
    pub name: String,
    pub gender_category: GenderCategory,
    pub website_url: Option<String>,
}
