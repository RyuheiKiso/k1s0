// k1s0 tier3 client_purge_event emit 実装
// layers.yaml の purge_triggers（5 trigger × audit_emit: client_purge_event）を物理実装する
// R3-5: 全 purge trigger point で emitClientPurgeEvent を呼び出すことを義務付ける

// OpenTelemetry trace API をインポートする（span event として audit emit する）
import { trace } from "@opentelemetry/api";

// PurgeTrigger は layers.yaml の purge_triggers trigger_id と 1:1 に対応する型
// 5 trigger: logout / refresh_token_expiry / tenant_switch / actor_switch / device_bound_key_rotate
export type PurgeTrigger =
  // ログアウト時の全 layer purge
  | "logout"
  // refresh token 失効時の全 layer purge
  | "refresh_token_expiry"
  // テナント切り替え時の全 layer purge
  | "tenant_switch"
  // actor 切り替え時の全 layer purge
  | "actor_switch"
  // device_bound_key ローテーション時の全 layer purge
  | "device_bound_key_rotate";

// emitClientPurgeEvent は client_purge_event を OTel span event として emit する
// layers.yaml purge_triggers の audit_emit: client_purge_event 宣言に対する物理実装
// trigger: 発火した purge trigger 種別（PurgeTrigger 型）
// tenantId: purge 対象のテナント ID（public API には raw token/key bytes を含まない）
export function emitClientPurgeEvent(trigger: PurgeTrigger, tenantId: string): void {
  // アクティブな OTel span を取得する（span がない場合は no-op になる）
  const span = trace.getActiveSpan();
  // span が存在する場合は client_purge_event を span event として記録する
  span?.addEvent("client_purge_event", {
    // purge trigger 種別を記録する（layers.yaml trigger_id と一致させる）
    "tier3_ext.trigger": trigger,
    // purge 対象テナント ID を記録する（audit trail の一部）
    "tier3_ext.tenant_id": tenantId,
  });
}

// ============================================================
// 各 purge trigger point での emitClientPurgeEvent 呼び出し例
// ============================================================
//
// (1) logout trigger:
//   emitClientPurgeEvent("logout", tenantId);
//   dispatch(reducePurge(state, "logout"));
//
// (2) refresh_token_expiry trigger:
//   emitClientPurgeEvent("refresh_token_expiry", tenantId);
//   dispatch(reducePurge(state, "refresh_token_expiry"));
//
// (3) tenant_switch trigger:
//   emitClientPurgeEvent("tenant_switch", tenantId);
//   dispatch(reducePurge(state, "tenant_switch"));
//
// (4) actor_switch trigger:
//   emitClientPurgeEvent("actor_switch", tenantId);
//   dispatch(reducePurge(state, "actor_switch"));
//
// (5) device_bound_key_rotate trigger:
//   emitClientPurgeEvent("device_bound_key_rotate", tenantId);
//   dispatch(reducePurge(state, "device_bound_key_rotate"));
//
// ============================================================
// 上記パターンを各 trigger 呼び出し元（reducer dispatch 前）に必ず挿入すること
// audit_emit 宣言に対する物理実装の完全性を担保する
// ============================================================
