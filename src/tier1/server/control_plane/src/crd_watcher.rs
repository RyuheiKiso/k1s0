// crd_watcher.rs — spec tier1 control_plane: Tier1Service CRD watcher
// kube 0.95 の runtime::watcher で Tier1Service CRD を watch し、conformance class の変更を検知する。
// 変更検知時に CrdWatchEvent を broadcast channel で配信する。

// anyhow: エラー伝搬ライブラリ
use anyhow::Result;
// futures_util: watcher stream を処理するための StreamExt
use futures_util::StreamExt;
// kube: Kubernetes API クライアント
use kube::{Api, Client, CustomResource};
// kube runtime: watcher + reflector
use kube::runtime::{watcher, WatchStreamExt};
// serde: CRD spec/status の JSON デシリアライズ
use serde::{Deserialize, Serialize};
// tokio: 非同期チャネル（broadcast）と shutdown signal
use tokio::sync::broadcast;
// tracing: 構造化ロギング
use tracing::{debug, error, info, warn};

// Tier1ServiceSpec は Tier1Service CRD の Spec を宣言する。
// src/tier1/operator/api/v1/types.go の Tier1ServiceSpec と同期する。
// kube::CustomResource derive で kube::Resource を自動実装する。
// schemars::JsonSchema は kube::CustomResource の derive マクロが要求する。
#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[kube(
    group = "k1s0.io",
    version = "v1",
    kind = "Tier1Service",
    namespaced,
    status = "Tier1ServiceStatus"
)]
pub struct Tier1ServiceSpec {
    // conformanceClass: 適用する Bidi conformance class（v1_interactive 等）
    #[serde(rename = "conformanceClass", default)]
    pub conformance_class: String,
    // adapter: 使用する transport adapter（grpc_native 等）
    #[serde(default)]
    pub adapter: String,
    // quotaClass: テナント容量クラス（v1_standard 等）
    #[serde(rename = "quotaClass", default)]
    pub quota_class: String,
    // sloClass: SLO クラス（v1_api 等）
    #[serde(rename = "sloClass", default)]
    pub slo_class: String,
    // replicas: Pod のレプリカ数（デフォルト 1）
    #[serde(default = "default_replicas")]
    pub replicas: i32,
}

// default_replicas はレプリカ数のデフォルト値を返す。
fn default_replicas() -> i32 {
    // デフォルトは 1 レプリカとする（spec に replicas が省略された場合）
    1
}

// Tier1ServiceStatus は Tier1Service CRD の Status を宣言する。
// schemars::JsonSchema は kube::CustomResource の derive マクロが要求する。
#[derive(Debug, Clone, Deserialize, Serialize, Default, schemars::JsonSchema)]
pub struct Tier1ServiceStatus {
    // conformanceStatus: conformance テストの結果（green / red / pending）
    #[serde(rename = "conformanceStatus", default = "default_pending")]
    pub conformance_status: String,
    // lastReconciledAt: 最後に Reconcile した日時（RFC 3339 形式）
    #[serde(rename = "lastReconciledAt", default)]
    pub last_reconciled_at: String,
}

// default_pending はステータスのデフォルト値を返す。
fn default_pending() -> String {
    // 初期状態は pending とする
    "pending".to_string()
}

// CrdWatchEvent は CRD の変更イベントを宣言する。
// broadcast channel で配信して他のサブシステム（flagd publisher 等）が購読できる。
#[derive(Debug, Clone)]
pub enum CrdWatchEvent {
    // Added: 新しい Tier1Service リソースが作成された
    Added {
        // Kubernetes namespace
        namespace: String,
        // リソース名
        name: String,
        // spec の内容
        spec: Tier1ServiceSpec,
    },
    // Modified: 既存の Tier1Service リソースが変更された
    Modified {
        // Kubernetes namespace
        namespace: String,
        // リソース名
        name: String,
        // 変更後の spec
        spec: Tier1ServiceSpec,
    },
    // Deleted: Tier1Service リソースが削除された
    Deleted {
        // Kubernetes namespace
        namespace: String,
        // リソース名
        name: String,
    },
}

// CrdWatcher は Tier1Service CRD を kube::runtime::watcher で watch する。
pub struct CrdWatcher {
    // channel_capacity: broadcast channel のバッファサイズ
    channel_capacity: usize,
}

impl CrdWatcher {
    // new は CrdWatcher を構築する。
    pub fn new() -> Self {
        Self {
            // broadcast channel のバッファサイズ（256 イベント）
            channel_capacity: 256,
        }
    }

    // watch は Tier1Service CRD を watch してイベントを broadcast channel で配信する。
    // 戻り値の Receiver で呼び出し元がイベントを購読できる。
    // shutdown_rx で graceful shutdown を受け付ける。
    pub async fn watch(
        &self,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<broadcast::Receiver<CrdWatchEvent>> {
        // broadcast channel を作成する（送信側と受信側を分離する）
        let (event_tx, event_rx) = broadcast::channel(self.channel_capacity);

        // kube::Client を in-cluster config または ~/.kube/config から構築する
        let client = match Client::try_default().await {
            Ok(c) => {
                // Kubernetes API への接続成功をログに記録する
                info!("kube::Client connected to Kubernetes API");
                c
            }
            Err(e) => {
                // Kubernetes 接続失敗時は警告を出してダミーループに入る
                warn!(
                    error = %e,
                    "kube::Client connection failed — CrdWatcher operating in degraded mode (no kubeconfig?)"
                );
                // 接続できない場合はイベントなしで watch を継続する（開発環境用）
                let event_tx_clone = event_tx.clone();
                tokio::spawn(async move {
                    // 30 秒ごとに接続再試行ログを出力する（実際の watch は行わない）
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        debug!("CrdWatcher: degraded mode heartbeat (no Kubernetes connection)");
                        // event_tx_clone は使用するが何も送信しない（接続回復まで待機）
                        let _ = event_tx_clone.receiver_count();
                    }
                });
                return Ok(event_rx);
            }
        };

        // Tier1Service の kube::Api を all namespaces で構築する
        let api: Api<Tier1Service> = Api::all(client.clone());

        // watcher の設定を構築する（fieldSelector / labelSelector は現在なし）
        let watcher_config = watcher::Config::default();

        // broadcast channel の送信側をクローンしてスポーンタスクに移動する
        let event_tx_clone = event_tx.clone();

        // watcher ループをバックグラウンドタスクとして起動する
        tokio::spawn(async move {
            // kube::runtime::watcher で watcher stream を構築して Box::pin で Pin する
            // Pin は select! マクロでストリームを借用するために必要
            let mut watcher_stream = Box::pin(watcher(
                Api::<Tier1Service>::all(client),
                watcher::Config::default(),
            ));
            // shutdown signal と watcher stream を select で処理する
            loop {
                tokio::select! {
                    // shutdown signal を受け取った場合はループを終了する
                    _ = shutdown_rx.recv() => {
                        info!("CrdWatcher: shutdown signal received");
                        break;
                    }
                    // watcher stream から次のイベントを取得する
                    Some(event_result) = watcher_stream.next() => {
                        match event_result {
                            // リソース追加・変更（Init 系も Apply として扱う）
                            Ok(watcher::Event::Apply(tier1_service)) => {
                                // namespace と name を取得する
                                let namespace = tier1_service
                                    .metadata
                                    .namespace
                                    .as_deref()
                                    .unwrap_or("default")
                                    .to_string();
                                let name = tier1_service
                                    .metadata
                                    .name
                                    .as_deref()
                                    .unwrap_or("unknown")
                                    .to_string();
                                // CrdWatchEvent を構築して broadcast する
                                let event = CrdWatchEvent::Modified {
                                    namespace: namespace.clone(),
                                    name: name.clone(),
                                    spec: tier1_service.spec,
                                };
                                info!(
                                    namespace = %namespace,
                                    name = %name,
                                    "CrdWatcher: Tier1Service Apply event"
                                );
                                // broadcast channel にイベントを送信する（受信者がいない場合は無視する）
                                let _ = event_tx_clone.send(event);
                            }
                            // リソース削除
                            Ok(watcher::Event::Delete(tier1_service)) => {
                                // 削除対象のリソース情報を取得する
                                let namespace = tier1_service
                                    .metadata
                                    .namespace
                                    .as_deref()
                                    .unwrap_or("default")
                                    .to_string();
                                let name = tier1_service
                                    .metadata
                                    .name
                                    .as_deref()
                                    .unwrap_or("unknown")
                                    .to_string();
                                // 削除イベントを broadcast する
                                let event = CrdWatchEvent::Deleted {
                                    namespace: namespace.clone(),
                                    name: name.clone(),
                                };
                                info!(
                                    namespace = %namespace,
                                    name = %name,
                                    "CrdWatcher: Tier1Service Delete event"
                                );
                                let _ = event_tx_clone.send(event);
                            }
                            // watcher stream の初期化ページング完了（全 list 取得済み）
                            Ok(watcher::Event::InitApply(tier1_service)) => {
                                // 初期化時のリソースを Added イベントとして扱う
                                let namespace = tier1_service
                                    .metadata
                                    .namespace
                                    .as_deref()
                                    .unwrap_or("default")
                                    .to_string();
                                let name = tier1_service
                                    .metadata
                                    .name
                                    .as_deref()
                                    .unwrap_or("unknown")
                                    .to_string();
                                let event = CrdWatchEvent::Added {
                                    namespace: namespace.clone(),
                                    name: name.clone(),
                                    spec: tier1_service.spec,
                                };
                                debug!(
                                    namespace = %namespace,
                                    name = %name,
                                    "CrdWatcher: Tier1Service InitApply (initial list)"
                                );
                                let _ = event_tx_clone.send(event);
                            }
                            // InitDone: 初期リスト取得完了
                            Ok(watcher::Event::InitDone) => {
                                info!("CrdWatcher: initial list complete (InitDone)");
                            }
                            // Init: watcher 開始
                            Ok(watcher::Event::Init) => {
                                debug!("CrdWatcher: watcher initialized (Init)");
                            }
                            // watcher エラー（API server 一時障害等）は警告を出して続行する
                            Err(e) => {
                                warn!(error = %e, "CrdWatcher: watcher error, will retry");
                                // エラー後は少し待機してから再試行する
                                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                            }
                        }
                    }
                    // watcher stream が終了した場合はループを終了する
                    else => {
                        info!("CrdWatcher: watcher stream ended");
                        break;
                    }
                }
            }
            info!("CrdWatcher: watch loop terminated");
        });

        // 呼び出し元が購読できる broadcast Receiver を返す
        Ok(event_rx)
    }
}

// Default implementation for CrdWatcher
impl Default for CrdWatcher {
    fn default() -> Self {
        // new() と同じ設定でデフォルトを構築する
        Self::new()
    }
}
