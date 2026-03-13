use async_graphql::SimpleObject;

/// スケジュールジョブ
#[derive(Debug, Clone, SimpleObject)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: String,
    pub status: String,
    pub next_run_at: Option<String>,
    pub last_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// ジョブ実行履歴
#[derive(Debug, Clone, SimpleObject)]
pub struct JobExecution {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_by: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
}
