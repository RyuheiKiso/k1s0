// 本ファイルは k1s0-sdk の Client 型と接続管理を提供する。
//
// 利用例:
//   let cfg = Config { target: "http://tier1-state.tier1-facade.svc.cluster.local:50001".into(),
//                       tenant_id: "tenant-A".into(), subject: "svc-foo".into() };
//   let client = Client::connect(cfg).await?;
//   let resp = client.state().get("valkey-default", "user/123").await?;

use crate::proto::k1s0::tier1::common::v1::TenantContext;
use crate::proto::k1s0::tier1::state::v1::state_service_client::StateServiceClient;
use crate::proto::k1s0::tier1::pubsub::v1::pub_sub_service_client::PubSubServiceClient;
use crate::proto::k1s0::tier1::secrets::v1::secrets_service_client::SecretsServiceClient;
use tonic::transport::{Channel, Endpoint};

// Config は Client 初期化時に渡す設定。
#[derive(Debug, Clone)]
pub struct Config {
    /// gRPC 接続先（例: "http://tier1-state.tier1-facade.svc.cluster.local:50001"）。
    pub target: String,
    /// テナント ID（全 RPC の TenantContext.tenant_id に自動付与）。
    pub tenant_id: String,
    /// 主体識別子（subject）。
    pub subject: String,
}

// Client は 12 service へのアクセス起点。
#[derive(Debug, Clone)]
pub struct Client {
    /// gRPC channel（tonic の Channel は Clone 可能、内部で接続プール共有）。
    channel: Channel,
    /// 自動付与する TenantContext 情報。
    cfg: Config,
}

impl Client {
    /// Config から Client を生成する。target に対する gRPC 接続を確立する。
    pub async fn connect(cfg: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Endpoint を構築して接続する。
        let endpoint = Endpoint::from_shared(cfg.target.clone())?;
        // tonic Channel を確立する（プール共有、Clone 安価）。
        let channel = endpoint.connect().await?;
        // Client を返却する。
        Ok(Self { channel, cfg })
    }

    /// Client が想定する Config を返す（読取専用）。
    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// State 動詞統一 facade。
    pub fn state(&self) -> crate::state::StateFacade {
        // facade は Channel と Config を共有する Clone を保持する。
        crate::state::StateFacade::new(self.clone())
    }

    /// PubSub 動詞統一 facade。
    pub fn pubsub(&self) -> crate::pubsub::PubSubFacade {
        // 同上。
        crate::pubsub::PubSubFacade::new(self.clone())
    }

    /// Secrets 動詞統一 facade。
    pub fn secrets(&self) -> crate::secrets::SecretsFacade {
        // 同上。
        crate::secrets::SecretsFacade::new(self.clone())
    }

    /// 動詞統一 facade が未実装の service にアクセスする際の生成 stub クライアント。
    /// 例: client.raw_state() で StateServiceClient<Channel> を直接取得。
    pub fn raw_state(&self) -> StateServiceClient<Channel> {
        StateServiceClient::new(self.channel.clone())
    }

    /// 同上 PubSubService の raw client。
    pub fn raw_pubsub(&self) -> PubSubServiceClient<Channel> {
        PubSubServiceClient::new(self.channel.clone())
    }

    /// 同上 SecretsService の raw client。
    pub fn raw_secrets(&self) -> SecretsServiceClient<Channel> {
        SecretsServiceClient::new(self.channel.clone())
    }

    /// 内部用: 親 Client の Channel を facade に共有する。
    pub(crate) fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// 内部用: 親 Client の Config から TenantContext proto を生成する。
    pub(crate) fn tenant_context(&self) -> TenantContext {
        TenantContext {
            tenant_id: self.cfg.tenant_id.clone(),
            subject: self.cfg.subject.clone(),
            correlation_id: String::new(),
        }
    }
}
