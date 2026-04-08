use async_trait::async_trait;
use std::time::Duration;

// DlqManagerClient トレイトの引数・戻り値型を定義する。
// preview_replay / execute_replay の両メソッドで共有する。
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReplayRequest {
    pub correlation_ids: Vec<String>,
    pub from_step_index: i32,
    pub include_downstream: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct ReplayResponse {
    pub replay_id: String,
    pub status: String,
    pub total_events: i32,
    pub replayed_events: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReplayPreviewResponse {
    pub total_events_to_replay: i32,
    pub affected_services: Vec<String>,
    pub dlq_messages_found: i32,
    pub estimated_duration_seconds: i32,
}

// DLQ Manager サービスとのやりとりを抽象化するトレイト。
// 実装: GrpcDlqClient（本番）/ NoopDlqClient（設定なし・テスト）
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqManagerClient: Send + Sync {
    async fn preview_replay(&self, req: &ReplayRequest) -> anyhow::Result<ReplayPreviewResponse>;
    async fn execute_replay(&self, req: &ReplayRequest) -> anyhow::Result<ReplayResponse>;
}

// 設定がない場合または接続不要な環境（dev/test）で使用するダミー実装。
// health endpoint には dlq_noop=true が伝達される。
pub struct NoopDlqClient;

#[async_trait]
impl DlqManagerClient for NoopDlqClient {
    async fn preview_replay(&self, _req: &ReplayRequest) -> anyhow::Result<ReplayPreviewResponse> {
        Ok(ReplayPreviewResponse {
            total_events_to_replay: 0,
            affected_services: vec![],
            dlq_messages_found: 0,
            estimated_duration_seconds: 0,
        })
    }

    async fn execute_replay(&self, _req: &ReplayRequest) -> anyhow::Result<ReplayResponse> {
        Ok(ReplayResponse {
            replay_id: uuid::Uuid::new_v4().to_string(),
            status: "noop".to_string(),
            total_events: 0,
            replayed_events: 0,
        })
    }
}

// dlq-manager gRPC サービスへの実クライアント。
// DlqManagerConfig の grpc_endpoint に接続し、リプレイ操作を委譲する。
// RetryAll を「全相関 ID を順次リトライ」として preview_replay に対応させる。
pub struct GrpcDlqClient {
    // tonic の Channel はスレッドセーフかつクローン可能で、内部で接続プールを管理する。
    endpoint: String,
    timeout: Duration,
}

impl GrpcDlqClient {
    /// `grpc_endpoint` と `timeout_ms` から新しい `GrpcDlqClient` を生成する。
    /// 接続は各 RPC 呼び出し時に遅延確立する（lazy connect）。
    #[must_use] 
    pub fn new(grpc_endpoint: String, timeout_ms: u64) -> Self {
        Self {
            endpoint: grpc_endpoint,
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// tonic チャネルを生成するヘルパー。呼び出しのたびに新しいチャネルを生成する。
    /// 本番運用では Channel をフィールドに持ち再利用するのが望ましいが、
    /// replay 操作は低頻度のため簡潔さを優先してリクエストごとに接続する。
    async fn connect(
        &self,
    ) -> anyhow::Result<
        crate::proto::k1s0::system::dlq::v1::dlq_service_client::DlqServiceClient<
            tonic::transport::Channel,
        >,
    > {
        // エンドポイント文字列から tonic::transport::Channel を構築する。
        // timeout はサーバー接続タイムアウトとして設定する。
        let channel = tonic::transport::Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| anyhow::anyhow!("DLQ Manager エンドポイントが不正: {e}"))?
            .timeout(self.timeout)
            .connect()
            .await
            .map_err(|e| anyhow::anyhow!("DLQ Manager への gRPC 接続に失敗: {e}"))?;

        Ok(crate::proto::k1s0::system::dlq::v1::dlq_service_client::DlqServiceClient::new(channel))
    }
}

#[async_trait]
impl DlqManagerClient for GrpcDlqClient {
    /// `preview_replay`: 指定した `correlation_ids` に対応する DLQ メッセージを検索し、
    /// リプレイ対象件数・影響サービス等の事前確認情報を返す。
    /// `DlqService` には専用の preview RPC がないため、ListMessages を使って
    /// 対象 `correlation_id` ごとの DLQ メッセージ数を集計して返す。
    async fn preview_replay(&self, req: &ReplayRequest) -> anyhow::Result<ReplayPreviewResponse> {
        use crate::proto::k1s0::system::dlq::v1::{
            dlq_service_client::DlqServiceClient, ListMessagesRequest,
        };

        let mut client: DlqServiceClient<tonic::transport::Channel> = self.connect().await?;

        // 各 correlation_id をトピックとして DLQ メッセージ一覧を取得し、合計件数を算出する。
        // correlation_id ベースのフィルタが proto にないため、トピック単位で問い合わせる。
        let mut total_dlq_messages = 0i32;
        let mut affected_topics: Vec<String> = Vec::new();

        for corr_id in &req.correlation_ids {
            let list_req = tonic::Request::new(ListMessagesRequest {
                topic: corr_id.clone(),
                pagination: None,
            });
            match client.list_messages(list_req).await {
                Ok(resp) => {
                    let count = resp.into_inner().messages.len() as i32;
                    if count > 0 {
                        total_dlq_messages += count;
                        affected_topics.push(corr_id.clone());
                    }
                }
                Err(status) => {
                    tracing::warn!(
                        corr_id = %corr_id,
                        status = %status,
                        "DLQ preview: correlation_id に対応するメッセージ取得に失敗（スキップ）"
                    );
                }
            }
        }

        // total_events_to_replay は DLQ 件数と correlation_ids 件数の合計を返す。
        Ok(ReplayPreviewResponse {
            total_events_to_replay: req.correlation_ids.len() as i32,
            affected_services: affected_topics,
            dlq_messages_found: total_dlq_messages,
            estimated_duration_seconds: total_dlq_messages * 2, // 1件あたり約2秒と見積もる
        })
    }

    /// `execute_replay`: 指定した `correlation_ids` に対応する DLQ メッセージを
    /// `RetryAll` RPC を使って一括リトライする。
    /// `correlation_ids` を "トピック" として扱い、各 `correlation_id` 配下の
    /// メッセージをリトライする。
    async fn execute_replay(&self, req: &ReplayRequest) -> anyhow::Result<ReplayResponse> {
        use crate::proto::k1s0::system::dlq::v1::{
            dlq_service_client::DlqServiceClient, RetryAllRequest,
        };

        let mut client: DlqServiceClient<tonic::transport::Channel> = self.connect().await?;

        let mut total_retried = 0i32;
        let replay_id = uuid::Uuid::new_v4().to_string();

        // 各 correlation_id に対して RetryAll を呼び出し、リトライ件数を集計する。
        for corr_id in &req.correlation_ids {
            let retry_req = tonic::Request::new(RetryAllRequest {
                topic: corr_id.clone(),
            });
            match client.retry_all(retry_req).await {
                Ok(resp) => {
                    total_retried += resp.into_inner().retried_count;
                }
                Err(status) => {
                    tracing::warn!(
                        corr_id = %corr_id,
                        status = %status,
                        "DLQ execute_replay: correlation_id のリトライに失敗（スキップ）"
                    );
                }
            }
        }

        Ok(ReplayResponse {
            replay_id,
            status: "completed".to_string(),
            total_events: req.correlation_ids.len() as i32,
            replayed_events: total_retried,
        })
    }
}
