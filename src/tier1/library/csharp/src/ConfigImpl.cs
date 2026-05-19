// ConfigImpl.cs — k1s0 tier1 Library C# 実装: IFeatureFlagClient / IConfigClient の OpenFeature facade 実装
// 07_設定適合仕様.md §IFeatureFlagClient / §IConfigClient（族内共通 API）に準拠する。
// OpenFeature SDK を L2* ラップして公開 API に OSS 型（IFeatureClient 等）を露出しない。
// flagd / LaunchDarkly / OpenFeature Provider を実装で切り替え可能にする。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Channels: Channel<T> で IAsyncEnumerable を実装する
using System.Threading.Channels;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// OpenFeature: OpenFeature SDK の主要型（内部のみ使用する）
using OpenFeature;
// OpenFeature.Model: EvaluationContext / Value に使用する（内部のみ使用する）
using OpenFeature.Model;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// FeatureFlagImpl は IFeatureFlagClient の OpenFeature SDK facade 実装クラス。
/// OpenFeature の FeatureClient を L2* ラップして公開 API に OSS 型を露出しない。
/// EvalContext を OpenFeature EvaluationContext に変換して評価する。
/// </summary>
// FeatureFlagImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class FeatureFlagImpl : IFeatureFlagClient
{
    // _client: OpenFeature FeatureClient（L2* ラップのため型を内部に隠蔽する）
    private readonly FeatureClient _client;

    /// <summary>
    /// コンストラクタ: OpenFeature FeatureClient を注入する。
    /// FeatureClient 型で受け取り、内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: FeatureClient を依存注入する
    public FeatureFlagImpl(FeatureClient client)
    {
        // null チェック: null が渡された場合は例外を投げる
        _client = client ?? throw new ArgumentNullException(nameof(client));
    }

    // ToEvaluationContext は Library の EvalContext を OpenFeature の EvaluationContext に変換する
    private static EvaluationContext ToEvaluationContext(EvalContext ec)
    {
        // EvaluationContextBuilder を使って変換する
        var builder = EvaluationContext.Builder()
            .SetTargetingKey(ec.TenantId);
        // UserId が設定されている場合は属性として追加する
        if (ec.UserId is not null)
        {
            // userId 属性を設定する
            builder.Set("userId", ec.UserId);
        }
        // Attrs が設定されている場合は全属性を追加する
        if (ec.Attrs is not null)
        {
            // 各属性を EvaluationContext に追加する
            foreach (var kv in ec.Attrs)
            {
                // 属性をビルダーに設定する
                builder.Set(kv.Key, kv.Value);
            }
        }
        // EvaluationContext を構築して返す
        return builder.Build();
    }

    // ToFlagEvalReason は OpenFeature の Reason を Library の FlagEvalReason に変換する
    private static FlagEvalReason ToFlagEvalReason(string? reason)
    {
        // null / 空文字は Static として扱う
        return reason switch
        {
            // TARGETING_MATCH: ターゲティングルール一致
            "TARGETING_MATCH" => FlagEvalReason.Targeting,
            // SPLIT: A/B split test
            "SPLIT" => FlagEvalReason.Split,
            // DEFAULT: デフォルト値フォールバック
            "DEFAULT" => FlagEvalReason.Default,
            // それ以外は Static（静的評価）として扱う
            _ => FlagEvalReason.Static,
        };
    }

    /// <summary>
    /// GetBoolAsync は bool 型フラグを評価して値を返す。
    /// EvalContext を OpenFeature EvaluationContext に変換して評価する。
    /// </summary>
    // GetBoolAsync メソッド実装: OpenFeature FeatureClient.GetBooleanValue を呼び出す（OpenFeature 1.x API）
    public async Task<bool> GetBoolAsync(string key, EvalContext ec, bool defaultVal, CancellationToken cancellationToken = default)
    {
        // EvalContext を OpenFeature EvaluationContext に変換する
        var evalCtx = ToEvaluationContext(ec);
        // OpenFeature 1.x の GetBooleanValue（Task を返す）を呼び出してフラグを評価する
        return await _client.GetBooleanValue(key, defaultVal, evalCtx).ConfigureAwait(false);
    }

    /// <summary>
    /// GetBoolDetailAsync は bool 型フラグを評価して詳細結果を返す（評価理由を含む）。
    /// </summary>
    // GetBoolDetailAsync メソッド実装: OpenFeature FeatureClient.GetBooleanDetails を呼び出す
    public async Task<BoolEvalResult> GetBoolDetailAsync(string key, EvalContext ec, bool defaultVal, CancellationToken cancellationToken = default)
    {
        // EvalContext を OpenFeature EvaluationContext に変換する
        var evalCtx = ToEvaluationContext(ec);
        // OpenFeature 1.x の GetBooleanDetails（Task を返す）を呼び出して詳細結果を取得する
        var detail = await _client.GetBooleanDetails(key, defaultVal, evalCtx).ConfigureAwait(false);
        // FlagEvaluationDetails を Library の BoolEvalResult に変換して返す
        return new BoolEvalResult(
            detail.Value,
            ToFlagEvalReason(detail.Reason),
            detail.Variant ?? string.Empty);
    }

    /// <summary>
    /// GetStringAsync は string 型フラグを評価して値を返す。
    /// </summary>
    // GetStringAsync メソッド実装: OpenFeature FeatureClient.GetStringValue を呼び出す
    public async Task<string> GetStringAsync(string key, EvalContext ec, string defaultVal, CancellationToken cancellationToken = default)
    {
        // EvalContext を OpenFeature EvaluationContext に変換する
        var evalCtx = ToEvaluationContext(ec);
        // OpenFeature 1.x の GetStringValue（Task を返す）を呼び出してフラグを評価する
        return await _client.GetStringValue(key, defaultVal, evalCtx).ConfigureAwait(false);
    }

    /// <summary>
    /// GetStringDetailAsync は string 型フラグを評価して詳細結果を返す。
    /// </summary>
    // GetStringDetailAsync メソッド実装: OpenFeature FeatureClient.GetStringDetails を呼び出す
    public async Task<StringEvalResult> GetStringDetailAsync(string key, EvalContext ec, string defaultVal, CancellationToken cancellationToken = default)
    {
        // EvalContext を OpenFeature EvaluationContext に変換する
        var evalCtx = ToEvaluationContext(ec);
        // OpenFeature 1.x の GetStringDetails（Task を返す）を呼び出して詳細結果を取得する
        var detail = await _client.GetStringDetails(key, defaultVal, evalCtx).ConfigureAwait(false);
        // FlagEvaluationDetails を Library の StringEvalResult に変換して返す
        return new StringEvalResult(
            detail.Value,
            ToFlagEvalReason(detail.Reason),
            detail.Variant ?? string.Empty);
    }

    /// <summary>
    /// GetDoubleAsync は double 型フラグを評価して値を返す。
    /// </summary>
    // GetDoubleAsync メソッド実装: OpenFeature FeatureClient.GetDoubleValue を呼び出す
    public async Task<double> GetDoubleAsync(string key, EvalContext ec, double defaultVal, CancellationToken cancellationToken = default)
    {
        // EvalContext を OpenFeature EvaluationContext に変換する
        var evalCtx = ToEvaluationContext(ec);
        // OpenFeature 1.x の GetDoubleValue（Task を返す）を呼び出してフラグを評価する
        return await _client.GetDoubleValue(key, defaultVal, evalCtx).ConfigureAwait(false);
    }
}

/// <summary>
/// ConfigImpl は IConfigClient の環境変数 / 静的設定 facade 実装クラス。
/// 環境変数を主要ソースとして設定値を返す（Consul / etcd 等への差し替え可能）。
/// WatchAsync は IAsyncEnumerable で設定変更イベントを提供する（Channel ベース）。
/// </summary>
// ConfigImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class ConfigImpl : IConfigClient
{
    // _overrides: テスト / 動的設定用の上書き辞書（コンストラクタで注入する）
    private readonly IReadOnlyDictionary<string, string>? _overrides;

    /// <summary>
    /// コンストラクタ: オプションの上書き辞書を受け取る。
    /// overrides が null の場合は環境変数のみを使用する。
    /// </summary>
    // コンストラクタ: 上書き辞書を注入する（省略可能）
    public ConfigImpl(IReadOnlyDictionary<string, string>? overrides = null)
    {
        // 上書き辞書を保持する（null の場合は環境変数のみを使用する）
        _overrides = overrides;
    }

    // Resolve はキーに対応する値を返す（上書き辞書 → 環境変数 の優先順位）
    private string? Resolve(string key)
    {
        // 上書き辞書に存在する場合は上書き値を返す
        if (_overrides is not null && _overrides.TryGetValue(key, out var over))
        {
            // 上書き値を返す
            return over;
        }
        // 環境変数から取得する（ドット区切りをアンダースコアに変換する）
        var envKey = key.Replace('.', '_').Replace('-', '_').ToUpperInvariant();
        // 環境変数の値を返す（null の場合は null を返す）
        return Environment.GetEnvironmentVariable(envKey);
    }

    /// <summary>
    /// GetStringAsync は string 型設定値を取得する。
    /// キーが存在しない場合は defaultVal を返す。
    /// </summary>
    // GetStringAsync メソッド実装: 環境変数から string 設定値を取得する
    public Task<string> GetStringAsync(string key, string defaultVal, CancellationToken cancellationToken = default)
    {
        // キーを解決する（上書き辞書 → 環境変数）
        var raw = Resolve(key);
        // 値が存在する場合はそのまま返す（存在しない場合は defaultVal を返す）
        return Task.FromResult(raw ?? defaultVal);
    }

    /// <summary>
    /// GetLongAsync は long 型設定値を取得する。
    /// パース失敗または未設定の場合は defaultVal を返す。
    /// </summary>
    // GetLongAsync メソッド実装: 環境変数から long 設定値を取得する
    public Task<long> GetLongAsync(string key, long defaultVal, CancellationToken cancellationToken = default)
    {
        // キーを解決する
        var raw = Resolve(key);
        // raw が null または空の場合は defaultVal を返す
        if (raw is null) return Task.FromResult(defaultVal);
        // long にパースする（失敗した場合は defaultVal を返す）
        return Task.FromResult(long.TryParse(raw, out var v) ? v : defaultVal);
    }

    /// <summary>
    /// GetBoolAsync は bool 型設定値を取得する。
    /// "true" / "1" / "yes" の場合は true を返す（大文字小文字無視）。
    /// </summary>
    // GetBoolAsync メソッド実装: 環境変数から bool 設定値を取得する
    public Task<bool> GetBoolAsync(string key, bool defaultVal, CancellationToken cancellationToken = default)
    {
        // キーを解決する
        var raw = Resolve(key);
        // raw が null の場合は defaultVal を返す
        if (raw is null) return Task.FromResult(defaultVal);
        // "true" / "1" / "yes" の場合は true を返す
        var lower = raw.Trim().ToLowerInvariant();
        // ブール値の判定を行う
        return Task.FromResult(lower is "true" or "1" or "yes");
    }

    /// <summary>
    /// GetDoubleAsync は double 型設定値を取得する。
    /// パース失敗または未設定の場合は defaultVal を返す。
    /// </summary>
    // GetDoubleAsync メソッド実装: 環境変数から double 設定値を取得する
    public Task<double> GetDoubleAsync(string key, double defaultVal, CancellationToken cancellationToken = default)
    {
        // キーを解決する
        var raw = Resolve(key);
        // raw が null の場合は defaultVal を返す
        if (raw is null) return Task.FromResult(defaultVal);
        // double にパースする（失敗した場合は defaultVal を返す）
        return Task.FromResult(double.TryParse(raw, out var v) ? v : defaultVal);
    }

    /// <summary>
    /// GetValueAsync は ConfigValue（メタデータ付き）で設定値を取得する。
    /// キーが存在しない場合は null を返す。
    /// </summary>
    // GetValueAsync メソッド実装: 環境変数から ConfigValue を取得する
    public Task<ConfigValue?> GetValueAsync(string key, CancellationToken cancellationToken = default)
    {
        // キーを解決する
        var raw = Resolve(key);
        // raw が null の場合は null を返す
        if (raw is null) return Task.FromResult<ConfigValue?>(null);
        // ConfigValue を構築して返す（env ソース、バージョン 0）
        return Task.FromResult<ConfigValue?>(new ConfigValue(raw, "env", 0));
    }

    /// <summary>
    /// WatchAsync は設定キーの変更を監視して IAsyncEnumerable でイベントを提供する。
    /// cancellationToken のキャンセルで監視を停止する。
    /// この実装は現在の値を 1 回だけ発行して終了する（ポーリング拡張のプレースホルダ）。
    /// </summary>
    // WatchAsync メソッド実装: 現在の値を 1 回発行して終了する
    public async IAsyncEnumerable<ConfigValue> WatchAsync(string key, [EnumeratorCancellation] CancellationToken cancellationToken = default)
    {
        // 現在の値を取得する
        var current = await GetValueAsync(key, cancellationToken).ConfigureAwait(false);
        // 現在の値が存在する場合は発行する
        if (current is not null)
        {
            // 現在の値を yield する
            yield return current;
        }
        // 監視のプレースホルダ: キャンセルされるまで待機する
        await Task.Delay(Timeout.Infinite, cancellationToken).ConfigureAwait(false);
    }
}
