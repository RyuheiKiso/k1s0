// IFeatureFlag.cs — k1s0 tier1 Library C# 実装: Configuration / Feature Flag の L2* interface
// 07_設定適合仕様.md §IFeatureFlagClient / §IConfigClient（族内共通 API）に準拠する。
// OpenFeature / LaunchDarkly 等の OSS API を Library 独自語彙に翻訳する L2* facade を宣言する。
// 公開シグネチャに OSS 型を露出しない（ldclient 等は一切含まない）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// EvalContext は L2* Feature Flag の評価コンテキストを宣言する型。
/// OpenFeature EvaluationContext に準拠した Library 独自語彙とする。
/// TenantId / UserId を必須 / 省略可フィールドとして持つ。
/// </summary>
// EvalContext クラス定義
public sealed class EvalContext
{
    /// <summary>TenantId: フラグ評価の対象テナント識別子（必須）</summary>
    // TenantId プロパティ（必須）
    public required string TenantId { get; init; }

    /// <summary>UserId: フラグ評価の対象ユーザー識別子（省略可能: A/B test 等で使用する）</summary>
    // UserId プロパティ
    public string? UserId { get; init; }

    /// <summary>Attrs: 追加評価属性（region / plan / custom 等）</summary>
    // Attrs プロパティ
    public IReadOnlyDictionary<string, string>? Attrs { get; init; }
}

/// <summary>
/// FlagEvalReason はフラグ評価理由を宣言する enum。
/// OpenFeature EvaluationReason に準拠した Library 独自語彙とする。
/// </summary>
// FlagEvalReason 列挙型定義
public enum FlagEvalReason
{
    /// <summary>Static: 静的（ルールマッチなし / デフォルト値）</summary>
    Static,
    /// <summary>Targeting: ターゲティングルールに一致して評価した</summary>
    Targeting,
    /// <summary>Split: A/B split test により評価した</summary>
    Split,
    /// <summary>Default: デフォルト値にフォールバックした（エラー発生時等）</summary>
    Default,
}

/// <summary>
/// BoolEvalResult は bool 型フラグの評価結果を宣言する型。
/// 値だけでなく評価理由も含めて返す（監査ログ・デバッグに使用する）。
/// </summary>
// BoolEvalResult レコード定義
public sealed record BoolEvalResult(
    // Value: 評価されたフラグ値
    bool Value,
    // Reason: フラグ評価理由
    FlagEvalReason Reason,
    // Variant: 評価されたバリアント名
    string Variant
);

/// <summary>
/// StringEvalResult は string 型フラグの評価結果を宣言する型。
/// </summary>
// StringEvalResult レコード定義
public sealed record StringEvalResult(
    string Value,
    FlagEvalReason Reason,
    string Variant
);

/// <summary>
/// DoubleEvalResult は double 型フラグの評価結果を宣言する型。
/// </summary>
// DoubleEvalResult レコード定義
public sealed record DoubleEvalResult(
    double Value,
    FlagEvalReason Reason,
    string Variant
);

/// <summary>
/// IFeatureFlagClient は L2* Feature Flag 評価 interface を宣言する。
/// OpenFeature Provider を Library 独自語彙で抽象化する。
/// OSS 型（ldclient 等）を引数・戻り値に一切含まない。
/// </summary>
// IFeatureFlagClient インターフェース定義
public interface IFeatureFlagClient
{
    /// <summary>
    /// GetBoolAsync は bool 型フラグを評価して値を返す。
    /// key はフラグキー（"feature.new-ui" 等のドット記法を推奨する）。
    /// ec は評価コンテキスト（TenantId は必須）。
    /// defaultVal はフォールバック値（エラー発生時に使用する）。
    /// </summary>
    // GetBoolAsync メソッド: bool フラグを評価する
    Task<bool> GetBoolAsync(
        string key,
        EvalContext ec,
        bool defaultVal,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// GetBoolDetailAsync は bool 型フラグを評価して詳細結果を返す（評価理由を含む）。
    /// </summary>
    // GetBoolDetailAsync メソッド: bool フラグの詳細評価結果を返す
    Task<BoolEvalResult> GetBoolDetailAsync(
        string key,
        EvalContext ec,
        bool defaultVal,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// GetStringAsync は string 型フラグを評価して値を返す。
    /// </summary>
    // GetStringAsync メソッド: string フラグを評価する
    Task<string> GetStringAsync(
        string key,
        EvalContext ec,
        string defaultVal,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// GetStringDetailAsync は string 型フラグを評価して詳細結果を返す。
    /// </summary>
    // GetStringDetailAsync メソッド: string フラグの詳細評価結果を返す
    Task<StringEvalResult> GetStringDetailAsync(
        string key,
        EvalContext ec,
        string defaultVal,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// GetDoubleAsync は double 型フラグを評価して値を返す。
    /// </summary>
    // GetDoubleAsync メソッド: double フラグを評価する
    Task<double> GetDoubleAsync(
        string key,
        EvalContext ec,
        double defaultVal,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// ConfigValue は設定値と付随するメタデータを宣言する型。
/// OSS の config 型を露出せず Library 独自語彙で表現する。
/// </summary>
// ConfigValue レコード定義
public sealed record ConfigValue(
    // Raw: 設定の生値（文字列表現）
    string Raw,
    // Source: 設定の取得元（"env" / "file" / "remote" 等）
    string Source,
    // Version: 設定バージョン（リモート設定ストアの revision 等）
    long Version
);

/// <summary>
/// IConfigClient は L2* 設定取得 interface を宣言する。
/// Consul / etcd / ConfigMap 等の設定ストアを Library 独自語彙で抽象化する。
/// OSS 型を引数・戻り値に一切含まない。
/// </summary>
// IConfigClient インターフェース定義
public interface IConfigClient
{
    /// <summary>
    /// GetStringAsync は string 型設定値を取得する。
    /// key はドット記法のキー（"service.timeout" 等）。
    /// 値が存在しない場合は defaultVal を返す。
    /// </summary>
    // GetStringAsync メソッド: string 設定値を取得する
    Task<string> GetStringAsync(string key, string defaultVal, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetLongAsync は long 型設定値を取得する。
    /// </summary>
    // GetLongAsync メソッド: long 設定値を取得する
    Task<long> GetLongAsync(string key, long defaultVal, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetBoolAsync は bool 型設定値を取得する。
    /// </summary>
    // GetBoolAsync メソッド: bool 設定値を取得する
    Task<bool> GetBoolAsync(string key, bool defaultVal, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetDoubleAsync は double 型設定値を取得する。
    /// </summary>
    // GetDoubleAsync メソッド: double 設定値を取得する
    Task<double> GetDoubleAsync(string key, double defaultVal, CancellationToken cancellationToken = default);

    /// <summary>
    /// GetValueAsync は ConfigValue（メタデータ付き）で設定値を取得する。
    /// キーが存在しない場合は null を返す（エラーと区別する）。
    /// </summary>
    // GetValueAsync メソッド: ConfigValue でメタデータ付き設定値を取得する
    Task<ConfigValue?> GetValueAsync(string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// WatchAsync は設定キーの変更を監視して IAsyncEnumerable でイベントを提供する。
    /// cancellationToken のキャンセルで監視を停止する（goroutine リーク防止と同様）。
    /// </summary>
    // WatchAsync メソッド: 設定変更を監視する
    IAsyncEnumerable<ConfigValue> WatchAsync(string key, CancellationToken cancellationToken = default);
}
