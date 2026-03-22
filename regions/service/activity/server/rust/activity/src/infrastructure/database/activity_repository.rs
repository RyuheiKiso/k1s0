// アクティビティリポジトリの PostgreSQL 実装。冪等性キーによる重複チェック付き。
use crate::domain::entity::activity::{Activity, ActivityFilter, ActivityStatus, ActivityType, CreateActivity};
use crate::domain::repository::activity_repository::ActivityRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ActivityPostgresRepository {
    pool: PgPool,
}

impl ActivityPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ActivityRow {
    id: Uuid,
    task_id: Uuid,
    actor_id: String,
    activity_type: String,
    content: Option<String>,
    duration_minutes: Option<i32>,
    status: String,
    idempotency_key: Option<String>,
    version: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ActivityRow> for Activity {
    type Error = anyhow::Error;
    fn try_from(row: ActivityRow) -> Result<Self, Self::Error> {
        Ok(Activity {
            id: row.id,
            task_id: row.task_id,
            actor_id: row.actor_id,
            activity_type: row.activity_type.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            content: row.content,
            duration_minutes: row.duration_minutes,
            status: row.status.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            idempotency_key: row.idempotency_key,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl ActivityRepository for ActivityPostgresRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Activity>> {
        let row = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at FROM activity.activities WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Activity::try_from).transpose()
    }

    async fn find_by_idempotency_key(&self, key: &str) -> anyhow::Result<Option<Activity>> {
        let row = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at FROM activity.activities WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Activity::try_from).transpose()
    }

    async fn find_all(&self, filter: &ActivityFilter) -> anyhow::Result<Vec<Activity>> {
        let rows = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at FROM activity.activities WHERE ($1::uuid IS NULL OR task_id = $1) AND ($2::text IS NULL OR actor_id = $2) AND ($3::text IS NULL OR status = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5",
        )
        .bind(filter.task_id)
        .bind(&filter.actor_id)
        .bind(filter.status.as_ref().map(|s| s.as_str()))
        .bind(filter.limit.unwrap_or(50))
        .bind(filter.offset.unwrap_or(0))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Activity::try_from).collect()
    }

    async fn count(&self, filter: &ActivityFilter) -> anyhow::Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM activity.activities WHERE ($1::uuid IS NULL OR task_id = $1) AND ($2::text IS NULL OR actor_id = $2) AND ($3::text IS NULL OR status = $3)",
        )
        .bind(filter.task_id)
        .bind(&filter.actor_id)
        .bind(filter.status.as_ref().map(|s| s.as_str()))
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn create(&self, input: &CreateActivity, actor_id: &str) -> anyhow::Result<Activity> {
        let mut tx = self.pool.begin().await?;
        let activity_id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"INSERT INTO activity.activities (id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version)
               VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, 1)
               RETURNING id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at"#,
        )
        .bind(activity_id)
        .bind(input.task_id)
        .bind(actor_id)
        .bind(input.activity_type.as_str())
        .bind(&input.content)
        .bind(input.duration_minutes)
        .bind(&input.idempotency_key)
        .fetch_one(&mut *tx)
        .await?;

        // Outbox イベント
        sqlx::query(
            "INSERT INTO activity.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'activity', 'ActivityCreated', $3)",
        )
        .bind(Uuid::new_v4())
        .bind(activity_id)
        .bind(serde_json::json!({ "activity_id": activity_id, "task_id": input.task_id, "actor_id": actor_id, "activity_type": input.activity_type.as_str() }))
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Activity::try_from(row)
    }

    async fn update_status(&self, id: Uuid, status: &str, _updated_by: Option<&str>) -> anyhow::Result<Activity> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"UPDATE activity.activities SET status = $2, version = version + 1, updated_at = now()
               WHERE id = $1
               RETURNING id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at"#,
        )
        .bind(id)
        .bind(status)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Activity '{}' not found", id))?;

        // Approved のみ outbox イベントを発行する
        if status == "approved" {
            sqlx::query(
                "INSERT INTO activity.outbox_events (id, aggregate_id, aggregate_type, event_type, payload) VALUES ($1, $2, 'activity', 'ActivityApproved', $3)",
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(serde_json::json!({ "activity_id": id }))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Activity::try_from(row)
    }
}
