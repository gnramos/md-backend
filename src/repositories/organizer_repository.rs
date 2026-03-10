use async_trait::async_trait;

use crate::{
    errors::AppResult,
    repositories::{
        Registry,
        types::{IdNameRow, organizers::OrganizerStructureRow},
    },
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrganizerRepository: Send + Sync {
    async fn find_options(&self) -> AppResult<Vec<IdNameRow>>;
    async fn find_structures_by_ids(
        &self,
        organizer_ids: Vec<i32>,
    ) -> AppResult<Vec<OrganizerStructureRow>>;
}

#[async_trait]
impl OrganizerRepository for Registry {
    async fn find_options(&self) -> AppResult<Vec<IdNameRow>> {
        let rows = sqlx::query_as("SELECT id, name FROM organizer")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    async fn find_structures_by_ids(
        &self,
        organizer_ids: Vec<i32>,
    ) -> AppResult<Vec<OrganizerStructureRow>> {
        let rows = sqlx::query_as(
            "SELECT DISTINCT
                o.id AS organizer_id,
                o.name AS organizer_name,
                o.website_url AS organizer_website_url,

                c.id AS competition_id,
                c.name AS competition_name,
                c.gender_category AS competition_gender_category,
                c.website_url AS competition_website_url,

                COUNT(DISTINCT t.id) OVER (PARTITION BY c.id) AS competition_total_teams,

                COUNT(*) FILTER (WHERE tem.role = 'Contestant')
                    OVER (PARTITION BY c.id) AS competition_total_participants,

                COUNT(*) FILTER (
                    WHERE tem.role = 'Contestant'
                    AND m.gender = 'Female'
                ) OVER (PARTITION BY c.id) AS competition_female_participants,

                e.id AS event_id,
                e.name AS event_name,

                COUNT(DISTINCT t.id) OVER (PARTITION BY e.id) AS event_total_teams,

                COUNT(*) FILTER (WHERE tem.role = 'Contestant')
                    OVER (PARTITION BY e.id) AS event_total_participants,

                COUNT(*) FILTER (
                    WHERE tem.role = 'Contestant'
                    AND m.gender = 'Female'
                ) OVER (PARTITION BY e.id) AS event_female_participants

            FROM organizer o
            JOIN competition c ON o.id = c.organizer_id
            JOIN event e ON c.id = e.competition_id
            JOIN team_event te ON e.id = te.event_id
            JOIN team t ON t.id = te.team_id
            JOIN team_event_member tem ON te.id = tem.team_event_id
            JOIN member m ON m.id = tem.member_id

            WHERE c.id = ANY($1::int[])
                AND EXTRACT(YEAR FROM e.date) = (
                    SELECT EXTRACT(YEAR FROM MAX(e2.date))
                    FROM event e2
                    WHERE e2.competition_id = c.id
                )
            
            GROUP BY
                o.id, o.name, o.website_url,
                c.id, c.name, c.gender_category, c.website_url,
                e.id, e.name
                
            ORDER BY o.name, c.name, e.name",
        )
        .bind(organizer_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
