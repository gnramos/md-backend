use async_trait::async_trait;

use crate::{errors::AppResult, repositories::{Registry, types::{IdNameRow, competitions::CompetitionStructureRow}}};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CompetitionRepository: Send + Sync {
    async fn find_options_by_organizers(
        &self,
        organizer_ids: Option<Vec<i32>>
    ) -> AppResult<Vec<IdNameRow>>;
    async fn find_structures_by_ids(
        &self,
        competition_ids: Vec<i32>
    ) -> AppResult<Vec<CompetitionStructureRow>>;
}

#[async_trait]
impl CompetitionRepository for Registry {
    async fn find_options_by_organizers(
        &self,
        organizer_ids: Option<Vec<i32>>
    ) -> AppResult<Vec<IdNameRow>> {
        let rows = if let Some(ids) = organizer_ids {
            sqlx::query_as(
               "SELECT 
                   id, name 
               FROM competition
               WHERE organizer_id = ANY($1)
               ORDER BY name"
           )
           .bind(ids)
           .fetch_all(&self.pool)
           .await?
        } else {
            sqlx::query_as(
                "SELECT 
                    id, name
                FROM competition
                ORDER BY name"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    async fn find_structures_by_ids(
        &self,
        competitions_ids: Vec<i32>
    ) -> AppResult<Vec<CompetitionStructureRow>> {
        let rows = sqlx::query_as(
            "WITH RECURSIVE location_tree AS (
                -- 1) caso base (anchor member)
                SELECT
                    l.id,
                    l.parent_id,
                    l.name,
                    1 AS depth,
                    e.id AS event_id
                FROM event e
                JOIN location l on l.id = e.location_id

                UNION ALL

                -- 2) parte recursiva (recursive member)
                SELECT
                    parent.id,
                    parent.parent_id,
                    parent.name,
                    lt.depth + 1,
                    lt.event_id
                FROM location parent
                JOIN location_tree lt ON parent.id = lt.parent_id
            )

            SELECT
                c.id AS competition_id,
                c.name AS competition_name,
                c.gender_category AS competition_gender_category,
                c.website_url AS competition_website_url,

                e.id AS event_id,
                e.name AS event_name,
                e.level AS event_level,
                e.date AS event_date,

                string_agg(lt.name, ', ' ORDER BY lt.depth)
                    AS event_location,

                COUNT(*) FILTER (WHERE tem.role = 'Contestant')
                    OVER (PARTITION BY e.id) AS event_total_participants,

                COUNT(*) FILTER (
                    WHERE tem.role = 'Contestant'
                    AND m.gender = 'Female'
                ) OVER (PARTITION BY e.id) AS event_female_participants,

                t.id AS team_id,
                t.name AS team_name,

                COUNT(*) FILTER (WHERE tem.role = 'Contestant')
                    AS team_total_members,

                COUNT(*) FILTER (
                    WHERE tem.role = 'Contestant'
                    AND m.gender = 'Female'
                ) AS team_female_members

            FROM competition c
            JOIN event e ON c.id = e.competition_id
            JOIN location l ON l.id = e.location_id
            JOIN location_tree lt ON lt.id = l.id
            JOIN team_event te ON e.id = te.event_id
            JOIN team t ON t.id = te.team_id
            JOIN team_event_member tem ON te.id = tem.team_event_id
            JOIN member m ON m.id = tem.member_id

            WHERE c.id = ANY($1::int[])
                AND e.date = (
                    SELECT MAX(e2.date)
                    FROM event e2
                    WHERE e2.competition_id = c.id
                )

            GROUP BY
                c.id, c.name, c.gender_category, c.website_url,
                e.id, e.name, e.level, e.date,
                l.id, l.name,
                t.id, t.name"
        )
        .bind(competitions_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}