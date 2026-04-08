use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::scorecard::Scorecard;
use crate::domain::repository::ScorecardRepository;

#[derive(Debug, Clone, sqlx::FromRow)]
struct ScorecardRow {
    service_id: Uuid,
    documentation_score: f64,
    test_coverage_score: f64,
    slo_compliance_score: f64,
    security_score: f64,
    overall_score: f64,
    evaluated_at: DateTime<Utc>,
}

impl From<ScorecardRow> for Scorecard {
    fn from(row: ScorecardRow) -> Self {
        Scorecard {
            service_id: row.service_id,
            documentation_score: row.documentation_score,
            test_coverage_score: row.test_coverage_score,
            slo_compliance_score: row.slo_compliance_score,
            security_score: row.security_score,
            overall_score: row.overall_score,
            evaluated_at: row.evaluated_at,
        }
    }
}

/// `ScorecardPostgresRepository` は `PostgreSQL` ベースのスコアカードリポジトリ。
pub struct ScorecardPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ScorecardPostgresRepository {
    #[allow(dead_code)]
    #[must_use] 
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    #[must_use] 
    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl ScorecardRepository for ScorecardPostgresRepository {
    async fn get(&self, service_id: Uuid) -> anyhow::Result<Option<Scorecard>> {
        let start = std::time::Instant::now();

        let row = sqlx::query_as::<_, ScorecardRow>(
            "SELECT service_id, documentation_score, test_coverage_score, \
             slo_compliance_score, security_score, overall_score, evaluated_at \
             FROM service_catalog.scorecards WHERE service_id = $1",
        )
        .bind(service_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("get", "scorecards", start.elapsed().as_secs_f64());
        }

        Ok(row.map(std::convert::Into::into))
    }

    async fn upsert(&self, scorecard: &Scorecard) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        sqlx::query(
            "INSERT INTO service_catalog.scorecards \
             (service_id, documentation_score, test_coverage_score, \
              slo_compliance_score, security_score, overall_score, evaluated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (service_id) DO UPDATE SET \
             documentation_score = EXCLUDED.documentation_score, \
             test_coverage_score = EXCLUDED.test_coverage_score, \
             slo_compliance_score = EXCLUDED.slo_compliance_score, \
             security_score = EXCLUDED.security_score, \
             overall_score = EXCLUDED.overall_score, \
             evaluated_at = EXCLUDED.evaluated_at",
        )
        .bind(scorecard.service_id)
        .bind(scorecard.documentation_score)
        .bind(scorecard.test_coverage_score)
        .bind(scorecard.slo_compliance_score)
        .bind(scorecard.security_score)
        .bind(scorecard.overall_score)
        .bind(scorecard.evaluated_at)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("upsert", "scorecards", start.elapsed().as_secs_f64());
        }

        Ok(())
    }
}
