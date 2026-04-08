use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerJob {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
    pub status: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// テナント ID: RLS によるテナント分離のために使用する（CRIT-005 対応）
    pub tenant_id: String,
}

impl SchedulerJob {
    #[must_use] 
    pub fn new(name: String, cron_expression: String, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: format!("job_{}", uuid::Uuid::new_v4().simple()),
            name,
            description: None,
            cron_expression,
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload,
            status: "active".to_string(),
            next_run_at: None,
            last_run_at: None,
            created_at: now,
            updated_at: now,
            tenant_id: "system".to_string(),
        }
    }

    /// cron 式から次回実行時刻を計算する。
    #[must_use] 
    pub fn next_run_at(&self) -> Option<DateTime<Utc>> {
        let cron_6field = to_6field_cron(&self.cron_expression);
        let tz = parse_timezone(&self.timezone)?;
        cron::Schedule::from_str(&cron_6field)
            .ok()?
            .upcoming(tz)
            .next()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

/// 5フィールド crontab 形式を cron クレートの6フィールド形式に変換する。
/// 既に6/7フィールドの場合はそのまま返す。
fn to_6field_cron(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() == 5 {
        format!("0 {expr}")
    } else {
        expr.to_string()
    }
}

/// cron 式を検証する。cron クレートでパース可能かどうかで判定する。
#[must_use] 
pub fn validate_cron(expr: &str) -> bool {
    let cron_6field = to_6field_cron(expr);
    cron::Schedule::from_str(&cron_6field).is_ok()
}

#[must_use] 
pub fn validate_timezone(timezone: &str) -> bool {
    parse_timezone(timezone).is_some()
}

#[must_use] 
pub fn parse_timezone(timezone: &str) -> Option<Tz> {
    timezone.parse::<Tz>().ok()
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_cron() {
        assert!(validate_cron("* * * * *"));
        assert!(validate_cron("0 12 * * 1"));
        assert!(validate_cron("30 6 1 1 *"));
    }

    #[test]
    fn invalid_cron() {
        assert!(!validate_cron("* * *"));
        assert!(!validate_cron(""));
    }

    #[test]
    fn next_run_at_returns_some() {
        let job = SchedulerJob::new(
            "test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        assert!(job.next_run_at().is_some());
    }
}
