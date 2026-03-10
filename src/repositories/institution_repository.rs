use async_trait::async_trait;

use crate::{
    errors::AppResult,
    repositories::{
        Registry,
        types::{IdNameRow, institutions::InstitutionStructureRow},
    },
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InstitutionRepository: Send + Sync {
    async fn find_options_by_competitions(
        &self,
        competition_ids: Option<Vec<i32>>,
    ) -> AppResult<Vec<IdNameRow>>;
    async fn find_structures_by_ids(
        &self,
        institution_ids: Vec<i32>,
    ) -> AppResult<Vec<InstitutionStructureRow>>;
}

#[async_trait]
impl InstitutionRepository for Registry {
    async fn find_options_by_competitions(
        &self,
        competition_ids: Option<Vec<i32>>,
    ) -> AppResult<Vec<IdNameRow>> {
        let rows = if let Some(ids) = competition_ids {
            sqlx::query_as(
                "SELECT DISTINTC
                    i.id AS id,
                    i.name AS name
                FROM institution i
                JOIN team t ON t.institution_id = i.id
                JOIN team_event te ON te.team_id = t.id
                JOIN event e ON te.event_id = e.id
                JOIN competition c ON e.competition_id = c.id
                WHERE c.id = ANY($1::int[])
                ORDER BY i.name",
            )
            .bind(ids)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT
                    id, name
                FROM institution
                ORDER BY name",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    async fn find_structures_by_ids(
        &self,
        institution_ids: Vec<i32>,
    ) -> AppResult<Vec<InstitutionStructureRow>> {
        let rows = sqlx::query_as(
            "SELECT
                i.id AS institution_id,
                i.name AS institution_name,
                
                COUNT(DISTINCT t.id) OVER (PARTITION BY i.id) AS institution_total_teams,

                COUNT(*) FILTER (WHERE tem.role = 'Contestant')
                    OVER (PARTITION BY i.id) AS institution_total_contestants,

                COUNT(*) FILTER (
                    WHERE tem.role = 'Contestant'
                    AND m.gender = 'Female'
                ) OVER (PARTITION BY i.id) AS institution_female_contestants,

                c.id AS competition_id,
                c.name AS competition_name,
                c.website_url AS competition_website_url,

                e.id AS event_id,
                e.name AS event_name,

                t.id AS team_id,
                t.name AS team_name,

                te.rank AS team_event_rank,

                COUNT(*) FILTER (WHERE tem.role = 'Contestant') AS team_total_members,

                COUNT(*) FILTER(
                    WHERE tem.role = 'Contestant'
                    AND m.gender = 'Female'
                ) AS team_female_members
                
            FROM institution i
            JOIN team t ON t.intitution_id = i.id
            JOIN team_event te ON t.id = te.team_id
            JOIN event e ON e.id = te.event_id
            JOIN team_event_member tem ON te.id = tem.team_event_id
            JOIN member m ON m.id = tem.member_id

            WHERE i.id = ANY($1::int[])
                AND EXTRACT(YEAR FROM e.date) = (
                    SELECT EXTRACT(YEAR FROM MAX(e2.date))
                    FROM event e2
                    WHERE e2.competition_id = c.id
                )
            
            GROUP BY
                i.id, i.name,
                c.id, c.name, c.website_url,
                e.id, e.name,
                t.id, t.name,
                te.rank
                
            ORDER BY i.name, c.name, t.name",
        )
        .bind(institution_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
