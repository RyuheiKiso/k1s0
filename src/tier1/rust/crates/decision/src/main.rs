// 本ファイルは t1-decision Pod の起動エントリポイント。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-008（t1-decision Pod）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/09_Decision_API.md
//
// scope（リリース時点 最小骨格）:
//   - :50001 で listen（docs 正典 EXPOSE 50001）
//   - DecisionService / DecisionAdminService を登録（全 RPC は Status::unimplemented）
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-08）:
//   - ZEN Engine 統合（zen-engine crate）
//   - 内部 gRPC 経由の audit chain 連携
//   - OTel tracing / metrics interceptor

// SDK 公開 API の DecisionService / DecisionAdminService の Service trait と Server 型を import。
use k1s0_sdk_proto::k1s0::tier1::decision::v1::{
    // DecisionService（Evaluate / BatchEvaluate）の trait と Server 型。
    decision_service_server::{DecisionService, DecisionServiceServer},
    // DecisionAdminService（RegisterRule / ListVersions / GetRule）の trait と Server 型。
    decision_admin_service_server::{DecisionAdminService, DecisionAdminServiceServer},
    // Request / Response 型。
    BatchEvaluateRequest, BatchEvaluateResponse, EvaluateRequest, EvaluateResponse,
    GetRuleRequest, GetRuleResponse, ListVersionsRequest, ListVersionsResponse,
    RegisterRuleRequest, RegisterRuleResponse,
};
// tonic ランタイム / 型。
use tonic::{transport::Server, Request, Response, Status};
// SIGTERM / SIGINT 受信用。
use tokio::signal::unix::{signal, SignalKind};

// EXPOSE 50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md と整合。
// Rust 側も同じポート規約を採用する。
const DEFAULT_LISTEN: &str = "[::]:50001";

// DecisionServer は DecisionService の trait 実装（リリース時点 全 Status::unimplemented）。
#[derive(Default)]
struct DecisionServer;

#[tonic::async_trait]
impl DecisionService for DecisionServer {
    // ルール評価（同期）
    async fn evaluate(
        &self,
        _req: Request<EvaluateRequest>,
    ) -> Result<Response<EvaluateResponse>, Status> {
        // 実 ZEN Engine 結線は plan 04-08。
        Err(Status::unimplemented(
            "tier1/decision: Evaluate not yet wired to ZEN Engine (plan 04-08)",
        ))
    }

    // バッチ評価
    async fn batch_evaluate(
        &self,
        _req: Request<BatchEvaluateRequest>,
    ) -> Result<Response<BatchEvaluateResponse>, Status> {
        // 実 ZEN Engine 結線は plan 04-08。
        Err(Status::unimplemented(
            "tier1/decision: BatchEvaluate not yet wired to ZEN Engine (plan 04-08)",
        ))
    }
}

// DecisionAdminServer は DecisionAdminService の trait 実装。
#[derive(Default)]
struct DecisionAdminServer;

#[tonic::async_trait]
impl DecisionAdminService for DecisionAdminServer {
    // JDM 文書の登録
    async fn register_rule(
        &self,
        _req: Request<RegisterRuleRequest>,
    ) -> Result<Response<RegisterRuleResponse>, Status> {
        // 実 ZEN Engine 結線は plan 04-08。
        Err(Status::unimplemented(
            "tier1/decision: RegisterRule not yet wired to ZEN Engine (plan 04-08)",
        ))
    }

    // バージョン一覧
    async fn list_versions(
        &self,
        _req: Request<ListVersionsRequest>,
    ) -> Result<Response<ListVersionsResponse>, Status> {
        // 実 ZEN Engine 結線は plan 04-08。
        Err(Status::unimplemented(
            "tier1/decision: ListVersions not yet wired to ZEN Engine (plan 04-08)",
        ))
    }

    // 特定バージョンの取得
    async fn get_rule(
        &self,
        _req: Request<GetRuleRequest>,
    ) -> Result<Response<GetRuleResponse>, Status> {
        // 実 ZEN Engine 結線は plan 04-08。
        Err(Status::unimplemented(
            "tier1/decision: GetRule not yet wired to ZEN Engine (plan 04-08)",
        ))
    }
}

// graceful shutdown 用の Future を生成する。
// SIGINT / SIGTERM のいずれかを受信したら resolve する。
async fn shutdown_signal() {
    // SIGTERM ハンドラ（k8s Pod 終了）。
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    // SIGINT ハンドラ（Ctrl-C）。
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    // どちらかのシグナルを待つ。
    tokio::select! {
        _ = sigterm.recv() => {
            // SIGTERM ログ。
            eprintln!("tier1/decision: received SIGTERM, shutting down");
        },
        _ = sigint.recv() => {
            // SIGINT ログ。
            eprintln!("tier1/decision: received SIGINT, shutting down");
        },
    }
}

// プロセスエントリポイント（tokio runtime + tonic server + graceful shutdown）。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // listen address（CLI 引数で上書き可、本リリース時点 は固定既定値）。
    let addr = DEFAULT_LISTEN.parse()?;
    // 起動ログ。
    eprintln!("tier1/decision: gRPC server listening on {}", DEFAULT_LISTEN);
    // tonic Server に DecisionService と DecisionAdminService を登録して起動する。
    Server::builder()
        // DecisionService を登録。
        .add_service(DecisionServiceServer::new(DecisionServer::default()))
        // DecisionAdminService を登録。
        .add_service(DecisionAdminServiceServer::new(DecisionAdminServer::default()))
        // SIGINT / SIGTERM で graceful shutdown する。
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    // 正常終了。
    Ok(())
}
