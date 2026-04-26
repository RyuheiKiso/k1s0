// 本ファイルは t1-pii Pod の起動エントリポイント。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-009（t1-pii Pod、純関数ステートレス）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// scope（リリース時点 最小骨格）:
//   - :50001 で listen
//   - PiiService（Classify / Mask）を登録、全 RPC Status::unimplemented
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-10）:
//   - PII 検出ルール（NAME / EMAIL / PHONE / MYNUMBER / CREDITCARD 等）
//   - 正規表現 + 文字列パターン照合
//   - 信頼度スコアリング
//
// docs 正典: 本 Pod は純関数（ステートレス）で副作用なし。HPA 推奨。

// SDK 公開 API の PiiService の Service trait / Server 型 / Request / Response 型を import。
use k1s0_sdk_proto::k1s0::tier1::pii::v1::{
    // PiiService の trait と Server 型。
    pii_service_server::{PiiService, PiiServiceServer},
    // Request / Response 型。
    ClassifyRequest, ClassifyResponse, MaskRequest, MaskResponse,
};
// tonic ランタイム。
use tonic::{transport::Server, Request, Response, Status};
// SIGTERM / SIGINT 受信。
use tokio::signal::unix::{signal, SignalKind};

// EXPOSE 50001 規約。
const DEFAULT_LISTEN: &str = "[::]:50001";

// PiiServer は PiiService の trait 実装（リリース時点 全 Status::unimplemented）。
#[derive(Default)]
struct PiiServer;

#[tonic::async_trait]
impl PiiService for PiiServer {
    // PII 種別の検出
    async fn classify(
        &self,
        _req: Request<ClassifyRequest>,
    ) -> Result<Response<ClassifyResponse>, Status> {
        // 実 PII 検出ロジック実装は plan 04-10。
        Err(Status::unimplemented(
            "tier1/pii: Classify not yet implemented (plan 04-10)",
        ))
    }

    // マスキング
    async fn mask(
        &self,
        _req: Request<MaskRequest>,
    ) -> Result<Response<MaskResponse>, Status> {
        // 実 PII 検出ロジック実装は plan 04-10。
        Err(Status::unimplemented(
            "tier1/pii: Mask not yet implemented (plan 04-10)",
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
            eprintln!("tier1/pii: received SIGTERM, shutting down");
        },
        _ = sigint.recv() => {
            // SIGINT ログ。
            eprintln!("tier1/pii: received SIGINT, shutting down");
        },
    }
}

// プロセスエントリポイント。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // listen address。
    let addr = DEFAULT_LISTEN.parse()?;
    // 起動ログ。
    eprintln!("tier1/pii: gRPC server listening on {}", DEFAULT_LISTEN);
    // tonic Server に PiiService を登録して起動する。
    Server::builder()
        // PiiService を登録。
        .add_service(PiiServiceServer::new(PiiServer::default()))
        // SIGINT / SIGTERM で graceful shutdown。
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    // 正常終了。
    Ok(())
}
