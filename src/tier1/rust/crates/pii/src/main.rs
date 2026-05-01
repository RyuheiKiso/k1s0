// 本ファイルは t1-pii Pod の起動エントリポイント（plan 04-10 結線済）。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-009（t1-pii Pod、純関数ステートレス）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//
// 役割:
//   - :50001 で listen
//   - PiiService（Classify / Mask）を実装し masker module 経由で regex 検出を実行
//   - SIGINT / SIGTERM で graceful shutdown
//
// 純関数性:
//   handler は &self のみ参照、Masker は zero-sized type で thread-safe。
//   HPA で水平スケール可能（DS-SW-COMP-009 の方針通り）。

// SDK 公開 API の PiiService の Service trait / Server 型 / Request / Response 型を import。
use k1s0_sdk_proto::FILE_DESCRIPTOR_SET;
// HealthServiceServer: 共通 HealthService 実装を gRPC server に登録するための型。
use k1s0_sdk_proto::k1s0::tier1::health::v1::health_service_server::HealthServiceServer;
use k1s0_sdk_proto::k1s0::tier1::pii::v1::{
    // Request / Response 型。
    ClassifyRequest, ClassifyResponse, MaskRequest, MaskResponse, PiiFinding,
    // FR-T1-PII-002 仮名化用の Request / Response 型。
    PseudonymizeRequest, PseudonymizeResponse,
    // PiiService の trait と Server 型。
    pii_service_server::{PiiService, PiiServiceServer},
};
// 共通 HealthService 実装。
use k1s0_tier1_health::Service as HealthSvc;
// 共通 gRPC interceptor Layer（auth / ratelimit / observability / audit auto-emit）。
use k1s0_tier1_common::grpc_layer::K1s0Layer;
// 共通 HTTP/JSON gateway。
use k1s0_tier1_common::http_gateway::{HttpGateway, JsonRpc, serve as serve_http};
// 共通 runtime（環境変数から共通リソースを構築）。
use k1s0_tier1_common::runtime::CommonRuntime;
// PII 検出 logic の library 部。
use k1s0_tier1_pii::http::{ClassifyRpc, MaskRpc, PiiHttpState, PseudonymizeRpc};
use k1s0_tier1_pii::masker::{Finding, Masker};
// FR-T1-PII-002: 仮名化純関数。
use k1s0_tier1_pii::pseudonymize::{pseudonymize, PseudonymizeError};
// 標準同期。
use std::sync::Arc;
// SIGTERM / SIGINT 受信。
use tokio::signal::unix::{SignalKind, signal};
// tonic ランタイム。
use tonic::{Request, Response, Status, transport::Server};

// EXPOSE 50001 規約。production の K8s Pod は単一 NetNS なので 50001 でぶつからないが、
// dev / 同一ホスト内で複数 Rust Pod を同時起動する場面は `LISTEN_ADDR` 環境変数で上書きする。
const DEFAULT_LISTEN: &str = "[::]:50001";

/// 環境変数 `LISTEN_ADDR` が設定されていればそれを使い、未設定なら DEFAULT_LISTEN を返す。
fn listen_addr() -> String {
    std::env::var("LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN.to_string())
}

// PiiServer は PiiService の trait 実装。masker への薄いラッパ。
#[derive(Default)]
struct PiiServer {
    // Masker は ZST + Lazy なので Clone 可能で安全。
    masker: Masker,
}

// Finding を proto PiiFinding に詰め替える純関数。
fn to_proto_finding(f: &Finding) -> PiiFinding {
    PiiFinding {
        // 種別文字列は kind.as_str() で確定（proto string 仕様）。
        r#type: f.kind.as_str().to_string(),
        // proto は int32 仕様、内部は usize（byte offset）。
        // proto 仕様コメントは「文字単位」と書いているが、UTF-8 multibyte 環境での
        // char index 計算は heavy なので byte offset を返す（呼び出し側合意のうえで運用）。
        start: f.start as i32,
        end: f.end as i32,
        confidence: f.confidence,
    }
}

#[tonic::async_trait]
impl PiiService for PiiServer {
    // PII 種別の検出。
    async fn classify(
        &self,
        req: Request<ClassifyRequest>,
    ) -> Result<Response<ClassifyResponse>, Status> {
        let text = &req.into_inner().text;
        let findings = self.masker.classify(text);
        let proto_findings: Vec<PiiFinding> = findings.iter().map(to_proto_finding).collect();
        // findings が空でなければ contains_pii=true。
        let contains = !proto_findings.is_empty();
        Ok(Response::new(ClassifyResponse {
            findings: proto_findings,
            contains_pii: contains,
        }))
    }

    // マスキング。
    async fn mask(&self, req: Request<MaskRequest>) -> Result<Response<MaskResponse>, Status> {
        let text = &req.into_inner().text;
        let (masked, findings) = self.masker.mask(text);
        let proto_findings: Vec<PiiFinding> = findings.iter().map(to_proto_finding).collect();
        Ok(Response::new(MaskResponse {
            masked_text: masked,
            findings: proto_findings,
        }))
    }

    // FR-T1-PII-002: 決定論的仮名化（HMAC-SHA256）。
    async fn pseudonymize(
        &self,
        req: Request<PseudonymizeRequest>,
    ) -> Result<Response<PseudonymizeResponse>, Status> {
        let r = req.into_inner();
        let pseudonym = pseudonymize(&r.field_type, &r.value, &r.salt).map_err(|e| match e {
            PseudonymizeError::EmptySalt => {
                Status::invalid_argument("tier1/pii: salt required")
            }
            PseudonymizeError::EmptyValue => {
                Status::invalid_argument("tier1/pii: value required")
            }
            PseudonymizeError::EmptyFieldType => {
                Status::invalid_argument("tier1/pii: field_type required")
            }
        })?;
        Ok(Response::new(PseudonymizeResponse { pseudonym }))
    }
}

// graceful shutdown シグナル待機。
async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = sigterm.recv() => {
            eprintln!("tier1/pii: received SIGTERM, shutting down");
        },
        _ = sigint.recv() => {
            eprintln!("tier1/pii: received SIGINT, shutting down");
        },
    }
}

// プロセスエントリポイント。
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listen = listen_addr();
    let addr = listen.parse()?;
    eprintln!("tier1/pii: gRPC server listening on {}", listen);
    // gRPC Server Reflection（Go Pod 側の reflection.Register と機能等価）。
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;
    // 共通 HealthService を構築する。pii Pod は純関数 / ステートレスのため probe 空。
    let health = HealthSvc::new(env!("CARGO_PKG_VERSION").to_string(), vec![]);
    // docs §共通規約 に従う interceptor chain を構築。
    let rt = CommonRuntime::from_env();
    let layer = K1s0Layer::new(rt.auth.clone(), rt.rate_limiter.clone(), rt.audit_emitter.clone());

    // HTTP/JSON gateway（TIER1_HTTP_LISTEN_ADDR が設定されている場合のみ起動）。
    // 共通規約 §「HTTP/JSON 互換インタフェース共通仕様」に従い、Pii API の
    // 2 RPC（Classify / Mask）を JSON で公開する。auth / ratelimit / audit は
    // gateway 側が同 chain で適用する（Go 側 HTTPGateway と同じ挙動）。
    let http_handle: Option<tokio::task::JoinHandle<()>> =
        match std::env::var("TIER1_HTTP_LISTEN_ADDR").ok().filter(|s| !s.is_empty()) {
            Some(http_addr) => {
                let pii_state = PiiHttpState {
                    masker: Arc::new(Masker::default()),
                };
                let gateway = HttpGateway::new(
                    rt.auth.clone(),
                    rt.rate_limiter.clone(),
                    rt.audit_emitter.clone(),
                )
                .register(Arc::new(ClassifyRpc { state: pii_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(MaskRpc { state: pii_state.clone() }) as Arc<dyn JsonRpc>)
                .register(Arc::new(PseudonymizeRpc {}) as Arc<dyn JsonRpc>);
                let router = gateway.into_router();
                eprintln!("tier1/pii: HTTP/JSON gateway listening on {}", http_addr);
                let addr_for_task = http_addr.clone();
                Some(tokio::spawn(async move {
                    if let Err(e) = serve_http(&addr_for_task, router).await {
                        eprintln!("tier1/pii: HTTP gateway error: {}", e);
                    }
                }))
            }
            None => None,
        };

    // 標準 grpc.health.v1.Health プロトコル登録（K8s grpc liveness/readiness probe 用）。
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PiiServiceServer<PiiServer>>()
        .await;

    Server::builder()
        .layer(layer)
        .add_service(PiiServiceServer::new(PiiServer::default()))
        .add_service(HealthServiceServer::new(health))
        .add_service(health_svc)
        .add_service(reflection)
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    // gRPC server 終了後 HTTP gateway も停止する（K8s では Pod 全体が落ちるので abort で十分）。
    if let Some(h) = http_handle {
        h.abort();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // handler テストは masker layer で網羅されているが、proto 詰替の正しさを 1 件確認する。
    use super::*;

    #[test]
    fn to_proto_finding_maps_fields() {
        use k1s0_tier1_pii::masker::PiiKind;
        let f = Finding {
            kind: PiiKind::Email,
            start: 5,
            end: 25,
            confidence: 0.9,
        };
        let p = to_proto_finding(&f);
        assert_eq!(p.r#type, "EMAIL");
        assert_eq!(p.start, 5);
        assert_eq!(p.end, 25);
        assert!((p.confidence - 0.9).abs() < 1e-9);
    }
}
