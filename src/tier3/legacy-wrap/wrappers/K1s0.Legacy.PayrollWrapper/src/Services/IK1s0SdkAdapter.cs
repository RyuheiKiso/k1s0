// k1s0 公開 API を呼ぶ境界 interface。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md
//
// 設計動機:
//   wrapper Domain ロジック（PayrollService）は本 interface にのみ依存し、
//   tier1 アクセス手段（BFF REST / SDK gRPC）の差し替えを境界化する。

namespace K1s0.Legacy.PayrollWrapper.Services;

// IK1s0SdkAdapter は wrapper が k1s0 公開 API を呼ぶ境界。
public interface IK1s0SdkAdapter
{
    // BFF /api/state/save 相当: 計算結果 JSON を State に保存する。
    Task SaveStateAsync(string store, string key, string jsonValue, CancellationToken ct = default);
    // BFF /api/audit/record 相当: 監査イベントを記録する。
    Task RecordAuditAsync(string actor, string action, string resource, string outcome, CancellationToken ct = default);
}
