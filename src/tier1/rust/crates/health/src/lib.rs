//! tier1 Rust Pod 共通の HealthService 実装。
//!
//! 設計正典:
//! - `src/contracts/tier1/k1s0/tier1/health/v1/health_service.proto`
//! - `docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md`
//!
//! 役割:
//! - Liveness: process の生存と version / uptime を返す。依存先は見ない。
//! - Readiness: 依存先 probe を並列実行し、全件 reachable=true なら ready=true を返す。
//!
//! tier1 Go 側 `internal/health/` と同セマンティクスで実装する（同 proto から生成した
//! 同 RPC 定義に対する Rust 版）。

// 標準 / 外部 crate を import する。
// std::time でアップタイム計算する。
use std::sync::Arc;
// 起動時刻保持。
use std::time::Instant;

// 並列実行用に async_trait + future の合成を使う。
use async_trait::async_trait;
// 依存先 probe を並列実行する future 群。
use tokio::task::JoinSet;

// SDK 生成 proto 型（HealthServiceServer trait + Request/Response struct + DependencyStatus）。
// 経路: k1s0-sdk-proto → k1s0::tier1::health::v1。
use k1s0_sdk_proto::k1s0::tier1::health::v1::{
    DependencyStatus, LivenessRequest, LivenessResponse, ReadinessRequest, ReadinessResponse,
    health_service_server::HealthService,
};

// 依存先到達性検査の trait。Pod 起動時に実装を渡してもらう。
// async-trait で trait method の async 化を可能にする（tonic への Box<dyn> 化と整合）。
#[async_trait]
pub trait DependencyProbe: Send + Sync {
    // 依存先論理名（"openbao" / "valkey" / "kafka" 等）。Readiness の dependencies map のキー。
    fn name(&self) -> &str;
    // 到達性検査本体。Ok(()) なら reachable、Err なら error_message に詰める。
    async fn check(&self) -> Result<(), String>;
}

// HealthService 実装。Pod 起動時に version とprobeリストを渡して構築する。
pub struct Service {
    // ビルドバージョン（SemVer）。release ビルドで env! で注入する想定。
    version: String,
    // 起動時刻。Liveness の uptime_seconds 計算に使う（Instant は単調時計）。
    started_at: Instant,
    // 依存先 probe リスト。Arc で共有して並列実行する。
    probes: Vec<Arc<dyn DependencyProbe>>,
}

impl Service {
    // 新しい Service を生成する。起動時刻は new 呼出時に確定する。
    pub fn new(version: impl Into<String>, probes: Vec<Arc<dyn DependencyProbe>>) -> Self {
        // 構造体を組立てて返す。
        Self {
            // バージョン文字列を保持する。
            version: version.into(),
            // 起動時刻を確定する（Instant で単調に進む時刻）。
            started_at: Instant::now(),
            // 依存先 probe を保持する。
            probes,
        }
    }
}

// HealthService trait 実装。tonic は async fn を要求する。
#[tonic::async_trait]
impl HealthService for Service {
    // Liveness は version + uptime を返す。依存 backend は見ない。
    async fn liveness(
        &self,
        _request: tonic::Request<LivenessRequest>,
    ) -> Result<tonic::Response<LivenessResponse>, tonic::Status> {
        // 経過秒数を i64 に切り捨てる（proto 規定 int64）。
        let uptime = self.started_at.elapsed().as_secs() as i64;
        // 応答を組み立てて返す。
        Ok(tonic::Response::new(LivenessResponse {
            // SemVer 文字列。
            version: self.version.clone(),
            // 起動からの経過秒数。
            uptime_seconds: uptime,
        }))
    }

    // Readiness は probe 全件を並列実行し、全 OK なら ready=true を返す。
    async fn readiness(
        &self,
        _request: tonic::Request<ReadinessRequest>,
    ) -> Result<tonic::Response<ReadinessResponse>, tonic::Status> {
        // 結果集約用 map。proto の dependencies フィールドにそのまま渡す。
        let mut deps: std::collections::HashMap<String, DependencyStatus> =
            std::collections::HashMap::with_capacity(self.probes.len());
        // tokio JoinSet で probe を並列実行する。
        let mut set: JoinSet<(String, Result<(), String>)> = JoinSet::new();
        // 各 probe を spawn する。
        for probe in &self.probes {
            // Arc clone で move-able にする。
            let p = probe.clone();
            // 名前は async block の前にコピー（move clone）。
            let name = p.name().to_string();
            // 並列タスクを spawn する。
            set.spawn(async move {
                // probe.check の結果を (name, Result) に詰めて返す。
                let result = p.check().await;
                (name, result)
            });
        }
        // 全 probe 完了まで集約する。
        while let Some(res) = set.join_next().await {
            // タスク panic は probe 名不明な内部障害として skip（log は将来の OTel 経路）。
            let Ok((name, result)) = res else {
                // panic は Readiness 全体を 5xx に倒さない（部分失敗扱い、ready は false に倒す）。
                continue;
            };
            // probe 結果を DependencyStatus に詰める。
            let status = match result {
                // 到達 OK。
                Ok(()) => DependencyStatus {
                    // proto 規定: reachable のみ true、error_message は空。
                    reachable: true,
                    // error_message は空文字。
                    error_message: String::new(),
                },
                // 到達 NG（error_message に文字列を詰める）。
                Err(msg) => DependencyStatus {
                    // proto 規定: reachable=false の時のみ error_message が意味を持つ。
                    reachable: false,
                    // probe 由来の文字列。
                    error_message: msg,
                },
            };
            // map に登録する。
            deps.insert(name, status);
        }
        // 全件 reachable=true なら ready=true。
        let ready = deps.values().all(|d| d.reachable);
        // 応答を組み立てて返す。
        Ok(tonic::Response::new(ReadinessResponse {
            // 全体の ready 判定。
            ready,
            // 各依存の個別状態。
            dependencies: deps,
        }))
    }
}

// 単純な closure ベースの DependencyProbe 実装ヘルパ。
// Pod 側で 1 行で probe を定義できるよう公開する。
pub struct ClosureProbe<F>
where
    F: Fn() -> ClosureProbeFuture + Send + Sync + 'static,
{
    // 依存先論理名。
    name: String,
    // probe 実行関数（毎回 future を生成する）。
    probe: F,
}

// 戻り値の async ブロックを Box<dyn Future> に統一するため alias を切る。
pub type ClosureProbeFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

impl<F> ClosureProbe<F>
where
    F: Fn() -> ClosureProbeFuture + Send + Sync + 'static,
{
    // closure ベースの probe を構築する。
    pub fn new(name: impl Into<String>, probe: F) -> Self {
        // 構造体を組立てて返す。
        Self {
            // 依存先名。
            name: name.into(),
            // probe 関数。
            probe,
        }
    }
}

#[async_trait]
impl<F> DependencyProbe for ClosureProbe<F>
where
    F: Fn() -> ClosureProbeFuture + Send + Sync + 'static,
{
    // 依存先論理名を返す。
    fn name(&self) -> &str {
        // フィールドを参照で返す。
        &self.name
    }
    // probe 関数を呼び出して結果を返す。
    async fn check(&self) -> Result<(), String> {
        // 関数を呼んで future を取得し、await する。
        (self.probe)().await
    }
}

#[cfg(test)]
mod tests {
    // 標準 / crate を import する。
    use super::*;
    // tonic Request 構築用。
    use tonic::Request;

    // 簡易 probe（成功）。
    struct OkProbe(String);
    #[async_trait]
    impl DependencyProbe for OkProbe {
        // 依存先名。
        fn name(&self) -> &str {
            // 文字列を返す。
            &self.0
        }
        // 常に Ok を返す。
        async fn check(&self) -> Result<(), String> {
            // 成功。
            Ok(())
        }
    }

    // 簡易 probe（失敗）。
    struct FailProbe(String, String);
    #[async_trait]
    impl DependencyProbe for FailProbe {
        // 依存先名。
        fn name(&self) -> &str {
            // 文字列を返す。
            &self.0
        }
        // 常に Err を返す。
        async fn check(&self) -> Result<(), String> {
            // 失敗。error_message を持つ。
            Err(self.1.clone())
        }
    }

    // Liveness は version + uptime を返す。
    #[tokio::test]
    async fn liveness_returns_version_and_uptime() {
        // 空 probe で Service を構築する。
        let svc = Service::new("9.9.9-test", vec![]);
        // Liveness を呼ぶ。
        let resp = svc
            .liveness(Request::new(LivenessRequest {}))
            .await
            .expect("Liveness should not fail");
        // version は New に渡した値。
        assert_eq!(resp.get_ref().version, "9.9.9-test");
        // uptime は 0 以上。
        assert!(resp.get_ref().uptime_seconds >= 0);
    }

    // probes 空の Readiness は ready=true / 空 dependencies。
    #[tokio::test]
    async fn readiness_empty_probes_ready_true() {
        // 空 probe で Service を構築する。
        let svc = Service::new("0.0.0", vec![]);
        // Readiness を呼ぶ。
        let resp = svc
            .readiness(Request::new(ReadinessRequest {}))
            .await
            .expect("Readiness should not fail");
        // ready=true 期待。
        assert!(resp.get_ref().ready);
        // dependencies は空。
        assert!(resp.get_ref().dependencies.is_empty());
    }

    // probes 全件 OK の Readiness は ready=true / 各 dependency reachable=true。
    #[tokio::test]
    async fn readiness_all_probes_pass() {
        // 2 件の OK probe を設定する。
        let svc = Service::new(
            "0.0.0",
            vec![
                Arc::new(OkProbe("alpha".into())) as Arc<dyn DependencyProbe>,
                Arc::new(OkProbe("beta".into())) as Arc<dyn DependencyProbe>,
            ],
        );
        // Readiness を呼ぶ。
        let resp = svc
            .readiness(Request::new(ReadinessRequest {}))
            .await
            .expect("Readiness should not fail");
        // ready=true 期待。
        assert!(resp.get_ref().ready);
        // 各 dependency が reachable=true / error_message 空。
        for name in ["alpha", "beta"] {
            // dep を取得する。
            let dep = resp.get_ref().dependencies.get(name).expect("dep present");
            // reachable=true 期待。
            assert!(dep.reachable);
            // error_message は空。
            assert_eq!(dep.error_message, "");
        }
    }

    // 1 件失敗の Readiness は ready=false / 該当 dependency に error_message。
    #[tokio::test]
    async fn readiness_one_probe_fails() {
        // alpha は OK、beta は Fail。
        let svc = Service::new(
            "0.0.0",
            vec![
                Arc::new(OkProbe("alpha".into())) as Arc<dyn DependencyProbe>,
                Arc::new(FailProbe("beta".into(), "connection refused".into())) as Arc<dyn DependencyProbe>,
            ],
        );
        // Readiness を呼ぶ。
        let resp = svc
            .readiness(Request::new(ReadinessRequest {}))
            .await
            .expect("Readiness should not fail");
        // ready=false 期待。
        assert!(!resp.get_ref().ready);
        // alpha は reachable=true。
        assert!(resp.get_ref().dependencies["alpha"].reachable);
        // beta は reachable=false + error_message に文字列。
        assert!(!resp.get_ref().dependencies["beta"].reachable);
        // error_message 一致。
        assert_eq!(resp.get_ref().dependencies["beta"].error_message, "connection refused");
    }
}
