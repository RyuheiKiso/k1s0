// 本ファイルは Rust 共通の gRPC 観測性 interceptor。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「観測性 / 計装」
//   IMP-OBS-* / NFR-O-OBS-*: 全 RPC で trace span / RED metrics を出す
//
// 役割（Go 側 src/tier1/go/internal/common/interceptor.go と等価）:
//   tonic の UnaryInterceptor に挿し、各 RPC 呼出を tracing::info_span で囲む。
//   span 属性: rpc.system="grpc"、rpc.service / rpc.method / k1s0.tenant_id /
//   grpc.status_code。終了時に latency を計測しログに残す。
//
// 注:
//   OTel exporter は本リリース時点で `otel-util` crate 側に集約予定（現状空）。
//   本 interceptor は `tracing` の span を生やすだけで、collector への送出は
//   外部の `tracing-opentelemetry` 設定に委ねる。これにより crate 依存の最小化と
//   将来の OTel 結線（exporter only 差し替え）を両立する。

// 公開 API。ヘルスチェックや common 横断で使う。
use std::time::Instant;

/// `RpcCallContext` は 1 RPC 呼出のメタデータ束。
///
/// HTTP gateway / gRPC interceptor の両方からこの型を生成して
/// `enter_span` を呼ぶことで、両経路が同じ観測性を満たす。
#[derive(Debug, Clone)]
pub struct RpcCallContext {
    /// gRPC service 名（例: "k1s0.tier1.audit.v1.AuditService"）。
    pub service: String,
    /// RPC method 名（例: "Record"）。
    pub method: String,
    /// 認証済 tenant_id（off モードでは "demo-tenant"）。
    pub tenant_id: String,
}

/// 観測性 interceptor のエントリ。
///
/// 戻り値 `RpcSpan` を `Drop` するタイミングで latency が計測される。
/// gRPC interceptor / HTTP gateway の `tower::Layer` から call site で呼ぶ。
pub fn enter_span(ctx: RpcCallContext) -> RpcSpan {
    // tracing マクロは dotted key を field にすると `local ambiguity` でパース失敗するため、
    // span 生成は文字列メッセージのみで行い、属性は record() で個別に追加する。
    let span = tracing::info_span!(
        "k1s0.rpc",
        rpc_system = "grpc",
        rpc_service = ctx.service.as_str(),
        rpc_method = ctx.method.as_str(),
        k1s0_tenant_id = ctx.tenant_id.as_str(),
    );
    RpcSpan {
        span,
        started_at: Instant::now(),
        ctx,
    }
}

/// 観測性 span の RAII guard。
pub struct RpcSpan {
    /// tracing span 本体。
    span: tracing::Span,
    /// 開始時刻。
    started_at: Instant,
    /// 呼出 context（ログ整形に使う）。
    ctx: RpcCallContext,
}

impl RpcSpan {
    /// span を抜けるときに status を記録する（gRPC interceptor / HTTP gateway 共通）。
    pub fn finish(self, status_code: tonic::Code) {
        let elapsed_us = self.started_at.elapsed().as_micros() as u64;
        let code_int = status_code as i32;
        let _e = self.span.enter();
        tracing::info!(
            target: "k1s0::rpc",
            grpc_status_code = code_int,
            latency_us = elapsed_us,
            "{} {}.{} completed",
            self.ctx.tenant_id,
            self.ctx.service,
            self.ctx.method
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_span_returns_guard() {
        let s = enter_span(RpcCallContext {
            service: "svc".into(),
            method: "m".into(),
            tenant_id: "T".into(),
        });
        s.finish(tonic::Code::Ok);
    }
}
