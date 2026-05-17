// crd_watcher.rs — spec tier1 control_plane: Tier1Service CRD watcher
// kube 0.95 で Tier1Service CRD を watch し、conformance class の変更を検知する。
// 変更検知時に flagd_publisher.rs を呼び出してテナント別 feature flag を配布する。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// Tier1ServiceSpec は Tier1Service CRD の Spec を宣言する。
// src/tier1/operator/api/v1/types.go の Tier1ServiceSpec と同期する。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tier1ServiceSpec {
    // conformanceClass: 適用する Bidi conformance class
    #[serde(rename = "conformanceClass")]
    pub conformance_class: String,
    // adapter: 使用する transport adapter
    pub adapter: String,
    // quotaClass: テナント容量クラス
    #[serde(rename = "quotaClass")]
    pub quota_class: String,
    // sloClass: SLO クラス
    #[serde(rename = "sloClass")]
    pub slo_class: String,
    // replicas: Pod のレプリカ数
    pub replicas: i32,
}

// Tier1ServiceStatus は Tier1Service CRD の Status を宣言する。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Tier1ServiceStatus {
    // conformanceStatus: conformance テストの結果（green / red / pending）
    #[serde(rename = "conformanceStatus")]
    pub conformance_status: String,
    // lastReconciledAt: 最後に Reconcile した日時
    #[serde(rename = "lastReconciledAt")]
    pub last_reconciled_at: String,
}

// CrdWatchEvent は CRD の変更イベントを宣言する。
#[derive(Debug, Clone)]
pub enum CrdWatchEvent {
    // Added: 新しい Tier1Service リソースが作成された
    Added { namespace: String, name: String, spec: Tier1ServiceSpec },
    // Modified: 既存の Tier1Service リソースが変更された
    Modified { namespace: String, name: String, spec: Tier1ServiceSpec },
    // Deleted: Tier1Service リソースが削除された
    Deleted { namespace: String, name: String },
}

// CrdWatcher は Tier1Service CRD を watch する。
pub struct CrdWatcher {
    // kubeconfig: Kubernetes API server の接続設定（デフォルト: in-cluster config）
    kubeconfig: String,
}

impl CrdWatcher {
    // new は CrdWatcher を構築する。
    pub fn new(kubeconfig: String) -> Self {
        Self { kubeconfig }
    }

    // watch は Tier1Service CRD を watch してイベントを処理する。
    // kube 0.95 の watcher API を使って変更を検知し、flagd_publisher に通知する。
    pub async fn watch<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(CrdWatchEvent) + Send + 'static,
    {
        info!(
            kubeconfig = %self.kubeconfig,
            "CrdWatcher starting (kube watch API)"
        );
        // TODO: kube::Client::try_default() で Kubernetes API に接続する
        // let client = kube::Client::try_default().await?;
        // TODO: kube::Api::all() で Tier1Service を watch する
        // let api: kube::Api<Tier1Service> = kube::Api::all(client);
        // TODO: kube::runtime::watcher で変更イベントを処理する
        // let mut stream = kube::runtime::watcher(api, kube::runtime::watcher::Config::default());
        // while let Some(event) = stream.next().await { ... }
        info!("CrdWatcher: kube integration pending (kube 0.95 + controller-runtime)");
        // 現時点では定期的にダミーイベントをログに記録する
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            debug!("CrdWatcher: watch tick (pending kube API integration)");
        }
    }
}
