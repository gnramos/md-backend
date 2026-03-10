use crate::shared::types::Status;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub id: i32,
    pub status: Status,
    pub language: String,
    pub code: String,
    pub submission_time: NaiveDateTime,
    pub team_event_id: i32,
    pub problem_id: i32,
}
