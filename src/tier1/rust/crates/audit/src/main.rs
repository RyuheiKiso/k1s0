// 本ファイルは t1-audit Pod の起動エントリポイント。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、WORM 追記専用）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// scope（リリース時点 最小骨格）:
//   - :50001 で listen
//   - AuditService（Record / Query）を登録、全 RPC Status::unimplemented
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-09）:
//   - Postgres WORM 接続（CloudNativePG operator 経由）
//   - 改竄検知（hash chain / audit_id 採番）
//   - PII Mask 自動適用（Query 時の出力フィルタ）

// SDK 公開 API の AuditService の Service trait / Server 型 / Request / Response 型を import。
use k1s0_sdk_proto::k1s0::tier1::audit::v1::{
    // Request / Response 型。
    QueryAuditRequest,
    QueryAuditResponse,
    RecordAuditRequest,
    RecordAuditResponse,
    // AuditService の trait と Server 型。
    audit_service_server::{AuditService, AuditServiceServer},
};
// tonic ランタイム。
use tonic::{Request, Response, Status, transport::Server};
// SIGTERM / SIGINT 受信。
use tokio::signal::unix::{SignalKind, signal};

// EXPOSE 50001 規約。
const DEFAULT_LISTEN: &str = "[::]:50001";

// AuditServer は AuditService の trait 実装（リリース時点 全 Status::unimplemented）。
#[derive(Default)]
struct AuditServer;

#[tonic::async_trait]
impl AuditService for AuditServer {
    // 監査イベント記録
    async fn record(
        &self,
        _req: Request<RecordAuditRequest>,
    ) -> Result<Response<RecordAuditResponse>, Status> {
        // 実 Postgres WORM 結線は plan 04-09。
        Err(Status::unimplemented(
            "tier1/audit: Record not yet wired to Postgres WORM (plan 04-09)",
        ))
    }

    // 監査イベント検索
    async fn query(
        &self,
        _req: Request<QueryAuditRequest>,
    ) -> Result<Response<QueryAuditResponse>, Status> {
        // 実 Postgres WORM 結線は plan 04-09。
        Err(Status::unimplemented(
            "tier1/audit: Query not yet wired to Postgres WORM (plan 04-09)",
        ))
    }
}

// graceful shutdown シグナル待機。
async fn shutdown_signal() {
    // SIGTERM ハンドラ。
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    // SIGINT ハンドラ。
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    // どちらかのシグナルを待つ。
    tokio::select! {
        _ = sigterm.recv() => {
            // SIGTERM ログ。
            eprintln!("tier1/audit: received SIGTERM, shutting down");
        },
        _ = sigint.recv() => {
            // SIGINT ログ。
            eprintln!("tier1/audit: received SIGINT, shutting down");
        },
    }
}

// プロセスエントリポイント。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // listen address。
    let addr = DEFAULT_LISTEN.parse()?;
    // 起動ログ。
    eprintln!("tier1/audit: gRPC server listening on {}", DEFAULT_LISTEN);
    // tonic Server に AuditService を登録して起動する。
    Server::builder()
        // AuditService を登録。
        .add_service(AuditServiceServer::new(AuditServer))
        // SIGINT / SIGTERM で graceful shutdown。
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    // 正常終了。
    Ok(())
}
