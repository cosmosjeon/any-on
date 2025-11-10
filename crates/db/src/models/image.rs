use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct Image {
    pub id: Uuid,
    pub file_path: String, // relative path within cache/images/
    pub original_name: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub hash: String, // SHA256 hash for deduplication
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateImage {
    pub file_path: String,
    pub original_name: String,
    pub mime_type: Option<String>,
    pub size_bytes: i64,
    pub hash: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct TaskImage {
    pub id: Uuid,
    pub task_id: Uuid,
    pub image_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateTaskImage {
    pub task_id: Uuid,
    pub image_id: Uuid,
}

impl Image {
    pub async fn create(
        pool: &SqlitePool,
        data: &CreateImage,
        user_id: &str,
    ) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Image,
            r#"INSERT INTO images (id, file_path, original_name, mime_type, size_bytes, hash, user_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id as "id!: Uuid",
                         file_path as "file_path!",
                         original_name as "original_name!",
                         mime_type,
                         size_bytes as "size_bytes!",
                         hash as "hash!",
                         created_at as "created_at!: DateTime<Utc>",
                         updated_at as "updated_at!: DateTime<Utc>""#,
            id,
            data.file_path,
            data.original_name,
            data.mime_type,
            data.size_bytes,
            data.hash,
            user_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_user(pool: &SqlitePool, user_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Image,
            r#"SELECT id as "id!: Uuid",
                      file_path as "file_path!",
                      original_name as "original_name!",
                      mime_type,
                      size_bytes as "size_bytes!",
                      hash as "hash!",
                      created_at as "created_at!: DateTime<Utc>",
                      updated_at as "updated_at!: DateTime<Utc>"
               FROM images
               WHERE user_id = $1
               ORDER BY created_at DESC"#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id_for_user(
        pool: &SqlitePool,
        id: Uuid,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Image,
            r#"SELECT id as "id!: Uuid",
                      file_path as "file_path!",
                      original_name as "original_name!",
                      mime_type,
                      size_bytes as "size_bytes!",
                      hash as "hash!",
                      created_at as "created_at!: DateTime<Utc>",
                      updated_at as "updated_at!: DateTime<Utc>"
               FROM images
               WHERE id = $1 AND user_id = $2"#,
            id,
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_hash(pool: &SqlitePool, hash: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Image,
            r#"SELECT id as "id!: Uuid",
                      file_path as "file_path!",
                      original_name as "original_name!",
                      mime_type,
                      size_bytes as "size_bytes!",
                      hash as "hash!",
                      created_at as "created_at!: DateTime<Utc>",
                      updated_at as "updated_at!: DateTime<Utc>"
               FROM images
               WHERE hash = $1"#,
            hash
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Image,
            r#"SELECT id as "id!: Uuid",
                      file_path as "file_path!",
                      original_name as "original_name!",
                      mime_type,
                      size_bytes as "size_bytes!",
                      hash as "hash!",
                      created_at as "created_at!: DateTime<Utc>",
                      updated_at as "updated_at!: DateTime<Utc>"
               FROM images
               WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_task_id(
        pool: &SqlitePool,
        task_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Image,
            r#"SELECT i.id as "id!: Uuid",
                      i.file_path as "file_path!",
                      i.original_name as "original_name!",
                      i.mime_type,
                      i.size_bytes as "size_bytes!",
                      i.hash as "hash!",
                      i.created_at as "created_at!: DateTime<Utc>",
                      i.updated_at as "updated_at!: DateTime<Utc>"
               FROM images i
               JOIN task_images ti ON i.id = ti.image_id
               WHERE ti.task_id = $1
               ORDER BY ti.created_at"#,
            task_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn delete(
        pool: &SqlitePool,
        id: Uuid,
        user_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM images WHERE id = $1 AND user_id = $2"#,
            id,
            user_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn claim_orphaned(pool: &SqlitePool, user_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
                UPDATE images
                SET user_id = $1
                WHERE user_id IS NULL
            "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_orphaned_images(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Image,
            r#"SELECT i.id as "id!: Uuid",
                      i.file_path as "file_path!",
                      i.original_name as "original_name!",
                      i.mime_type,
                      i.size_bytes as "size_bytes!",
                      i.hash as "hash!",
                      i.created_at as "created_at!: DateTime<Utc>",
                      i.updated_at as "updated_at!: DateTime<Utc>"
               FROM images i
               LEFT JOIN task_images ti ON i.id = ti.image_id
               WHERE ti.task_id IS NULL"#
        )
        .fetch_all(pool)
        .await
    }
}

impl TaskImage {
    pub async fn create(pool: &SqlitePool, data: &CreateTaskImage) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            TaskImage,
            r#"INSERT INTO task_images (id, task_id, image_id)
               VALUES ($1, $2, $3)
               RETURNING id as "id!: Uuid",
                         task_id as "task_id!: Uuid",
                         image_id as "image_id!: Uuid", 
                         created_at as "created_at!: DateTime<Utc>""#,
            id,
            data.task_id,
            data.image_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn associate_many(
        pool: &SqlitePool,
        task_id: Uuid,
        image_ids: &[Uuid],
    ) -> Result<(), sqlx::Error> {
        for &image_id in image_ids {
            let task_image = CreateTaskImage { task_id, image_id };
            TaskImage::create(pool, &task_image).await?;
        }
        Ok(())
    }

    /// Associate multiple images with a task, skipping duplicates.
    pub async fn associate_many_dedup(
        pool: &SqlitePool,
        task_id: Uuid,
        image_ids: &[Uuid],
    ) -> Result<(), sqlx::Error> {
        for &image_id in image_ids {
            let id = Uuid::new_v4();
            sqlx::query!(
                r#"INSERT INTO task_images (id, task_id, image_id)
                   SELECT $1, $2, $3
                   WHERE NOT EXISTS (
                       SELECT 1 FROM task_images WHERE task_id = $2 AND image_id = $3
                   )"#,
                id,
                task_id,
                image_id
            )
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    pub async fn delete_by_task_id(pool: &SqlitePool, task_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM task_images WHERE task_id = $1"#, task_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
