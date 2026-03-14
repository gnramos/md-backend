use async_trait::async_trait;

use crate::{
    errors::AppResult,
    repositories::{
        Registry,
        types::{
            IdNameRow,
            competitions::{CompetitionLocationStatsRow, CompetitionStructureRow},
        },
    },
    shared::types::LocationType,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CompetitionRepository: Send + Sync {
    async fn find_options_by_organizers(
        &self,
        organizer_ids: Option<Vec<i32>>,
    ) -> AppResult<Vec<IdNameRow>>;
    async fn find_structures_by_ids(
        &self,
        competition_ids: Vec<i32>,
    ) -> AppResult<Vec<CompetitionStructureRow>>;
    async fn find_location_stats_by_competition(
        &self,
        competition_id: i32,
        location_type: LocationType,
        year: i32,
    ) -> AppResult<Vec<CompetitionLocationStatsRow>>;
}

#[async_trait]
impl CompetitionRepository for Registry {
    async fn find_options_by_organizers(
        &self,
        organizer_ids: Option<Vec<i32>>,
    ) -> AppResult<Vec<IdNameRow>> {
        let rows = if let Some(ids) = organizer_ids {
            sqlx::query_as(
                "SELECT
                    id, name
                FROM competition
                WHERE organizer_id = ANY($1::int[])
                ORDER BY name",
            )
            .bind(ids)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                "SELECT
                    id, name
                FROM competition
                ORDER BY name",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows)
    }

    async fn find_structures_by_ids(
        &self,
        competitions_ids: Vec<i32>,
    ) -> AppResult<Vec<CompetitionStructureRow>> {
        let rows = sqlx::query_as(
            "WITH selected_events AS (
                SELECT
                    c.id AS competition_id,
                    c.name AS competition_name,
                    c.website_url AS competition_website_url,
                    c.gender_category AS competition_gender_category,

                    e.id AS event_id,
                    e.name AS event_name,
                    e.level AS event_level
                FROM competition c
                JOIN event e ON e.competition_id = c.id
                WHERE c.id = ANY($1::int[])
            ),
            event_metadata AS (
                SELECT
                    se.competition_id,
                    se.competition_name,
                    se.competition_website_url,
                    se.competition_gender_category,
                    se.event_id,
                    se.event_name,
                    se.event_level,
                    MAX(ei.date) AS event_date,
                    array_agg(
                        DISTINCT EXTRACT(YEAR FROM ei.date)::int
                        ORDER BY EXTRACT(YEAR FROM ei.date)::int
                    ) AS event_years
                FROM selected_events se
                JOIN event_instance ei ON ei.event_id = se.event_id
                GROUP BY
                    se.competition_id, se.competition_name, se.competition_gender_category, se.competition_website_url, se.event_id,
                    se.event_name, se.event_level
            ),
            latest_year_team_rows AS (
                SELECT
                    em.competition_id,
                    em.competition_name,
                    em.competition_website_url,
                    em.competition_gender_category,
                    em.event_id,
                    em.event_name,
                    em.event_level,
                    em.event_date,
                    em.event_years,
                    ei.location_id AS event_location_id,

                    te.id AS team_event_id,
                    te.rank AS team_rank,
                    te.campus_location_id,

                    t.id AS team_id,
                    t.name AS team_name,

                    i.id AS institution_id,
                    i.name AS institution_name,
                    i.short_name AS institution_short_name,
                    i.main_location_id AS main_location_id,

                    COUNT(*) FILTER (WHERE tem.role = 'Contestant') AS team_total_members,
                    COUNT(*) FILTER (
                        WHERE tem.role = 'Contestant'
                        AND m.gender = 'Female'
                    ) AS team_female_members
                FROM event_metadata em
                JOIN event_instance ei ON ei.event_id = em.event_id
                    AND EXTRACT(YEAR FROM ei.date)::int = EXTRACT(YEAR FROM em.event_date)::int
                JOIN team_event te ON te.event_instance_id = ei.id
                JOIN team t ON t.id = te.team_id
                JOIN institution i ON i.id = t.institution_id
                JOIN team_event_member tem ON tem.team_event_id = te.id
                JOIN member m ON m.id = tem.member_id
                GROUP BY
                    em.competition_id, em.competition_name, em.competition_gender_category, em.competition_website_url, em.event_id, em.event_name, em.event_level, em.event_date, em.event_years,
                    ei.location_id,
                    te.id, te.rank, te.campus_location_id,
                    t.id, t.name,
                    i.id, i.name, i.short_name, i.main_location_id
            ),
            event_location_base AS (
                SELECT DISTINCT
                    lytr.event_id,
                    lytr.event_location_id
                FROM latest_year_team_rows lytr
            ),
            event_location AS (
                SELECT
                    elb.event_id,
                    string_agg(lt.name, ', ' ORDER BY lt.depth) AS event_location
                FROM event_location_base elb
                CROSS JOIN LATERAL get_location_tree(elb.event_location_id) lt
                GROUP BY elb.event_id
            ),
            event_totals AS (
                SELECT
                    event_id,
                    COUNT(DISTINCT institution_id) AS event_total_institutions,
                    COUNT(DISTINCT team_id) AS event_total_teams,
                    SUM(team_total_members) AS event_total_participants,
                    SUM(team_female_members) AS event_female_participants
                FROM latest_year_team_rows lytr
                GROUP BY event_id
            ),
            competition_totals AS (
                SELECT
                    competition_id,
                    COUNT(DISTINCT institution_id) AS competition_total_institutions,
                    COUNT(DISTINCT team_id) AS competition_total_teams,
                    SUM(team_total_members) AS competition_total_participants,
                    SUM(team_female_members) AS competition_female_participants
                FROM latest_year_team_rows lytr
                GROUP BY competition_id
            ),
            team_location AS (
                SELECT
                    lytr.team_event_id,
                    string_agg(lt.name, ', ' ORDER BY lt.depth) AS institution_location
                FROM latest_year_team_rows lytr
                CROSS JOIN LATERAL get_location_tree(
                    COALESCE(lytr.campus_location_id, lytr.main_location_id)    
                ) lt
                GROUP BY lytr.team_event_id
            ),
            team_location_types AS (
                SELECT DISTINCT
                    lytr.team_event_id,
                    lytr.event_id,
                    lytr.competition_id,
                    lt.type AS location_type,
                    lt.depth AS location_depth
                FROM latest_year_team_rows lytr
                CROSS JOIN LATERAL get_location_tree(
                    COALESCE(lytr.campus_location_id, lytr.main_location_id)
                ) lt
            ),
            event_location_types AS (
                SELECT
                    event_id,
                    array_agg(DISTINCT location_type) AS event_location_types
                FROM team_location_types
                GROUP BY event_id
            ),
            competition_location_types AS (
                SELECT
                    competition_id,
                    array_agg(DISTINCT location_type) AS competition_location_types
                FROM team_location_types
                GROUP BY competition_id
            )
            SELECT
                lytr.competition_id,
                lytr.competition_name,
                lytr.competition_website_url,
                lytr.competition_gender_category,
                ct.competition_total_institutions,
                ct.competition_total_teams,
                ct.competition_total_participants,
                ct.competition_female_participants,
                clt.competition_location_types,

                lytr.event_id,
                lytr.event_name,
                lytr.event_level,
                lytr.event_date,
                el.event_location,
                lytr.event_years,
                et.event_total_institutions,
                et.event_total_teams,
                et.event_total_participants,
                et.event_female_participants,
                elt.event_location_types,

                lytr.institution_name,
                lytr.institution_short_name,
                tl.institution_location,

                lytr.team_id,
                lytr.team_name,
                lytr.team_rank,
                lytr.team_total_members,
                lytr.team_female_members

            FROM latest_year_team_rows lytr
            JOIN competition_totals ct ON ct.competition_id = lytr.competition_id
            JOIN competition_location_types clt ON clt.competition_id = lytr.competition_id
            JOIN event_location el ON el.event_id = lytr.event_id
            JOIN event_totals et ON et.event_id = lytr.event_id
            JOIN event_location_types elt ON elt.event_id = lytr.event_id
            JOIN team_location tl ON tl.team_event_id = lytr.team_event_id
            
            ORDER BY lytr.competition_name, lytr.event_level, lytr.event_name, lytr.team_rank"
        )
        .bind(competitions_ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn find_location_stats_by_competition(
        &self,
        competition_id: i32,
        location_type: LocationType,
        year: i32,
    ) -> AppResult<Vec<CompetitionLocationStatsRow>> {
        let rows = sqlx::query_as(
            "SELECT
                lt.id AS location_id,
                lt.name AS location_name,

                COUNT(DISTINCT i.id) AS total_institutions,
                COUNT(DISTINCT t.id) AS total_teams,

                SUM(p.total_participants) AS total_participants,
                SUM(p.female_participants) AS female_participants

            FROM team_event te
            JOIN team t ON t.id = te.team_id
            JOIN institution i ON i.id = t.institution_id
            CROSS JOIN LATERAL get_location_tree(COALESCE(te.campus_location_id, i.main_location_id)) lt
            JOIN event_instance ei ON ei.id = te.event_instance_id
            JOIN event e ON e.id = ei.event_id

            JOIN (
                SELECT
                    tem.team_event_id,
                    COUNT(*) FILTER (WHERE tem.role = 'Contestant') AS total_participants,
                    COUNT(*) FILTER (
                        WHERE tem.role = 'Contestant'
                        AND m.gender = 'Female'
                    ) AS female_participants
                FROM team_event_member tem
                JOIN member m ON m.id = tem.member_id
                GROUP BY tem.team_event_id
            ) p ON p.team_event_id = te.id

            WHERE e.competition_id = $1::int
            AND lt.type = $2::location_type
            AND EXTRACT(YEAR FROM ei.date) = $3::int

            GROUP BY lt.id, lt.name
            ORDER BY lt.name"
        )
        .bind(competition_id)
        .bind(location_type)
        .bind(year)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

/*

"SELECT
    c.id AS competition_id,
    c.name AS competition_name,
    c.gender_category AS competition_gender_category,
    c.website_url AS competition_website_url,

    COUNT(DISTINCT t.id) OVER (PARTITION BY c.id) AS competition_total_teams,
    SUM(p.team_total_members) OVER (PARTITION BY c.id) AS competition_total_participants,
    SUM(p.team_female_members) OVER (PARTITION BY c.id) AS competition_female_participants,

    e.id AS event_id,
    e.name AS event_name,
    e.level AS event_level,
    ei.date AS event_date,

    string_agg(elt.name, ', ' ORDER BY elt.depth) AS event_location,
    COUNT(DISTINCT t.id) OVER (PARTITION BY e.id) AS event_total_teams,
    SUM(p.team_total_members) OVER (PARTITION BY e.id) AS event_total_participants,
    SUM(p.team_female_members) OVER (PARTITION BY e.id) AS event_female_participants,
    COUNT(DISTINCT i.id) OVER (PARTITION BY e.id) AS event_total_institutions,

    i.name AS institution_name,
    i.short_name AS institution_short_name,
    string_agg(ilt.name, ', ' ORDER BY ilt.depth) AS institution_location,

    ilt.type AS institution_location_type,

    t.id AS team_id,
    t.name AS team_name,
    te.rank AS team_rank,
    p.team_total_members AS team_total_members,
    p.team_female_members AS team_female_members

FROM competition c
JOIN event e ON c.id = e.competition_id
JOIN event_instance ei ON ei.event_id = e.id
CROSS JOIN LATERAL get_location_tree(ei.location_id) elt
JOIN team_event te ON te.event_instance_id = ei.id
JOIN team t ON t.id = te.team_id
JOIN institution i ON i.id = t.institution_id

CROSS JOIN LATERAL get_location_tree(COALESCE(
    te.campus_location_id,
    i.main_location_id)
) ilt

JOIN (
    SELECT
        tem.team_event_id,
        COUNT(*) FILTER (WHERE tem.role = 'Contestant') AS team_total_members,
    COUNT(*) FILTER (
        WHERE tem.role = 'Contestant'
        AND m.gender = 'Female'
    ) AS team_female_members
    FROM team_event_member tem
    JOIN member m ON m.id = tem.member_id
    GROUP BY tem.team_event_id
) p ON p.team_event_id = te.id

WHERE c.id = ANY($1::int[])
AND EXTRACT(YEAR FROM ei.date) = (
    SELECT EXTRACT(YEAR FROM MAX(ei2.date))
    FROM event_instance ei2
    JOIN event e2 ON e2.id = ei2.event_id
    WHERE e2.competition_id = c.id
)

GROUP BY
    c.id, c.name, c.gender_category, c.website_url,
    e.id, e.name, e.level, ei.date,
    i.id, i.name, i.short_name,
    ilt.type,
    te.id, te.rank,
    t.id, t.name,
    p.team_total_members, p.team_female_members
    
ORDER BY c.name, e.level, e.name, te.rank, t.name"
*/