// k1s0-sdk の最小単体テスト雛形。
//
// 範囲（リリース時点）:
//   - Config 構造体の構築 / Clone
//   - proto 型の re-export 経路（k1s0_sdk::proto 配下から型がアクセス可能）
//   - 各 facade のメソッドシグネチャ（コンパイル時点で担保）
//
// 採用初期で拡張:
//   - tonic mock server を立てた end-to-end テスト
//   - integration test（tier1 facade 起動済 cluster に対する疎通確認）

use k1s0_sdk::{Config, proto};

#[test]
fn config_construction_and_clone() {
    let cfg = Config {
        target: "http://localhost:50001".into(),
        tenant_id: "tenant-A".into(),
        subject: "svc-foo".into(),
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.target, "http://localhost:50001");
    assert_eq!(cloned.tenant_id, "tenant-A");
    assert_eq!(cloned.subject, "svc-foo");
}

#[test]
fn proto_state_request_construction() {
    // proto re-export 経由で k1s0::tier1::state::v1::GetRequest が見えること。
    let req = proto::k1s0::tier1::state::v1::GetRequest {
        store: "valkey-default".into(),
        key: "user/1".into(),
        context: Some(proto::k1s0::tier1::common::v1::TenantContext {
            tenant_id: "tenant-A".into(),
            subject: "svc-foo".into(),
            correlation_id: String::new(),
        }),
    };
    assert_eq!(req.store, "valkey-default");
    assert_eq!(req.context.unwrap().tenant_id, "tenant-A");
}

#[test]
fn proto_log_severity_otel_alignment() {
    // OTel SeverityNumber との整合（IDL 正典）。
    use proto::k1s0::tier1::log::v1::Severity;
    assert_eq!(Severity::Trace as i32, 0);
    assert_eq!(Severity::Debug as i32, 5);
    assert_eq!(Severity::Info as i32, 9);
    assert_eq!(Severity::Warn as i32, 13);
    assert_eq!(Severity::Error as i32, 17);
    assert_eq!(Severity::Fatal as i32, 21);
}
