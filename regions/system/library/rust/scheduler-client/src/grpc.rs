use crate::client::SchedulerClient;
use crate::error::SchedulerError;
use crate::job::{Job, JobExecution, JobFilter, JobRequest, JobStatus, Schedule};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ---- HTTP 用 JSON 中間型 ----------------------------------------

/// scheduler-server との JSON 交換用スケジュール形式。
/// Go 実装の scheduleJSON と対応。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduleJson {
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cron: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    one_shot: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interval_secs: Option<i64>,
}

/// scheduler-server へのジョブ登録リクエスト形式。
#[derive(Debug, Serialize)]
struct JobRequestJson {
    name: String,
    schedule: ScheduleJson,
    payload: serde_json::Value,
    max_retries: u32,
    timeout_secs: u64,
}

/// scheduler-server のジョブレスポンス形式。
#[derive(Debug, Deserialize)]
struct JobResponseJson {
    id: String,
    name: String,
    schedule: ScheduleJson,
    status: String,
    payload: serde_json::Value,
    max_retries: u32,
    timeout_secs: u64,
    created_at: DateTime<Utc>,
    next_run_at: Option<DateTime<Utc>>,
}

/// scheduler-server の実行履歴レスポンス形式。
#[derive(Debug, Deserialize)]
struct JobExecutionResponseJson {
    id: String,
    job_id: String,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    result: String,
    #[serde(default)]
    error: Option<String>,
}

// ---- 変換ヘルパー ------------------------------------------------

fn to_schedule_json(s: &Schedule) -> ScheduleJson {
    match s {
        Schedule::Cron(expr) => ScheduleJson {
            kind: "cron".to_string(),
            cron: Some(expr.clone()),
            one_shot: None,
            interval_secs: None,
        },
        Schedule::OneShot(at) => ScheduleJson {
            kind: "one_shot".to_string(),
            cron: None,
            one_shot: Some(*at),
            interval_secs: None,
        },
        Schedule::Interval(d) => ScheduleJson {
            kind: "interval".to_string(),
            cron: None,
            one_shot: None,
            interval_secs: Some(d.as_secs() as i64),
        },
    }
}

fn from_schedule_json(sj: ScheduleJson) -> Result<Schedule, SchedulerError> {
    match sj.kind.as_str() {
        "cron" => {
            let expr = sj.cron.ok_or_else(|| {
                SchedulerError::InvalidSchedule("cron フィールドがありません".to_string())
            })?;
            Ok(Schedule::Cron(expr))
        }
        "one_shot" => {
            let at = sj.one_shot.ok_or_else(|| {
                SchedulerError::InvalidSchedule("one_shot フィールドがありません".to_string())
            })?;
            Ok(Schedule::OneShot(at))
        }
        "interval" => {
            let secs = sj.interval_secs.ok_or_else(|| {
                SchedulerError::InvalidSchedule("interval_secs フィールドがありません".to_string())
            })?;
            Ok(Schedule::Interval(std::time::Duration::from_secs(
                secs as u64,
            )))
        }
        other => Err(SchedulerError::InvalidSchedule(format!(
            "不明なスケジュール種別: {}",
            other
        ))),
    }
}

fn job_status_to_str(s: &JobStatus) -> &'static str {
    match s {
        JobStatus::Pending => "pending",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Paused => "paused",
        JobStatus::Cancelled => "cancelled",
    }
}

fn job_status_from_str(s: &str) -> JobStatus {
    match s {
        "pending" => JobStatus::Pending,
        "running" => JobStatus::Running,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        "paused" => JobStatus::Paused,
        "cancelled" => JobStatus::Cancelled,
        _ => JobStatus::Pending,
    }
}

fn from_job_response(r: JobResponseJson) -> Result<Job, SchedulerError> {
    let schedule = from_schedule_json(r.schedule)?;
    Ok(Job {
        id: r.id,
        name: r.name,
        schedule,
        status: job_status_from_str(&r.status),
        payload: r.payload,
        max_retries: r.max_retries,
        timeout_secs: r.timeout_secs,
        created_at: r.created_at,
        next_run_at: r.next_run_at,
    })
}

// ---- エラーハンドリング ------------------------------------------

fn map_reqwest_err(e: reqwest::Error) -> SchedulerError {
    if e.is_timeout() {
        SchedulerError::Timeout
    } else {
        SchedulerError::ServerError(e.to_string())
    }
}

async fn check_response(
    resp: reqwest::Response,
    id: &str,
) -> Result<reqwest::Response, SchedulerError> {
    if resp.status().as_u16() == 404 {
        return Err(SchedulerError::JobNotFound(id.to_string()));
    }
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SchedulerError::ServerError(format!("{}: {}", status, body)));
    }
    Ok(resp)
}

// ---- GrpcSchedulerClient ----------------------------------------

/// scheduler-server への HTTP REST クライアント。
/// 名称は Go 実装の GrpcSchedulerClient に合わせているが、
/// 実際には HTTP REST API を使用する。
pub struct GrpcSchedulerClient {
    http: Client,
    base_url: String,
}

impl GrpcSchedulerClient {
    /// 新しい GrpcSchedulerClient を生成する。
    /// `server_url` は `"http://scheduler-server:8080"` または `"scheduler-server:8080"` の形式。
    pub async fn new(server_url: impl Into<String>) -> Result<Self, SchedulerError> {
        let mut base = server_url.into();
        if !base.starts_with("http://") && !base.starts_with("https://") {
            base = format!("http://{}", base);
        }
        let base = base.trim_end_matches('/').to_string();
        Ok(Self {
            http: Client::new(),
            base_url: base,
        })
    }
}

#[async_trait]
impl SchedulerClient for GrpcSchedulerClient {
    async fn create_job(&self, req: JobRequest) -> Result<Job, SchedulerError> {
        let body = JobRequestJson {
            name: req.name,
            schedule: to_schedule_json(&req.schedule),
            payload: req.payload,
            max_retries: req.max_retries,
            timeout_secs: req.timeout_secs,
        };

        let resp = self
            .http
            .post(format!("{}/api/v1/jobs", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(map_reqwest_err)?;

        let resp = check_response(resp, "create_job").await?;
        let result: JobResponseJson = resp.json().await.map_err(map_reqwest_err)?;
        from_job_response(result)
    }

    async fn cancel_job(&self, job_id: &str) -> Result<(), SchedulerError> {
        let path = format!(
            "{}/api/v1/jobs/{}/cancel",
            self.base_url,
            urlencoding::encode(job_id)
        );
        let resp = self
            .http
            .post(&path)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(map_reqwest_err)?;

        check_response(resp, job_id).await?;
        Ok(())
    }

    async fn pause_job(&self, job_id: &str) -> Result<(), SchedulerError> {
        let path = format!(
            "{}/api/v1/jobs/{}/pause",
            self.base_url,
            urlencoding::encode(job_id)
        );
        let resp = self
            .http
            .post(&path)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(map_reqwest_err)?;

        check_response(resp, job_id).await?;
        Ok(())
    }

    async fn resume_job(&self, job_id: &str) -> Result<(), SchedulerError> {
        let path = format!(
            "{}/api/v1/jobs/{}/resume",
            self.base_url,
            urlencoding::encode(job_id)
        );
        let resp = self
            .http
            .post(&path)
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(map_reqwest_err)?;

        check_response(resp, job_id).await?;
        Ok(())
    }

    async fn get_job(&self, job_id: &str) -> Result<Job, SchedulerError> {
        let path = format!(
            "{}/api/v1/jobs/{}",
            self.base_url,
            urlencoding::encode(job_id)
        );
        let resp = self.http.get(&path).send().await.map_err(map_reqwest_err)?;

        let resp = check_response(resp, job_id).await?;
        let result: JobResponseJson = resp.json().await.map_err(map_reqwest_err)?;
        from_job_response(result)
    }

    async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<Job>, SchedulerError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(ref status) = filter.status {
            query.push(("status", job_status_to_str(status).to_string()));
        }
        if let Some(ref prefix) = filter.name_prefix {
            if !prefix.is_empty() {
                query.push(("name_prefix", prefix.clone()));
            }
        }

        let resp = self
            .http
            .get(format!("{}/api/v1/jobs", self.base_url))
            .query(&query)
            .send()
            .await
            .map_err(map_reqwest_err)?;

        let resp = check_response(resp, "list_jobs").await?;
        let results: Vec<JobResponseJson> = resp.json().await.map_err(map_reqwest_err)?;

        results.into_iter().map(from_job_response).collect()
    }

    async fn get_executions(&self, job_id: &str) -> Result<Vec<JobExecution>, SchedulerError> {
        let path = format!(
            "{}/api/v1/jobs/{}/executions",
            self.base_url,
            urlencoding::encode(job_id)
        );
        let resp = self.http.get(&path).send().await.map_err(map_reqwest_err)?;

        let resp = check_response(resp, job_id).await?;
        let results: Vec<JobExecutionResponseJson> = resp.json().await.map_err(map_reqwest_err)?;

        Ok(results
            .into_iter()
            .map(|r| JobExecution {
                id: r.id,
                job_id: r.job_id,
                started_at: r.started_at,
                finished_at: r.finished_at,
                result: r.result,
                error: r.error,
            })
            .collect())
    }
}
