use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: i32,
    pub competition_id: i32,
    pub location_id: i32,
    pub name: String,
    pub level: i32,
    pub date: NaiveDate,
}
