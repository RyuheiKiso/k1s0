// k1s0 BFF / SDK 呼出の interface。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/03_maui_native配置.md

namespace K1s0.Native.Hub.Services;

// IK1s0Service は ViewModel が依存する k1s0 アクセス境界。
public interface IK1s0Service
{
    // store / key で State を取得する。未存在は null。
    Task<string?> GetStateAsync(string store, string key, CancellationToken ct = default);
}
