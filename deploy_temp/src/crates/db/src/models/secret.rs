use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, FromRow, Clone)]
pub struct SecretRecord {
    pub id: Uuid,
    pub user_id: String,
    pub provider: String,
    pub name: String,
    pub key_version: i64,
    pub secret_blob: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SecretRecord {
    pub async fn upsert(
        pool: &SqlitePool,
        user_id: &str,
        provider: &str,
        name: &str,
        secret_blob: &[u8],
        key_version: i64,
    ) -> Result<(), sqlx::Error> {
        let secret_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO secrets (id, user_id, provider, name, secret_blob, key_version)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT(user_id, provider, name)
            DO UPDATE SET
                secret_blob = excluded.secret_blob,
                key_version = excluded.key_version,
                updated_at = datetime('now', 'subsec')
            "#,
            secret_id,
            user_id,
            provider,
            name,
            secret_blob,
            key_version
        )
        .execute(pool)
        .await?
        .rows_affected();

        Ok(())
    }

    pub async fn delete(
        pool: &SqlitePool,
        user_id: &str,
        provider: &str,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM secrets WHERE user_id = $1 AND provider = $2 AND name = $3"#,
            user_id,
            provider,
            name
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get(
        pool: &SqlitePool,
        user_id: &str,
        provider: &str,
        name: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            SecretRecord,
            r#"
            SELECT
                id as "id!: Uuid",
                user_id,
                provider,
                name,
                key_version,
                secret_blob,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM secrets
            WHERE user_id = $1 AND provider = $2 AND name = $3
            "#,
            user_id,
            provider,
            name
        )
        .fetch_optional(pool)
        .await
    }
}
