// k1s0-sdk-proto の最小単体テスト雛形。
//
// 範囲: 14 service の生成 stub が k1s0::tier1::<api>::v1 module 経由で
//       アクセス可能なことを smoke test として担保する。

use k1s0_sdk_proto::k1s0::tier1;

#[test]
fn all_14_services_accessible() {
    // 各 service の Request 型を 1 つずつ構築できること（コンパイル時担保 + ランタイム）。
    let _ = tier1::state::v1::GetRequest::default();
    let _ = tier1::pubsub::v1::PublishRequest::default();
    let _ = tier1::secrets::v1::GetSecretRequest::default();
    let _ = tier1::log::v1::SendLogRequest::default();
    let _ = tier1::workflow::v1::StartRequest::default();
    let _ = tier1::decision::v1::EvaluateRequest::default();
    let _ = tier1::audit::v1::RecordAuditRequest::default();
    let _ = tier1::pii::v1::ClassifyRequest::default();
    let _ = tier1::feature::v1::EvaluateRequest::default();
    let _ = tier1::binding::v1::InvokeBindingRequest::default();
    let _ = tier1::serviceinvoke::v1::InvokeRequest::default();
    let _ = tier1::telemetry::v1::EmitMetricRequest::default();
    let _ = tier1::decision::v1::RegisterRuleRequest::default();
    let _ = tier1::feature::v1::RegisterFlagRequest::default();
    let _ = tier1::health::v1::LivenessRequest::default();
}

#[test]
fn common_types_accessible() {
    use k1s0_sdk_proto::k1s0::tier1::common::v1::{ErrorDetail, K1s0ErrorCategory, TenantContext};
    let _ = TenantContext::default();
    let _ = ErrorDetail::default();
    // K1s0ErrorCategory の数値が IDL 正典と整合すること。
    // proto enum 値は K1S0_ERROR_<NAME>。buf lint で ENUM_VALUE_PREFIX を除外している
    // ため prost 標準の prefix 削除が効かず、生成 Rust 側は `K1s0Error<Name>` の
    // PascalCase 全文を保持する。
    assert_eq!(K1s0ErrorCategory::K1s0ErrorUnspecified as i32, 0);
    assert_eq!(K1s0ErrorCategory::K1s0ErrorInvalidArgument as i32, 1);
    assert_eq!(K1s0ErrorCategory::K1s0ErrorDeadlineExceeded as i32, 9);
}
