// k1s0 tier3 client_purge_event emit 実装（Rust 等価強度実装）
// layers.yaml の purge_triggers（5 trigger × audit_emit: client_purge_event）を物理実装する
// R3-5: 全 purge trigger point で emit_client_purge_event を呼び出すことを義務付ける

// OpenTelemetry trace API: span event 記録に使用する
use opentelemetry::{global, KeyValue};
// OpenTelemetry trace trait: with_active_span 等に使用する
use opentelemetry::trace::Tracer;

// PurgeTrigger は layers.yaml の purge_triggers trigger_id と 1:1 に対応する enum
// 5 trigger: Logout / RefreshTokenExpiry / TenantSwitch / ActorSwitch / DeviceBoundKeyRotate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeTrigger {
    // ログアウト時の全 layer purge
    Logout,
    // refresh token 失効時の全 layer purge
    RefreshTokenExpiry,
    // テナント切り替え時の全 layer purge
    TenantSwitch,
    // actor 切り替え時の全 layer purge
    ActorSwitch,
    // device_bound_key ローテーション時の全 layer purge
    DeviceBoundKeyRotate,
}

impl PurgeTrigger {
    // as_str は PurgeTrigger を layers.yaml trigger_id 文字列に変換する
    pub fn as_str(&self) -> &'static str {
        // enum variant を layers.yaml trigger_id と 1:1 に対応する文字列に変換する
        match self {
            // Logout → "logout"
            PurgeTrigger::Logout => "logout",
            // RefreshTokenExpiry → "refresh_token_expiry"
            PurgeTrigger::RefreshTokenExpiry => "refresh_token_expiry",
            // TenantSwitch → "tenant_switch"
            PurgeTrigger::TenantSwitch => "tenant_switch",
            // ActorSwitch → "actor_switch"
            PurgeTrigger::ActorSwitch => "actor_switch",
            // DeviceBoundKeyRotate → "device_bound_key_rotate"
            PurgeTrigger::DeviceBoundKeyRotate => "device_bound_key_rotate",
        }
    }
}

/// emit_client_purge_event は client_purge_event を OTel span event として emit する
/// layers.yaml purge_triggers の audit_emit: client_purge_event 宣言に対する物理実装
/// trigger: 発火した purge trigger 種別（PurgeTrigger 型）
/// tenant_id: purge 対象のテナント ID（public API には raw token/key bytes を含まない）
pub fn emit_client_purge_event(trigger: PurgeTrigger, tenant_id: &str) {
    // OTel global tracer を取得する（k1s0-tier3 の tracer 名で記録する）
    let tracer = global::tracer("k1s0-tier3");
    // アクティブなコンテキストで span を開始し client_purge_event を記録する
    tracer.in_span("client_purge_event", |cx| {
        // コンテキストから span を取得する
        use opentelemetry::trace::SpanExt as _;
        // アクティブな span を取得する
        let span = cx.span();
        // client_purge_event を span event として記録する
        span.add_event(
            // event 名: client_purge_event
            "client_purge_event",
            // span event の属性として trigger と tenant_id を設定する
            vec![
                // purge trigger 種別を記録する（layers.yaml trigger_id と一致させる）
                KeyValue::new("tier3_ext.trigger", trigger.as_str()),
                // purge 対象テナント ID を記録する（audit trail の一部）
                KeyValue::new("tier3_ext.tenant_id", tenant_id.to_string()),
            ],
        );
    });
}
