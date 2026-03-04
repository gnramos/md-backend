use serde::Serialize;

use crate::shared::GenderCategory;

#[derive(Debug, Serialize)]
pub struct OrganizerStructure {
    pub id: i32,
    pub name: String,
    pub website_url: String,
    pub competitions: Vec<CompetitionSubStructure>
}

impl OrganizerStructure {
    pub fn new(id: i32, name: String, website_url: String) -> Self {
        Self { id, name, website_url, competitions: Vec::new() }
    }
}

#[derive(Debug, Serialize)]
pub struct CompetitionSubStructure {
    pub id: i32,
    pub name: String,
    pub website_url: String,
    pub gender_category: GenderCategory,
    pub events: Vec<EventSubStructure>
}

impl CompetitionSubStructure {
    pub fn new(id: i32, name: String, website_url: String, gender_category: GenderCategory, events: Vec<EventSubStructure>) -> Self {
        Self { id, name, website_url, gender_category, events }
    }
}

#[derive(Debug, Serialize)]
pub struct EventSubStructure {
    pub id: i32,
    pub name: String
}

impl EventSubStructure {
    pub fn new(id: i32, name: String) -> Self {
        Self { id, name }
    }
}