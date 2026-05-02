// k1s0 BFF / SDK 呼出の interface（Admin 用）。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_maui_native配置.md
//
// Hub 側は State.Get の単一 method のみ露出するが、Admin は管理者向けに
// Audit.Query を主機能として公開する（リリース時点 で TenantList / Feature 管理を追加）。

namespace K1s0.Native.Admin.Services;

// AuditEvent は BFF /api/audit/query の応答 1 件分。
public sealed record AuditEvent(
    long OccurredAtMillis,
    string Actor,
    string Action,
    string Resource,
    string Outcome);

// IK1s0Service は ViewModel が依存する k1s0 アクセス境界。
public interface IK1s0Service
{
    // 直近 hours 時間以内の監査イベントを最大 limit 件返す。
    // 空配列は「該当なし」を意味する（例外ではない）。
    Task<IReadOnlyList<AuditEvent>> QueryAuditAsync(int hours, int limit, CancellationToken ct = default);
}
