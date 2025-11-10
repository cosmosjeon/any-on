use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Tag {
    pub id: Uuid,
    pub tag_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateTag {
    pub tag_name: String,
    pub content: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct UpdateTag {
    pub tag_name: Option<String>,
    pub content: Option<String>,
}

impl Tag {
    /// Find all tags for a specific user
    pub async fn find_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Tag,
            r#"SELECT id as "id!: Uuid", tag_name, content as "content!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tags
               WHERE user_id = $1
               ORDER BY tag_name ASC"#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    /// Find tag by ID with user ownership validation
    pub async fn find_by_id_for_user(
        pool: &SqlitePool,
        id: Uuid,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Tag,
            r#"SELECT id as "id!: Uuid", tag_name, content as "content!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tags
               WHERE id = $1 AND user_id = $2"#,
            id,
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Tag,
            r#"SELECT id as "id!: Uuid", tag_name, content as "content!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tags
               ORDER BY tag_name ASC"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Tag,
            r#"SELECT id as "id!: Uuid", tag_name, content as "content!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>"
               FROM tags
               WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn create(
        pool: &SqlitePool,
        data: &CreateTag,
        user_id: &str,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Tag,
            r#"INSERT INTO tags (id, tag_name, content, user_id)
               VALUES ($1, $2, $3, $4)
               RETURNING id as "id!: Uuid", tag_name, content as "content!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            data.tag_name,
            data.content,
            user_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateTag,
        user_id: &str,
    ) -> Result<Self, sqlx::Error> {
        let existing = Self::find_by_id_for_user(pool, id, user_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;

        let tag_name = data.tag_name.as_ref().unwrap_or(&existing.tag_name);
        let content = data.content.as_ref().unwrap_or(&existing.content);

        sqlx::query_as!(
            Tag,
            r#"UPDATE tags
               SET tag_name = $2, content = $3, updated_at = datetime('now', 'subsec')
               WHERE id = $1 AND user_id = $4
               RETURNING id as "id!: Uuid", tag_name, content as "content!", created_at as "created_at!: DateTime<Utc>", updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            tag_name,
            content,
            user_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid, user_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM tags WHERE id = $1 AND user_id = $2",
            id,
            user_id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }

    /// Claim orphaned tags (where user_id is NULL) and assign them to the given user
    pub async fn claim_orphaned(pool: &SqlitePool, user_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                UPDATE tags
                SET user_id = $1
                WHERE user_id IS NULL
            "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
