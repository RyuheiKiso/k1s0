use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{Job, JobExecution};
use crate::infrastructure::config::BackendConfig;

#[allow(dead_code)]
pub mod proto {
    // buf generate の compile_well_known_types オプションで生成された .rs ファイルは
    // google::protobuf::Struct を super チェーンで参照する（例: super*4::google::protobuf::Struct）。
    // prost_types クレートが提供する型を google::protobuf 名前空間として再エクスポートし、
    // include_proto! 展開後のパス解決を正常に機能させる。
    pub mod google {
        pub mod protobuf {
            pub use ::prost_types::*;
        }
    }
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod scheduler {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.scheduler.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::scheduler::v1::scheduler_service_client::SchedulerServiceClient;

pub struct SchedulerGrpcClient {
    client: SchedulerServiceClient<Channel>,
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// タイムアウト設定（ミリ秒）。health_check のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl SchedulerGrpcClient {
    fn job_from_proto(j: proto::k1s0::system::scheduler::v1::Job) -> Job {
        Job {
            id: j.id,
            name: j.name,
            description: j.description,
            cron_expression: j.cron_expression,
            timezone: j.timezone,
            target_type: j.target_type,
            target: j.target,
            status: j.status,
            next_run_at: optional_timestamp_to_rfc3339(j.next_run_at),
            last_run_at: optional_timestamp_to_rfc3339(j.last_run_at),
            created_at: timestamp_to_rfc3339(j.created_at),
            updated_at: timestamp_to_rfc3339(j.updated_at),
        }
    }

    fn execution_from_proto(e: proto::k1s0::system::scheduler::v1::JobExecution) -> JobExecution {
        JobExecution {
            id: e.id,
            job_id: e.job_id,
            status: e.status,
            triggered_by: e.triggered_by,
            started_at: timestamp_to_rfc3339(e.started_at),
            finished_at: optional_timestamp_to_rfc3339(e.finished_at),
            duration_ms: e.duration_ms,
            error_message: e.error_message.filter(|s| !s.is_empty()),
        }
    }

    /// バックエンド設定からクライアントを生成する。
    /// connect_lazy() により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: SchedulerServiceClient::new(channel),
            address: cfg.address.clone(),
            timeout_ms: cfg.timeout_ms,
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_job(&self, job_id: &str) -> anyhow::Result<Option<Job>> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::GetJobRequest {
            job_id: job_id.to_owned(),
        });

        match self.client.clone().get_job(request).await {
            Ok(resp) => {
                let j = match resp.into_inner().job {
                    Some(j) => j,
                    None => return Ok(None),
                };
                Ok(Some(Self::job_from_proto(j)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("SchedulerService.GetJob failed: {}", e)),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_jobs(
        &self,
        status: Option<&str>,
        page_size: Option<i32>,
        page: Option<i32>,
    ) -> anyhow::Result<Vec<Job>> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::ListJobsRequest {
            status: status.unwrap_or_default().to_owned(),
            pagination: Some(proto::k1s0::system::common::v1::Pagination {
                page: page.unwrap_or(1),
                page_size: page_size.unwrap_or(50),
            }),
        });

        let resp = self
            .client
            .clone()
            .list_jobs(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.ListJobs failed: {}", e))?
            .into_inner();

        Ok(resp.jobs.into_iter().map(Self::job_from_proto).collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_job(
        &self,
        name: &str,
        description: &str,
        cron_expression: &str,
        timezone: &str,
        target_type: &str,
        target: &str,
    ) -> anyhow::Result<Job> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::CreateJobRequest {
            name: name.to_owned(),
            description: description.to_owned(),
            cron_expression: cron_expression.to_owned(),
            timezone: timezone.to_owned(),
            target_type: target_type.to_owned(),
            target: target.to_owned(),
            payload: None,
        });

        let j = self
            .client
            .clone()
            .create_job(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.CreateJob failed: {}", e))?
            .into_inner()
            .job
            .ok_or_else(|| anyhow::anyhow!("empty job in response"))?;

        Ok(Self::job_from_proto(j))
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_job(
        &self,
        job_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        cron_expression: Option<&str>,
        timezone: Option<&str>,
        target_type: Option<&str>,
        target: Option<&str>,
    ) -> anyhow::Result<Job> {
        let current = self
            .client
            .clone()
            .get_job(tonic::Request::new(
                proto::k1s0::system::scheduler::v1::GetJobRequest {
                    job_id: job_id.to_owned(),
                },
            ))
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.GetJob failed: {}", e))?
            .into_inner()
            .job
            .ok_or_else(|| anyhow::anyhow!("job not found: {}", job_id))?;

        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::UpdateJobRequest {
            job_id: job_id.to_owned(),
            name: name.map(|s| s.to_owned()).unwrap_or(current.name),
            description: description
                .map(|s| s.to_owned())
                .unwrap_or(current.description),
            cron_expression: cron_expression
                .map(|s| s.to_owned())
                .unwrap_or(current.cron_expression),
            timezone: timezone.map(|s| s.to_owned()).unwrap_or(current.timezone),
            target_type: target_type
                .map(|s| s.to_owned())
                .unwrap_or(current.target_type),
            target: target.map(|s| s.to_owned()).unwrap_or(current.target),
            payload: None,
        });

        let j = self
            .client
            .clone()
            .update_job(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.UpdateJob failed: {}", e))?
            .into_inner()
            .job
            .ok_or_else(|| anyhow::anyhow!("empty job in update response"))?;

        Ok(Self::job_from_proto(j))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_job(&self, job_id: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::DeleteJobRequest {
            job_id: job_id.to_owned(),
        });

        let resp = self
            .client
            .clone()
            .delete_job(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.DeleteJob failed: {}", e))?
            .into_inner();

        Ok(resp.success)
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn pause_job(&self, job_id: &str) -> anyhow::Result<Job> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::PauseJobRequest {
            job_id: job_id.to_owned(),
        });

        let j = self
            .client
            .clone()
            .pause_job(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.PauseJob failed: {}", e))?
            .into_inner()
            .job
            .ok_or_else(|| anyhow::anyhow!("empty job in pause response"))?;

        Ok(Self::job_from_proto(j))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn resume_job(&self, job_id: &str) -> anyhow::Result<Job> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::ResumeJobRequest {
            job_id: job_id.to_owned(),
        });

        let j = self
            .client
            .clone()
            .resume_job(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.ResumeJob failed: {}", e))?
            .into_inner()
            .job
            .ok_or_else(|| anyhow::anyhow!("empty job in resume response"))?;

        Ok(Self::job_from_proto(j))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn trigger_job(
        &self,
        job_id: &str,
    ) -> anyhow::Result<(String, String, String, String)> {
        let request = tonic::Request::new(proto::k1s0::system::scheduler::v1::TriggerJobRequest {
            job_id: job_id.to_owned(),
        });

        let resp = self
            .client
            .clone()
            .trigger_job(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.TriggerJob failed: {}", e))?
            .into_inner();

        Ok((
            resp.execution_id,
            resp.job_id,
            resp.status,
            timestamp_to_rfc3339(resp.triggered_at),
        ))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_job_execution(
        &self,
        execution_id: &str,
    ) -> anyhow::Result<Option<JobExecution>> {
        let request =
            tonic::Request::new(proto::k1s0::system::scheduler::v1::GetJobExecutionRequest {
                execution_id: execution_id.to_owned(),
            });

        match self.client.clone().get_job_execution(request).await {
            Ok(resp) => {
                let e = match resp.into_inner().execution {
                    Some(e) => e,
                    None => return Ok(None),
                };
                Ok(Some(Self::execution_from_proto(e)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "SchedulerService.GetJobExecution failed: {}",
                e
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_executions(
        &self,
        job_id: &str,
        page_size: Option<i32>,
        page: Option<i32>,
        status: Option<&str>,
    ) -> anyhow::Result<Vec<JobExecution>> {
        let request =
            tonic::Request::new(proto::k1s0::system::scheduler::v1::ListExecutionsRequest {
                job_id: job_id.to_owned(),
                pagination: Some(proto::k1s0::system::common::v1::Pagination {
                    page: page.unwrap_or(1),
                    page_size: page_size.unwrap_or(50),
                }),
                status: status.map(|s| s.to_owned()),
                from: None,
                to: None,
            });

        let resp = self
            .client
            .clone()
            .list_executions(request)
            .await
            .map_err(|e| anyhow::anyhow!("SchedulerService.ListExecutions failed: {}", e))?
            .into_inner();

        Ok(resp
            .executions
            .into_iter()
            .map(Self::execution_from_proto)
            .collect())
    }

    /// gRPC Health Check Protocol を使ってサービスの疎通確認を行う。
    /// Bearer token なしで接続できるため readyz ヘルスチェックに適している。
    /// tonic-health サービスが登録されているサーバーに対して Check RPC を送信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let channel = Channel::from_shared(self.address.clone())?
            .timeout(Duration::from_millis(self.timeout_ms))
            .connect_lazy();
        let mut health_client = tonic_health::pb::health_client::HealthClient::new(channel);
        let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
            service: "k1s0.system.scheduler.v1.SchedulerService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("scheduler gRPC Health Check 失敗: {}", e))?;
        Ok(())
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

fn optional_timestamp_to_rfc3339(
    ts: Option<proto::k1s0::system::common::v1::Timestamp>,
) -> Option<String> {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
}
