// k1s0 tier3 client_purge_event emit 実装（Go 等価強度実装）
// layers.yaml の purge_triggers（5 trigger × audit_emit: client_purge_event）を物理実装する
// R3-5: 全 purge trigger point で EmitClientPurgeEvent を呼び出すことを義務付ける
package state

import (
	// context パッケージ（OTel span 取得に使用する）
	"context"
	// OTel trace API（span event 記録に使用する）
	"go.opentelemetry.io/otel/trace"
	// OTel attribute（span event の属性設定に使用する）
	"go.opentelemetry.io/otel/attribute"
)

// PurgeTrigger は layers.yaml の purge_triggers trigger_id と 1:1 に対応する型
// 5 trigger: logout / refresh_token_expiry / tenant_switch / actor_switch / device_bound_key_rotate
type PurgeTrigger string

const (
	// PurgeTriggerLogout はログアウト時の全 layer purge を表す
	PurgeTriggerLogout PurgeTrigger = "logout"
	// PurgeTriggerRefreshTokenExpiry は refresh token 失効時の全 layer purge を表す
	PurgeTriggerRefreshTokenExpiry PurgeTrigger = "refresh_token_expiry"
	// PurgeTriggerTenantSwitch はテナント切り替え時の全 layer purge を表す
	PurgeTriggerTenantSwitch PurgeTrigger = "tenant_switch"
	// PurgeTriggerActorSwitch は actor 切り替え時の全 layer purge を表す
	PurgeTriggerActorSwitch PurgeTrigger = "actor_switch"
	// PurgeTriggerDeviceBoundKeyRotate は device_bound_key ローテーション時の全 layer purge を表す
	PurgeTriggerDeviceBoundKeyRotate PurgeTrigger = "device_bound_key_rotate"
)

// EmitClientPurgeEvent は client_purge_event を OTel span event として emit する
// layers.yaml purge_triggers の audit_emit: client_purge_event 宣言に対する物理実装
// ctx: アクティブな OTel span を含む context（span がない場合は no-op になる）
// trigger: 発火した purge trigger 種別（PurgeTrigger 型）
// tenantId: purge 対象のテナント ID（public API には raw token/key bytes を含まない）
func EmitClientPurgeEvent(ctx context.Context, trigger PurgeTrigger, tenantId string) {
	// context からアクティブな span を取得する
	span := trace.SpanFromContext(ctx)
	// client_purge_event を span event として記録する
	span.AddEvent("client_purge_event",
		// span event の属性として trigger と tenant_id を設定する
		trace.WithAttributes(
			// purge trigger 種別を記録する（layers.yaml trigger_id と一致させる）
			attribute.String("tier3_ext.trigger", string(trigger)),
			// purge 対象テナント ID を記録する（audit trail の一部）
			attribute.String("tier3_ext.tenant_id", tenantId),
		),
	)
}
