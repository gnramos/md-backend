use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::Serialize;

use crate::shared::types::{GenderCategory, LocationType};

// ======================== Response DTOs ========================
#[derive(Debug, Serialize)]
pub struct CompetitionStructure {
    pub id: i32,
    pub name: String,
    pub website_url: Option<String>,
    pub gender_category: GenderCategory,
    pub years: Vec<u32>,
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub events: Vec<EventSubStructure>,
}

#[derive(Debug, Serialize)]
pub struct EventSubStructure {
    pub id: i32,
    pub name: String,
    pub level: Option<u32>,
    pub date: NaiveDate,
    pub location: String,
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub teams: Vec<TeamSubStructure>,
}

#[derive(Debug, Serialize)]
pub struct TeamSubStructure {
    pub id: i32,
    pub name: String,
    pub rank: u32,
    pub institution_name: String,
    pub institution_short_name: Option<String>,
    pub institution_location: String,
    pub total_members: u32,
    pub female_percentage: f32,
}

// Used for updating competition stats and events by year in organizer page
#[derive(Default, Debug, Serialize)]
pub struct CompetitionYearStats {
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub events: Vec<crate::dtos::organizers::output::EventSubStructure>,
}

// Used for updating competition stats and events by year in competition page
#[derive(Debug, Serialize)]
pub struct CompetitionYearResults {
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub events: Vec<EventSubStructure>,
}

// ======================== Intermediate structures ========================
// Used while aggregating data (competition -> events -> teams)
// before converting to the final serializable payload.
#[derive(Debug)]
pub struct TempCompetitionStructure {
    pub id: i32,
    pub name: String,
    pub website_url: Option<String>,
    pub gender_category: GenderCategory,
    pub years: Vec<u32>,
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub events: IndexMap<i32, TempEventSubStructure>,
}

#[derive(Debug)]
pub struct TempEventSubStructure {
    pub id: i32,
    pub name: String,
    pub level: Option<u32>,
    pub date: NaiveDate,
    pub location: String,
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub teams: IndexMap<i32, TeamSubStructure>,
}

#[derive(Default, Debug)]
pub struct TempCompetitionYearResults {
    pub total_institutions: u32,
    pub total_teams: u32,
    pub total_participants: u32,
    pub female_percentage: f32,
    pub location_types: Vec<LocationType>,
    pub events: IndexMap<i32, TempEventSubStructure>,
}

// ======================== Conversion to final DTO ========================
impl From<TempCompetitionStructure> for CompetitionStructure {
    fn from(value: TempCompetitionStructure) -> Self {
        let mut location_types = value.location_types;
        location_types.sort();
        Self {
            id: value.id,
            name: value.name,
            website_url: value.website_url,
            gender_category: value.gender_category,
            years: value.years,
            total_institutions: value.total_institutions,
            total_teams: value.total_teams,
            total_participants: value.total_participants,
            female_percentage: value.female_percentage,
            location_types,
            events: value
                .events
                .into_values()
                .map(EventSubStructure::from)
                .collect(),
        }
    }
}

impl From<TempEventSubStructure> for EventSubStructure {
    fn from(value: TempEventSubStructure) -> Self {
        let mut location_types = value.location_types;
        location_types.sort();
        Self {
            id: value.id,
            name: value.name,
            level: value.level,
            date: value.date,
            location: value.location,
            total_institutions: value.total_institutions,
            total_teams: value.total_teams,
            total_participants: value.total_participants,
            female_percentage: value.female_percentage,
            location_types,
            teams: value.teams.into_values().collect(),
        }
    }
}

impl From<TempCompetitionYearResults> for CompetitionYearResults {
    fn from(value: TempCompetitionYearResults) -> Self {
        let mut location_types = value.location_types;
        location_types.sort();
        Self {
            total_institutions: value.total_institutions,
            total_teams: value.total_teams,
            total_participants: value.total_participants,
            female_percentage: value.female_percentage,
            location_types,
            events: value
                .events
                .into_values()
                .map(EventSubStructure::from)
                .collect(),
        }
    }
}

// ======================== Helper constructors ========================
impl TempCompetitionStructure {
    pub fn new(
        id: i32,
        name: String,
        website_url: Option<String>,
        gender_category: GenderCategory,
        years: Vec<i32>,
        total_institutions: i32,
        total_teams: i32,
        total_participants: i32,
        female_participants: i32,
        location_types: Vec<LocationType>,
        events: IndexMap<i32, TempEventSubStructure>,
    ) -> Self {
        Self {
            id,
            name,
            website_url,
            gender_category,
            years: years.into_iter().map(|y| y as u32).collect(),
            total_institutions: total_institutions as u32,
            total_teams: total_teams as u32,
            total_participants: total_participants as u32,
            female_percentage: female_participants as f32 / total_participants as f32,
            location_types,
            events,
        }
    }
}

impl TempEventSubStructure {
    pub fn new(
        id: i32,
        name: String,
        level: Option<i32>,
        date: NaiveDate,
        location: String,
        total_institutions: i32,
        total_teams: i32,
        total_participants: i32,
        female_participants: i32,
        location_types: Vec<LocationType>,
        teams: IndexMap<i32, TeamSubStructure>,
    ) -> Self {
        Self {
            id,
            name,
            level: level.map(|l| l as u32),
            date,
            location,
            total_institutions: total_institutions as u32,
            total_teams: total_teams as u32,
            total_participants: total_participants as u32,
            female_percentage: female_participants as f32 / total_participants as f32,
            location_types,
            teams,
        }
    }
}

impl TeamSubStructure {
    pub fn new(
        id: i32,
        name: String,
        rank: i32,
        institution_name: String,
        institution_short_name: Option<String>,
        institution_location: String,
        total_members: i32,
        female_members: i32,
    ) -> Self {
        Self {
            id,
            name,
            rank: rank as u32,
            institution_name,
            institution_short_name,
            institution_location,
            total_members: total_members as u32,
            female_percentage: female_members as f32 / total_members as f32,
        }
    }
}

impl CompetitionYearStats {
    pub fn update(
        &mut self,
        total_institutions: i32,
        total_teams: i32,
        total_participants: i32,
        female_participants: i32,
        location_types: Vec<LocationType>,
    ) {
        self.total_institutions = total_institutions as u32;
        self.total_teams = total_teams as u32;
        self.total_participants = total_participants as u32;
        self.female_percentage = female_participants as f32 / total_participants as f32;
        self.location_types = location_types;
    }
}

impl TempCompetitionYearResults {
    pub fn update(
        &mut self,
        total_institutions: i32,
        total_teams: i32,
        total_participants: i32,
        female_participants: i32,
        location_types: Vec<LocationType>,
    ) {
        self.total_institutions = total_institutions as u32;
        self.total_teams = total_teams as u32;
        self.total_participants = total_participants as u32;
        self.female_percentage = female_participants as f32 / total_participants as f32;
        self.location_types = location_types;
    }
}
