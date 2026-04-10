use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::rule::EvaluationLog;
use crate::domain::repository::EvaluationLogRepository;

/// 評価ログのJOINクエリ結果行（pagination用内部データ型）
#[derive(sqlx::FromRow)]
struct EvalLogJoinedRow {
    id: Uuid,
    tenant_id: String,
    #[allow(dead_code)]
    rule_set_id: Uuid,
    rule_id: Option<Uuid>,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    #[allow(dead_code)]
    matched: bool,
    #[allow(dead_code)]
    execution_time_ms: Option<i32>,
    #[allow(dead_code)]
    error_message: Option<String>,
    created_at: DateTime<Utc>,
    rule_set_name: String,
}

pub struct EvaluationLogPostgresRepository {
    pool: Arc<PgPool>,
}

impl EvaluationLogPostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EvaluationLogRepository for EvaluationLogPostgresRepository {
    async fn create(&self, log: &EvaluationLog) -> anyhow::Result<()> {
        // CRITICAL-RUST-001 監査対応: RLS テナント分離のためセッション変数を設定する。
        // set_config の第3引数 true は SET LOCAL（トランザクションスコープのみ有効）を意味する。
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&log.tenant_id)
            .execute(self.pool.as_ref())
            .await?;

        // We need a rule_set_id for the FK. Look up by name.
        let rule_set_id: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM rule_engine.rule_sets WHERE name = $1 LIMIT 1")
                .bind(&log.rule_set_name)
                .fetch_optional(self.pool.as_ref())
                .await?;

        let rule_set_id = rule_set_id
            .map(|r| r.0)
            .ok_or_else(|| anyhow::anyhow!("rule_set not found for name: {}", log.rule_set_name))?;

        let matched = log.matched_rule_id.is_some();
        let input = serde_json::json!({ "hash": &log.input_hash });

        // tenant_id カラムを INSERT に追加（migration 003 対応）
        sqlx::query(
            "INSERT INTO rule_engine.evaluation_logs \
             (id, tenant_id, rule_set_id, rule_id, input, output, matched, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(log.id)
        .bind(&log.tenant_id)
        .bind(rule_set_id)
        .bind(log.matched_rule_id)
        .bind(&input)
        .bind(&log.result)
        .bind(matched)
        .bind(log.evaluated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        rule_set_name: Option<String>,
        _domain: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(Vec<EvaluationLog>, u64)> {
        let offset = i64::from(page.saturating_sub(1) * page_size);
        let limit = i64::from(page_size);

        // Build dynamic WHERE clauses with separate parameter indices for
        // the data query ($1=limit, $2=offset, then $3..) and the count query ($1..).
        let mut data_where_clauses = Vec::new();
        let mut count_where_clauses = Vec::new();
        let mut data_idx = 2u32; // after $1=limit, $2=offset
        let mut count_idx = 0u32;

        if rule_set_name.is_some() {
            data_idx += 1;
            count_idx += 1;
            data_where_clauses.push(format!("rs.name = ${data_idx}"));
            count_where_clauses.push(format!("rs.name = ${count_idx}"));
        }
        if from.is_some() {
            data_idx += 1;
            count_idx += 1;
            data_where_clauses.push(format!("el.created_at >= ${data_idx}"));
            count_where_clauses.push(format!("el.created_at >= ${count_idx}"));
        }
        if to.is_some() {
            data_idx += 1;
            count_idx += 1;
            data_where_clauses.push(format!("el.created_at <= ${data_idx}"));
            count_where_clauses.push(format!("el.created_at <= ${count_idx}"));
        }

        let data_where_sql = if data_where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", data_where_clauses.join(" AND "))
        };

        let count_where_sql = if count_where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", count_where_clauses.join(" AND "))
        };

        let query_sql = format!(
            "SELECT el.id, el.tenant_id, el.rule_set_id, el.rule_id, el.input, el.output, \
                    el.matched, el.execution_time_ms, el.error_message, el.created_at, \
                    rs.name AS rule_set_name \
             FROM rule_engine.evaluation_logs el \
             JOIN rule_engine.rule_sets rs ON rs.id = el.rule_set_id \
             {data_where_sql} \
             ORDER BY el.created_at DESC LIMIT $1 OFFSET $2"
        );

        let count_sql = format!(
            "SELECT COUNT(*) \
             FROM rule_engine.evaluation_logs el \
             JOIN rule_engine.rule_sets rs ON rs.id = el.rule_set_id \
             {count_where_sql}"
        );

        let mut query = sqlx::query_as::<_, EvalLogJoinedRow>(&query_sql)
            .bind(limit)
            .bind(offset);

        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);

        if let Some(ref name) = rule_set_name {
            query = query.bind(name.clone());
            count_query = count_query.bind(name.clone());
        }
        if let Some(ref f) = from {
            query = query.bind(*f);
            count_query = count_query.bind(*f);
        }
        if let Some(ref t) = to {
            query = query.bind(*t);
            count_query = count_query.bind(*t);
        }

        let rows = query.fetch_all(self.pool.as_ref()).await?;
        let count = count_query.fetch_one(self.pool.as_ref()).await?;

        let logs = rows
            .into_iter()
            .map(|r| {
                let input_hash = {
                    use sha2::{Digest, Sha256};
                    let bytes = serde_json::to_vec(&r.input).unwrap_or_default();
                    let hash = Sha256::digest(&bytes);
                    format!("{hash:x}")
                };
                EvaluationLog {
                    id: r.id,
                    tenant_id: r.tenant_id,
                    rule_set_name: r.rule_set_name,
                    rule_set_version: 0,
                    matched_rule_id: r.rule_id,
                    input_hash,
                    result: r.output.unwrap_or(serde_json::Value::Null),
                    context: serde_json::Value::Object(serde_json::Map::new()),
                    evaluated_at: r.created_at,
                }
            })
            .collect();

        // LOW-008: 安全な型変換（オーバーフロー防止）
        Ok((logs, u64::try_from(count.0).unwrap_or(0)))
    }
}
