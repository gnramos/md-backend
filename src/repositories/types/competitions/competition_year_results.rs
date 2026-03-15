use chrono::NaiveDate;
use sqlx::prelude::FromRow;

use crate::shared::types::LocationType;

#[derive(FromRow)]
pub struct CompetitionYearResultRow {
    pub competition_total_institutions: i32,
    pub competition_total_teams: i32,
    pub competition_total_participants: i32,
    pub competition_female_participants: i32,
    pub competition_location_types: Vec<LocationType>,

    pub event_id: i32,
    pub event_name: String,
    pub event_level: Option<i32>,
    pub event_date: NaiveDate,
    pub event_location: String,
    pub event_total_institutions: i32,
    pub event_total_teams: i32,
    pub event_total_participants: i32,
    pub event_female_participants: i32,
    pub event_location_types: Vec<LocationType>,

    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub institution_location: String,

    pub team_id: i32,
    pub team_name: String,
    pub team_rank: i32,
    pub team_total_members: i32,
    pub team_female_members: i32,
}
