// 本ファイルは t1-decision Pod の起動エントリポイント（plan 04-08 結線済）。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-008（t1-decision Pod、JDM 評価エンジン）
//   docs/02_構想設計/adr/ADR-RULE-001-zen-engine.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//
// 役割:
//   - :50001 で listen
//   - DecisionService（Evaluate / BatchEvaluate）と DecisionAdminService
//     （RegisterRule / ListVersions / GetRule）を registry-backed 実装で登録
//   - SIGINT / SIGTERM で graceful shutdown
//   - DecisionService trait 実装本体は `k1s0_tier1_decision::service` に集約済
//     （src/CLAUDE.md: 500 行制限維持のため、main.rs は起動 wiring のみに絞る）。
//
// rule engine:
//   ZEN Engine 0.55+（gorules/zen）と JDM フォーマットを直接統合する（ADR-RULE-001 採用）。
//   登録された JDM は DecisionContent に parse + opcode キャッシュコンパイルされ、
//   評価時は DecisionEngine.evaluate_with_opts に委譲する。include_trace=true で
//   nodes 単位の評価トレースが返る（ADR-RULE-001 必須要件）。

use std::sync::Arc;

// SDK 公開 API の DecisionService / DecisionAdminService の Server 型と HealthService を import。
use k1s0_sdk_proto::FILE_DESCRIPTOR_SET;
// HealthServiceServer: 共通 HealthService 実装を gRPC server に登録するための型。
use k1s0_sdk_proto::k1s0::tier1::health::v1::health_service_server::HealthServiceServer;
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    decision_admin_service_server::DecisionAdminServiceServer,
    decision_service_server::DecisionServiceServer,
};
// 共通 HealthService 実装。
use k1s0_tier1_health::Service as HealthSvc;
// 共通 gRPC interceptor Layer（auth / ratelimit / observability / audit auto-emit）。
use k1s0_tier1_common::grpc_layer::K1s0Layer;
// 共通 HTTP/JSON gateway。
use k1s0_tier1_common::http_gateway::{HttpGateway, JsonRpc, serve as serve_http};
// 共通 runtime（環境変数から共通リソースを構築）。
use k1s0_tier1_common::runtime::CommonRuntime;
// HTTP/JSON gateway 用 adapter。
use k1s0_tier1_decision::http::{
    BatchEvaluateRpc, DecisionHttpState, EvaluateRpc, GetRuleRpc, ListVersionsRpc,
    RegisterRuleRpc,
};
// 内部 registry。
use k1s0_tier1_decision::registry::RuleRegistry;
// DecisionService / DecisionAdminService の trait 実装本体。
use k1s0_tier1_decision::service::{DecisionAdminServer, DecisionServer};
// SIGTERM / SIGINT 受信用。
use tokio::signal::unix::{SignalKind, signal};
// tonic ランタイム。
use tonic::transport::Server;

// EXPOSE 50001 規約。production の K8s Pod は単一 NetNS なので 50001 でぶつからないが、
// dev / 同一ホスト内で複数 Rust Pod を同時起動する場面は `LISTEN_ADDR` 環境変数で上書きする。
const DEFAULT_LISTEN: &str = "[::]:50001";

/// 環境変数 `LISTEN_ADDR` が設定されていればそれを使い、未設定なら DEFAULT_LISTEN を返す。
fn listen_addr() -> String {
    std::env::var("LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN.to_string())
}

async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = sigterm.recv() => { eprintln!("tier1/decision: received SIGTERM, shutting down"); },
        _ = sigint.recv() => { eprintln!("tier1/decision: received SIGINT, shutting down"); },
    }
}

/// `DECISION_JDM_DIR` 配下の JDM を起動時にロードし、watcher を起動する。
/// 値が "off" の場合は hot reload を無効化する（test / 単体起動経路）。
fn start_hot_reload(registry: &Arc<RuleRegistry>) -> Option<tokio::task::JoinHandle<()>> {
    let dir_env = std::env::var("DECISION_JDM_DIR")
        .unwrap_or_else(|_| "/etc/k1s0/decisions".to_string());
    if dir_env == "off" {
        eprintln!("tier1/decision: hot-reload disabled (DECISION_JDM_DIR=off)");
        return None;
    }
    let dir = std::path::PathBuf::from(dir_env);
    // 起動時 load: 失敗ファイルは warn ログのみで継続。
    match k1s0_tier1_decision::loader::load_initial(registry, &dir, "hot-reload") {
        Ok((ok, errors)) => {
            eprintln!(
                "tier1/decision: initial load loaded={} failed={} dir={}",
                ok,
                errors.len(),
                dir.display()
            );
            for (path, err) in &errors {
                eprintln!(
                    "tier1/decision: initial load error path={} error={}",
                    path.display(),
                    err
                );
            }
        }
        Err(e) => {
            eprintln!(
                "tier1/decision: initial load failed dir={} error={}",
                dir.display(),
                e
            );
        }
    }
    // watcher 起動: ディレクトリ未存在時は失敗するため warn して None で継続。
    match k1s0_tier1_decision::loader::spawn_watcher(
        registry.clone(),
        dir.clone(),
        "hot-reload".to_string(),
    ) {
        Ok(h) => {
            eprintln!(
                "tier1/decision: hot-reload watcher started dir={}",
                dir.display()
            );
            Some(h)
        }
        Err(e) => {
            eprintln!(
                "tier1/decision: hot-reload watcher start failed dir={} error={}",
                dir.display(),
                e
            );
            None
        }
    }
}

/// HTTP/JSON gateway を別 task で起動する（TIER1_HTTP_LISTEN_ADDR 設定時のみ）。
/// 共通規約 §「HTTP/JSON 互換」: DecisionService 2 RPC + DecisionAdminService 3 RPC を
/// JSON で公開する（5 unary RPC、bytes フィールドは base64 で表現）。
fn start_http_gateway(
    rt: &CommonRuntime,
    registry: Arc<RuleRegistry>,
) -> Option<tokio::task::JoinHandle<()>> {
    let http_addr = std::env::var("TIER1_HTTP_LISTEN_ADDR")
        .ok()
        .filter(|s| !s.is_empty())?;
    let http_state = DecisionHttpState { registry };
    let gateway = HttpGateway::new(
        rt.auth.clone(),
        rt.rate_limiter.clone(),
        rt.audit_emitter.clone(),
    )
    .register(Arc::new(EvaluateRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
    .register(Arc::new(BatchEvaluateRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
    .register(Arc::new(RegisterRuleRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
    .register(Arc::new(ListVersionsRpc { state: http_state.clone() }) as Arc<dyn JsonRpc>)
    .register(Arc::new(GetRuleRpc { state: http_state }) as Arc<dyn JsonRpc>);
    let router = gateway.into_router();
    eprintln!("tier1/decision: HTTP/JSON gateway listening on {}", http_addr);
    let addr_for_task = http_addr.clone();
    Some(tokio::spawn(async move {
        if let Err(e) = serve_http(&addr_for_task, router).await {
            eprintln!("tier1/decision: HTTP gateway error: {}", e);
        }
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // DS-SW-COMP-109: 共通 OTel 初期化。OTEL_EXPORTER_OTLP_ENDPOINT が設定済なら OTLP gRPC
    // 直送、未設定なら fmt layer のみ。Guard は main 関数の生存期間中保持する。
    let _otel_guard = k1s0_tier1_otel::init("t1-decision", "k1s0-tier1");
    let listen = listen_addr();
    let addr = listen.parse()?;
    eprintln!("tier1/decision: gRPC server listening on {}", listen);
    let registry = Arc::new(RuleRegistry::new());
    // 評価 / 登録時の rich audit emit に使う共通 emitter を main 側で確保する。
    // CommonRuntime::from_env() が下流で呼ばれるが、emitter のみ先に組み立てる。
    let audit_emitter = k1s0_tier1_common::runtime::load_audit_emitter_from_env();

    // FR-T1-DECISION-004: ConfigMap mount 配下の JDM ホットリロードを起動する。
    let _hot_reload_handle = start_hot_reload(&registry);

    let dec = DecisionServer {
        registry: registry.clone(),
        audit_emitter: audit_emitter.clone(),
    };
    let admin = DecisionAdminServer {
        registry: registry.clone(),
        audit_emitter,
    };
    // gRPC Server Reflection を有効化する（grpcurl の `list` / `describe` 対応、
    // Go Pod 側の reflection.Register と機能等価）。
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;
    // 共通 HealthService を構築する。decision Pod 自体は ZEN engine in-memory のため
    // 依存先 probe は空（リリース時点）。Postgres backed registry に切替時は probe 追加予定。
    let health = HealthSvc::new(env!("CARGO_PKG_VERSION").to_string(), vec![]);
    // docs §共通規約 に従う interceptor chain を構築。
    let rt = CommonRuntime::from_env();
    let layer = K1s0Layer::new(rt.auth.clone(), rt.rate_limiter.clone(), rt.audit_emitter.clone());

    // HTTP/JSON gateway を起動する（環境変数未設定なら nil）。
    let http_handle = start_http_gateway(&rt, registry);

    // 標準 grpc.health.v1.Health プロトコル登録（K8s grpc liveness/readiness probe 用）。
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<DecisionServiceServer<DecisionServer>>()
        .await;

    Server::builder()
        .layer(layer)
        .add_service(DecisionServiceServer::new(dec))
        .add_service(DecisionAdminServiceServer::new(admin))
        .add_service(HealthServiceServer::new(health))
        .add_service(health_svc)
        .add_service(reflection)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    if let Some(h) = http_handle {
        h.abort();
    }
    Ok(())
}
