// grpc_native.rs — spec 01 Bidi §adapter grpc_native
// 全 5 conformance_class をサポートする gRPC native streaming adapter。
// tonic Server::builder で bidi / server / client streaming を実装する。

use super::AdapterManifest;

// MANIFEST は grpc_native adapter の capability 自己宣言。
// generate_capabilities.py が集約して capabilities.lock.yaml を生成する。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "grpc_native",
    // grpc_native は全 5 conformance_class をサポートする（最も complete な adapter）
    supports: &[
        "v1_interactive",    // 双方向対話 gRPC bidi streaming
        "v1_alert",          // サーバー主導警報 gRPC server streaming
        "v1_event_feed",     // Domain Event 配信 gRPC server streaming
        "v1_live_snapshot",  // 最新値表示 gRPC server streaming
        "v1_bulk_upload",    // 大量データ投入 gRPC client streaming
    ],
    // gRPC native は TLS + HTTP/2 が前提のため fallback 不要
    requires_fallback: false,
    // tonic は HTTP/2 + protobuf が必須
    constraints: "http2=required, content_type=application/grpc",
};

// GrpcNativeAdapter は tonic ベースの gRPC bidi streaming adapter。
// bidi.rs の BidiSession state machine を tonic Streaming で wrap する。
pub struct GrpcNativeAdapter;

impl GrpcNativeAdapter {
    // conformance_class に対応した gRPC service を返す。
    // 実際の proto service は src/tier1/schema/bidi/v1/options.proto から codegen する。
    pub fn adapter_id() -> &'static str {
        // spec §adapter↔class supports 対応 の adapter 名を返す
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する。
    pub fn supports(conformance_class: &str) -> bool {
        // MANIFEST.supports から線形探索する（5 要素なので O(1) 同等）
        MANIFEST.supports.contains(&conformance_class)
    }
}
