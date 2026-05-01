use evt_domain::{
    AppError, PUBLIC_SPACE_DESCRIPTION, PUBLIC_SPACE_NAME, SpaceMemberSummary, SpaceRole,
    SpaceSummary, SpaceVisibility,
};
use sqlx::{FromRow, MySqlPool};

use super::map_db_error;

#[derive(Clone)]
pub struct SpaceRepository {
    pool: MySqlPool,
}

impl SpaceRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn ensure_default_space(
        &self,
        slug: &str,
        owner_user_id: i64,
    ) -> Result<SpaceSummary, AppError> {
        sqlx::query(
            r#"
            INSERT INTO spaces (slug, name, description, owner_user_id, visibility)
            VALUES (?, ?, ?, ?, 0)
            ON DUPLICATE KEY UPDATE
              name = VALUES(name),
              description = VALUES(description)
            "#,
        )
        .bind(slug)
        .bind(PUBLIC_SPACE_NAME)
        .bind(PUBLIC_SPACE_DESCRIPTION)
        .bind(owner_user_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        let space: SpaceSummary = self
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| AppError::Internal("default space cannot be loaded".into()))?;

        self.add_member(space.id, owner_user_id, SpaceRole::Owner, owner_user_id)
            .await?;

        Ok(space)
    }

    pub async fn create(
        &self,
        slug: &str,
        name: &str,
        description: &str,
        owner_user_id: i64,
        visibility: SpaceVisibility,
    ) -> Result<SpaceSummary, AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO spaces (slug, name, description, owner_user_id, visibility)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(slug)
        .bind(name)
        .bind(description)
        .bind(owner_user_id)
        .bind(visibility as i32)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        let space: SpaceSummary = self
            .find_by_id(result.last_insert_id() as i64)
            .await?
            .ok_or_else(|| AppError::Internal("created space cannot be loaded".into()))?;

        self.add_member(space.id, owner_user_id, SpaceRole::Owner, owner_user_id)
            .await?;

        Ok(space)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<SpaceSummary>, AppError> {
        sqlx::query_as::<_, SpaceRow>(
            r#"
            SELECT
              s.id,
              s.slug,
              s.name,
              s.description,
              s.owner_user_id,
              s.visibility,
              COALESCE(member_stats.members_count, 0) AS members_count,
              NULL AS current_user_role,
              s.created_at,
              s.updated_at
            FROM spaces s
            LEFT JOIN (
              SELECT space_id, COUNT(*) AS members_count
              FROM space_members
              GROUP BY space_id
            ) member_stats ON member_stats.space_id = s.id
            WHERE s.id = ?
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<SpaceSummary>, AppError> {
        sqlx::query_as::<_, SpaceRow>(
            r#"
            SELECT
              s.id,
              s.slug,
              s.name,
              s.description,
              s.owner_user_id,
              s.visibility,
              COALESCE(member_stats.members_count, 0) AS members_count,
              NULL AS current_user_role,
              s.created_at,
              s.updated_at
            FROM spaces s
            LEFT JOIN (
              SELECT space_id, COUNT(*) AS members_count
              FROM space_members
              GROUP BY space_id
            ) member_stats ON member_stats.space_id = s.id
            WHERE s.slug = ?
            LIMIT 1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn list_visible(
        &self,
        viewer_user_id: Option<i64>,
        limit: u64,
    ) -> Result<Vec<SpaceSummary>, AppError> {
        let rows = if let Some(viewer_user_id) = viewer_user_id {
            sqlx::query_as::<_, SpaceRow>(
                r#"
                SELECT
                  s.id,
                  s.slug,
                  s.name,
                  s.description,
                  s.owner_user_id,
                  s.visibility,
                  COALESCE(member_stats.members_count, 0) AS members_count,
                  CASE
                    WHEN s.owner_user_id = ? THEN 2
                    ELSE current_member.role
                  END AS current_user_role,
                  s.created_at,
                  s.updated_at
                FROM spaces s
                LEFT JOIN (
                  SELECT space_id, COUNT(*) AS members_count
                  FROM space_members
                  GROUP BY space_id
                ) member_stats ON member_stats.space_id = s.id
                LEFT JOIN space_members current_member
                  ON current_member.space_id = s.id
                 AND current_member.user_id = ?
                WHERE s.visibility = 0
                   OR s.owner_user_id = ?
                   OR current_member.user_id IS NOT NULL
                ORDER BY s.id ASC
                LIMIT ?
                "#,
            )
            .bind(viewer_user_id)
            .bind(viewer_user_id)
            .bind(viewer_user_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, SpaceRow>(
                r#"
                SELECT
                  s.id,
                  s.slug,
                  s.name,
                  s.description,
                  s.owner_user_id,
                  s.visibility,
                  COALESCE(member_stats.members_count, 0) AS members_count,
                  NULL AS current_user_role,
                  s.created_at,
                  s.updated_at
                FROM spaces s
                LEFT JOIN (
                  SELECT space_id, COUNT(*) AS members_count
                  FROM space_members
                  GROUP BY space_id
                ) member_stats ON member_stats.space_id = s.id
                WHERE s.visibility = 0
                ORDER BY s.id ASC
                LIMIT ?
                "#,
            )
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        };

        rows.map(|items| items.into_iter().map(Into::into).collect())
            .map_err(map_db_error)
    }

    pub async fn add_member(
        &self,
        space_id: i64,
        user_id: i64,
        role: SpaceRole,
        invited_by_user_id: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO space_members (space_id, user_id, role, invited_by_user_id)
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
              role = VALUES(role),
              invited_by_user_id = VALUES(invited_by_user_id),
              updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(space_id)
        .bind(user_id)
        .bind(role as i32)
        .bind(invited_by_user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    pub async fn find_member(
        &self,
        space_id: i64,
        user_id: i64,
    ) -> Result<Option<SpaceMemberSummary>, AppError> {
        sqlx::query_as::<_, SpaceMemberRow>(
            r#"
            SELECT
              sm.space_id,
              sm.user_id,
              u.username,
              COALESCE(up.nickname, u.username) AS nickname,
              COALESCE(up.avatar, '') AS avatar,
              sm.role,
              sm.invited_by_user_id,
              sm.created_at
            FROM space_members sm
            INNER JOIN users u ON u.id = sm.user_id
            LEFT JOIN user_profiles up ON up.user_id = sm.user_id
            WHERE sm.space_id = ? AND sm.user_id = ?
            LIMIT 1
            "#,
        )
        .bind(space_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    pub async fn list_members(&self, space_id: i64) -> Result<Vec<SpaceMemberSummary>, AppError> {
        sqlx::query_as::<_, SpaceMemberRow>(
            r#"
            SELECT
              sm.space_id,
              sm.user_id,
              u.username,
              COALESCE(up.nickname, u.username) AS nickname,
              COALESCE(up.avatar, '') AS avatar,
              sm.role,
              sm.invited_by_user_id,
              sm.created_at
            FROM space_members sm
            INNER JOIN users u ON u.id = sm.user_id
            LEFT JOIN user_profiles up ON up.user_id = sm.user_id
            WHERE sm.space_id = ?
            ORDER BY sm.role DESC, sm.created_at ASC, sm.user_id ASC
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_db_error)
    }

    pub async fn remove_member(&self, space_id: i64, user_id: i64) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM space_members
            WHERE space_id = ? AND user_id = ?
            "#,
        )
        .bind(space_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }
}

#[derive(Debug, FromRow)]
struct SpaceRow {
    id: i64,
    slug: String,
    name: String,
    description: String,
    owner_user_id: i64,
    visibility: i32,
    members_count: i64,
    current_user_role: Option<i32>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<SpaceRow> for SpaceSummary {
    fn from(row: SpaceRow) -> Self {
        Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
            description: row.description,
            owner_user_id: row.owner_user_id,
            visibility: if row.visibility == 1 {
                SpaceVisibility::Private
            } else {
                SpaceVisibility::Public
            },
            members_count: row.members_count,
            current_user_role: row.current_user_role.map(|role| match role {
                2 => SpaceRole::Owner,
                1 => SpaceRole::Admin,
                _ => SpaceRole::Member,
            }),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct SpaceMemberRow {
    space_id: i64,
    user_id: i64,
    username: String,
    nickname: String,
    avatar: String,
    role: i32,
    invited_by_user_id: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<SpaceMemberRow> for SpaceMemberSummary {
    fn from(row: SpaceMemberRow) -> Self {
        Self {
            space_id: row.space_id,
            user_id: row.user_id,
            username: row.username,
            nickname: row.nickname,
            avatar: row.avatar,
            role: match row.role {
                2 => SpaceRole::Owner,
                1 => SpaceRole::Admin,
                _ => SpaceRole::Member,
            },
            invited_by_user_id: row.invited_by_user_id,
            created_at: row.created_at,
        }
    }
}
