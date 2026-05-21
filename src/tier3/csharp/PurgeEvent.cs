// k1s0 tier3 client_purge_event emit 実装（C# .NET 8+ 等価強度実装）
// layers.yaml の purge_triggers（5 trigger × audit_emit: client_purge_event）を物理実装する
// R3-5: 全 purge trigger point で ClientPurgeEmitter.EmitClientPurgeEvent を呼び出すことを義務付ける

// System.Diagnostics: ActivitySource / Activity 経由で OTel span event を記録する
using System.Diagnostics;

// K1s0.Tier3.State 名前空間に配置する（既存 State.cs と同一 namespace）
namespace K1s0.Tier3.State;

// PurgeTrigger は layers.yaml の purge_triggers trigger_id と 1:1 に対応する enum
// 5 trigger: Logout / RefreshTokenExpiry / TenantSwitch / ActorSwitch / DeviceBoundKeyRotate
public enum PurgeTrigger
{
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

// PurgeTriggerExtensions は PurgeTrigger を layers.yaml trigger_id 文字列に変換する拡張メソッドを提供する
internal static class PurgeTriggerExtensions
{
    // ToTriggerIdString は PurgeTrigger を layers.yaml trigger_id 文字列に変換する
    internal static string ToTriggerIdString(this PurgeTrigger trigger) =>
        // enum variant を layers.yaml trigger_id と 1:1 に対応する文字列に変換する
        trigger switch
        {
            // Logout → "logout"
            PurgeTrigger.Logout => "logout",
            // RefreshTokenExpiry → "refresh_token_expiry"
            PurgeTrigger.RefreshTokenExpiry => "refresh_token_expiry",
            // TenantSwitch → "tenant_switch"
            PurgeTrigger.TenantSwitch => "tenant_switch",
            // ActorSwitch → "actor_switch"
            PurgeTrigger.ActorSwitch => "actor_switch",
            // DeviceBoundKeyRotate → "device_bound_key_rotate"
            PurgeTrigger.DeviceBoundKeyRotate => "device_bound_key_rotate",
            // 未知の variant はコンパイルエラーではなく実行時例外にする（網羅性は enum で担保）
            _ => throw new ArgumentOutOfRangeException(nameof(trigger), trigger, null),
        };
}

// ClientPurgeEmitter は client_purge_event を System.Diagnostics.Activity 経由で emit する static クラス
// System.Diagnostics.Activity は OpenTelemetry SDK が OTel span に変換する（自動計装）
public static class ClientPurgeEmitter
{
    // ActivitySource 名: k1s0-tier3（OTel instrumentation scope と一致させる）
    private static readonly ActivitySource _activitySource = new("k1s0-tier3");

    /// <summary>
    /// EmitClientPurgeEvent は client_purge_event を OTel span event（Activity event）として emit する。
    /// layers.yaml purge_triggers の audit_emit: client_purge_event 宣言に対する物理実装。
    /// </summary>
    /// <param name="trigger">発火した purge trigger 種別（PurgeTrigger 型）</param>
    /// <param name="tenantId">purge 対象のテナント ID（raw token/key bytes を含まない）</param>
    public static void EmitClientPurgeEvent(PurgeTrigger trigger, string tenantId)
    {
        // ActivitySource から新しい Activity を開始する（OTel span として記録される）
        using var activity = _activitySource.StartActivity("client_purge_event");
        // Activity が null の場合は OTel が設定されていない（no-op になる）
        if (activity is null) return;
        // purge trigger 種別を Activity Tag として記録する（layers.yaml trigger_id と一致させる）
        activity.SetTag("tier3_ext.trigger", trigger.ToTriggerIdString());
        // purge 対象テナント ID を Activity Tag として記録する（audit trail の一部）
        activity.SetTag("tier3_ext.tenant_id", tenantId);
        // client_purge_event を Activity Event として記録する（OTel span event に変換される）
        activity.AddEvent(new ActivityEvent("client_purge_event",
            // span event の属性として trigger と tenant_id を設定する
            tags: new ActivityTagsCollection
            {
                // purge trigger 種別を記録する
                { "tier3_ext.trigger", trigger.ToTriggerIdString() },
                // purge 対象テナント ID を記録する
                { "tier3_ext.tenant_id", tenantId },
            }
        ));
    }
}
