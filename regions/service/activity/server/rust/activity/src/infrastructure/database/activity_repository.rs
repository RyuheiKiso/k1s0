// アクティビティリポジトリの PostgreSQL 実装。
// RLS テナント分離のため、各 DB 操作の先頭で set_config('app.current_tenant_id', $1, true) を発行する。
// 冪等性キーによる重複チェック付き。Transactional Outbox パターンで outbox テーブルへ書き込む。
// 戻り値型は ActivityError（クリーンアーキテクチャ準拠。anyhow::Error は ActivityError::Infrastructure に変換する）。
use crate::domain::entity::activity::{Activity, ActivityFilter, CreateActivity};
use crate::domain::error::ActivityError;
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
            // ParseError は Display を実装しているため format! で文字列化して anyhow::Error に変換する
            activity_type: row.activity_type.parse().map_err(|e| anyhow::anyhow!("{}", e))?,
            content: row.content,
            duration_minutes: row.duration_minutes,
            status: row.status.parse().map_err(|e| anyhow::anyhow!("{}", e))?,
            idempotency_key: row.idempotency_key,
            version: row.version,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl ActivityRepository for ActivityPostgresRepository {
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> Result<Option<Activity>, ActivityError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから SELECT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at FROM activity_service.activities WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        row.map(Activity::try_from).transpose().map_err(ActivityError::Infrastructure)
    }

    async fn find_by_idempotency_key(&self, tenant_id: &str, key: &str) -> Result<Option<Activity>, ActivityError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから SELECT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        let row = sqlx::query_as::<_, ActivityRow>(
            "SELECT id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at FROM activity_service.activities WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        row.map(Activity::try_from).transpose().map_err(ActivityError::Infrastructure)
    }

    async fn find_all(&self, tenant_id: &str, filter: &ActivityFilter) -> Result<Vec<Activity>, ActivityError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから SELECT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        let rows = sqlx::query_as::<_, ActivityRow>(
            // task_id カラムは DB 上 text 型のため $1::text でキャストして比較する
            "SELECT id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at FROM activity_service.activities WHERE ($1::text IS NULL OR task_id = $1::text) AND ($2::text IS NULL OR actor_id = $2) AND ($3::text IS NULL OR status = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5",
        )
        .bind(filter.task_id.map(|id| id.to_string()))
        .bind(&filter.actor_id)
        .bind(filter.status.as_ref().map(|s| s.as_str()))
        .bind(filter.limit.unwrap_or(50))
        .bind(filter.offset.unwrap_or(0))
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        rows.into_iter().map(|r| Activity::try_from(r).map_err(ActivityError::Infrastructure)).collect()
    }

    async fn count(&self, tenant_id: &str, filter: &ActivityFilter) -> Result<i64, ActivityError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから COUNT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        let count: i64 = sqlx::query_scalar(
            // task_id カラムは DB 上 text 型のため $1::text でキャストして比較する
            "SELECT COUNT(*) FROM activity_service.activities WHERE ($1::text IS NULL OR task_id = $1::text) AND ($2::text IS NULL OR actor_id = $2) AND ($3::text IS NULL OR status = $3)",
        )
        .bind(filter.task_id.map(|id| id.to_string()))
        .bind(&filter.actor_id)
        .bind(filter.status.as_ref().map(|s| s.as_str()))
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        tx.commit().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        Ok(count)
    }

    async fn create(&self, tenant_id: &str, input: &CreateActivity, actor_id: &str) -> Result<Activity, ActivityError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから INSERT を実行する
        let mut tx = self.pool.begin().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        let activity_id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ActivityRow>(
            r#"INSERT INTO activity_service.activities (id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version)
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
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?;

        // HIGH-005 監査対応: Outbox イベントに tenant_id を含めてテナント分離を保証する
        sqlx::query(
            "INSERT INTO activity_service.outbox_events (id, aggregate_id, aggregate_type, event_type, payload, tenant_id) VALUES ($1, $2, 'activity', 'ActivityCreated', $3, $4)",
        )
        .bind(Uuid::new_v4())
        .bind(activity_id)
        .bind(serde_json::json!({ "activity_id": activity_id, "task_id": input.task_id, "actor_id": actor_id, "activity_type": input.activity_type.as_str() }))
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?;

        tx.commit().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        Activity::try_from(row).map_err(ActivityError::Infrastructure)
    }

    // updated_by を Option<String> として受け取る（mockall との互換性のため）
    async fn update_status(&self, tenant_id: &str, id: Uuid, status: &str, updated_by: Option<String>) -> Result<Activity, ActivityError> {
        // テナント分離のため SET LOCAL でセッション変数を設定してから UPDATE を実行する
        let mut tx = self.pool.begin().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;

        // updated_by を DB に書き込み、監査証跡を残す
        let row = sqlx::query_as::<_, ActivityRow>(
            r#"UPDATE activity_service.activities SET status = $2, updated_by = $3, version = version + 1, updated_at = now()
               WHERE id = $1
               RETURNING id, task_id, actor_id, activity_type, content, duration_minutes, status, idempotency_key, version, created_at, updated_at"#,
        )
        .bind(id)
        .bind(status)
        .bind(&updated_by)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ActivityError::Infrastructure(e.into()))?
        .ok_or_else(|| ActivityError::NotFound(format!("Activity '{}'", id)))?;

        // HIGH-005 監査対応: Approved・Rejected の両方で outbox イベントを発行する。
        // tenant_id を含めてテナント分離を保証する（イベント駆動の整合性確保）
        if status == "approved" {
            sqlx::query(
                "INSERT INTO activity_service.outbox_events (id, aggregate_id, aggregate_type, event_type, payload, tenant_id) VALUES ($1, $2, 'activity', 'ActivityApproved', $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(serde_json::json!({ "activity_id": id, "updated_by": updated_by }))
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        } else if status == "rejected" {
            // Rejected 時も outbox イベントを発行してダウンストリームサービスに通知する
            sqlx::query(
                "INSERT INTO activity_service.outbox_events (id, aggregate_id, aggregate_type, event_type, payload, tenant_id) VALUES ($1, $2, 'activity', 'ActivityRejected', $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(serde_json::json!({ "activity_id": id, "updated_by": updated_by }))
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ActivityError::Infrastructure(e.into()))?;
        }

        tx.commit().await.map_err(|e| ActivityError::Infrastructure(e.into()))?;
        Activity::try_from(row).map_err(ActivityError::Infrastructure)
    }
}
