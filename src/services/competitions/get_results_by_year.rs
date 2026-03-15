use indexmap::IndexMap;

use crate::{
    dtos::competitions::output::{
        CompetitionYearResults, TeamSubStructure, TempCompetitionYearResults, TempEventSubStructure,
    },
    errors::{AppError, AppResult},
    repositories::CompetitionRepository,
};

pub async fn get_results_by_year(
    repo: &dyn CompetitionRepository,
    competition_id: i32,
    year: Option<i32>,
) -> AppResult<CompetitionYearResults> {
    let year =
        year.ok_or_else(|| AppError::BadRequest("You need to specify the year.".to_string()))?;

    let structure = repo
        .find_competition_results_by_year(competition_id, year)
        .await?
        .into_iter()
        .fold(
            TempCompetitionYearResults::default(),
            |mut competition, row| {
                if competition.events.is_empty() {
                    competition.update(
                        row.competition_total_institutions,
                        row.competition_total_teams,
                        row.competition_total_participants,
                        row.competition_female_participants,
                        row.competition_location_types,
                    );
                }

                competition
                    .events
                    .entry(row.event_id)
                    .or_insert_with(|| {
                        TempEventSubStructure::new(
                            row.event_id,
                            row.event_name,
                            row.event_level,
                            row.event_date,
                            row.event_location,
                            row.event_total_institutions,
                            row.event_total_teams,
                            row.event_total_participants,
                            row.event_female_participants,
                            row.event_location_types,
                            IndexMap::new(),
                        )
                    })
                    .teams
                    .insert(
                        row.team_id,
                        TeamSubStructure::new(
                            row.team_id,
                            row.team_name,
                            row.team_rank,
                            row.institution_name,
                            row.institution_short_name,
                            row.institution_location,
                            row.team_total_members,
                            row.team_female_members,
                        ),
                    );

                competition
            },
        );

    Ok(CompetitionYearResults::from(structure))
}
