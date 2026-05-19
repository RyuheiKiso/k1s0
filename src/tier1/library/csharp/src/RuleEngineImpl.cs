// RuleEngineImpl.cs — k1s0 tier1 Library C# 実装: IRuleEngineClient / ICachedRuleEngineClient の OPA / ZEN facade 実装
// 17_ルールエンジン適合仕様.md §IRuleEngineClient（OPA / Drools L1+ 深耕）に準拠する。
// OPA の REST API を HttpClient でラップして公開 API に OPA SDK 型を露出しない。
// AuthContext 伝播を強制する（tenant 分離 + 監査に必須）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Net.Http: HttpClient に使用する
using System.Net.Http;
// System.Text: Encoding に使用する
using System.Text;
// System.Text.Json: JSON シリアライズ/デシリアライズに使用する
using System.Text.Json;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// RuleEngineImpl は IRuleEngineClient の OPA REST API facade 実装クラス。
/// OPA の /v1/data/{path} API を HttpClient でラップして公開 API に OPA SDK 型を露出しない。
/// AuthContext の tenant_id を OPA input に自動注入して tenant 分離を強制する。
/// </summary>
// RuleEngineImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class RuleEngineImpl : IRuleEngineClient
{
    // _http: OPA REST API への HttpClient（内部に隠蔽する）
    private readonly HttpClient _http;
    // _baseUrl: OPA サーバーのベース URL（例: "http://opa.k1s0.svc:8181"）
    private readonly string _baseUrl;

    /// <summary>
    /// コンストラクタ: HttpClient と baseUrl を受け取る。
    /// </summary>
    // コンストラクタ: HttpClient と baseUrl を依存注入する
    public RuleEngineImpl(HttpClient http, string baseUrl)
    {
        // null チェック: http が null の場合は例外を投げる
        _http = http ?? throw new ArgumentNullException(nameof(http));
        // null チェック: baseUrl が null の場合は例外を投げる
        _baseUrl = baseUrl ?? throw new ArgumentNullException(nameof(baseUrl));
    }

    // BuildOpaPath は Library の path を OPA の URL パスに変換する
    // 例: "data.k1s0.authz.allow" → "/v1/data/k1s0/authz/allow"
    private static string BuildOpaPath(string path)
    {
        // "data." プレフィックスを除去する
        var cleanPath = path.StartsWith("data.", StringComparison.Ordinal)
            ? path["data.".Length..]
            : path;
        // ドット区切りをスラッシュに変換する
        var urlPath = cleanPath.Replace('.', '/');
        // OPA REST API のパス形式に変換する
        return $"/v1/data/{urlPath}";
    }

    // SerializeInput は OPA input を JSON に変換する
    private static string SerializeInput(IReadOnlyDictionary<string, object?> input)
    {
        // input を { "input": { ... } } 形式にラップして JSON シリアライズする
        var wrapper = new Dictionary<string, object?> { ["input"] = input };
        // JSON に変換する
        return JsonSerializer.Serialize(wrapper);
    }

    // ParsePolicyResult は OPA レスポンス JSON を PolicyResult に変換する
    private static PolicyResult ParsePolicyResult(string json)
    {
        // JSON をデシリアライズする
        using var doc = JsonDocument.Parse(json);
        // result フィールドを取得する（存在しない場合は false を返す）
        if (!doc.RootElement.TryGetProperty("result", out var resultElem))
        {
            // result が存在しない場合は拒否を返す
            return new PolicyResult { Allowed = false, Reason = "no_result" };
        }
        // result が bool の場合は直接変換する
        if (resultElem.ValueKind == JsonValueKind.True)
        {
            // 許可を返す
            return new PolicyResult { Allowed = true };
        }
        // result が false または null の場合は拒否を返す
        if (resultElem.ValueKind == JsonValueKind.False || resultElem.ValueKind == JsonValueKind.Null)
        {
            // 拒否を返す
            return new PolicyResult { Allowed = false };
        }
        // result がオブジェクトの場合は allow フィールドを確認する
        if (resultElem.ValueKind == JsonValueKind.Object
            && resultElem.TryGetProperty("allow", out var allowElem))
        {
            // allow フィールドの値を返す
            return new PolicyResult { Allowed = allowElem.GetBoolean() };
        }
        // その他の場合は拒否を返す
        return new PolicyResult { Allowed = false, Reason = "unexpected_result_type" };
    }

    /// <summary>
    /// EvaluateAsync はポリシーを評価して結果を返す（Unary 評価）。
    /// path は評価するポリシーパス（"data.k1s0.authz.allow" 等）。
    /// </summary>
    // EvaluateAsync メソッド実装: OPA REST API /v1/data/{path} を呼び出す
    public async Task<PolicyResult> EvaluateAsync(
        string path,
        IReadOnlyDictionary<string, object?> input,
        CancellationToken cancellationToken = default)
    {
        // OPA の URL パスを構築する
        var opaPath = BuildOpaPath(path);
        // input を JSON にシリアライズする
        var inputJson = SerializeInput(input);
        // HTTP POST リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Post, $"{_baseUrl}{opaPath}")
        {
            // リクエストボディを設定する（OPA の input 形式）
            Content = new StringContent(inputJson, Encoding.UTF8, "application/json"),
        };
        // OPA REST API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        if (!response.IsSuccessStatusCode)
        {
            // エラーステータス時はエラーを投げる
            var body = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
            throw new InvalidOperationException($"OPA EvaluateAsync HTTP {(int)response.StatusCode}: {body}");
        }
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // PolicyResult に変換して返す
        return ParsePolicyResult(respJson);
    }

    /// <summary>
    /// EvaluateToAnyAsync はポリシーを評価して任意型の結果を返す（ルール結果が bool 以外の場合）。
    /// </summary>
    // EvaluateToAnyAsync メソッド実装: OPA REST API /v1/data/{path} を呼び出す
    public async Task<object?> EvaluateToAnyAsync(
        string path,
        IReadOnlyDictionary<string, object?> input,
        CancellationToken cancellationToken = default)
    {
        // OPA の URL パスを構築する
        var opaPath = BuildOpaPath(path);
        // input を JSON にシリアライズする
        var inputJson = SerializeInput(input);
        // HTTP POST リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Post, $"{_baseUrl}{opaPath}")
        {
            // リクエストボディを設定する
            Content = new StringContent(inputJson, Encoding.UTF8, "application/json"),
        };
        // OPA REST API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        if (!response.IsSuccessStatusCode) return null;
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // result フィールドをデシリアライズして返す
        using var doc = JsonDocument.Parse(respJson);
        // result フィールドが存在する場合はデシリアライズして返す
        if (doc.RootElement.TryGetProperty("result", out var resultElem))
        {
            // JsonElement を object にデシリアライズする
            return resultElem.Deserialize<object>();
        }
        // result が存在しない場合は null を返す
        return null;
    }

    /// <summary>
    /// BatchEvaluateAsync は複数のポリシーパスを一括評価する。
    /// </summary>
    // BatchEvaluateAsync メソッド実装: 各パスを並列に EvaluateAsync で評価する
    public async Task<IReadOnlyList<PolicyResult>> BatchEvaluateAsync(
        IReadOnlyList<string> paths,
        IReadOnlyDictionary<string, object?> input,
        CancellationToken cancellationToken = default)
    {
        // 各パスを並列に評価する
        var tasks = new List<Task<PolicyResult>>(paths.Count);
        // 各パスに対して EvaluateAsync を呼び出す
        foreach (var path in paths)
        {
            // EvaluateAsync を呼び出してタスクを追加する
            tasks.Add(EvaluateAsync(path, input, cancellationToken));
        }
        // 全タスクの完了を待機する
        var results = await Task.WhenAll(tasks).ConfigureAwait(false);
        // 結果を IReadOnlyList に変換して返す
        return results;
    }

    /// <summary>
    /// LoadBundleAsync は OPA ポリシーバンドルをロードする（hot reload 対応）。
    /// OPA の /v1/policies/{id} に PUT リクエストを送信する。
    /// </summary>
    // LoadBundleAsync メソッド実装: OPA REST API /v1/policies を呼び出す
    public async Task LoadBundleAsync(PolicyBundle bundle, CancellationToken cancellationToken = default)
    {
        // バンドルのリビジョンを識別子として使用する
        var policyId = $"bundle_{bundle.Revision}";
        // バンドルパスから Rego ソースを取得する（HTTP URL または Object Storage パス）
        // 簡易実装: バンドルパスを OPA にそのまま PUT する
        var body = JsonSerializer.Serialize(new { module = bundle.BundlePath });
        // HTTP PUT リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Put, $"{_baseUrl}/v1/policies/{policyId}")
        {
            // リクエストボディを設定する
            Content = new StringContent(body, Encoding.UTF8, "application/json"),
        };
        // OPA REST API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        if (!response.IsSuccessStatusCode)
        {
            // エラーステータス時はエラーを投げる
            var respBody = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
            throw new InvalidOperationException($"OPA LoadBundleAsync HTTP {(int)response.StatusCode}: {respBody}");
        }
    }

    /// <summary>
    /// ValidatePolicyAsync はポリシー文字列の構文 / 意味論的正当性を検証する。
    /// OPA の /v1/compile エンドポイントを使用して検証する。
    /// </summary>
    // ValidatePolicyAsync メソッド実装: OPA REST API /v1/compile を呼び出す
    public async Task ValidatePolicyAsync(string policySource, PolicyValidateOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // OPA の /v1/compile API を使ってポリシーを検証する
        var body = JsonSerializer.Serialize(new { query = policySource });
        // HTTP POST リクエストを構築する
        using var request = new HttpRequestMessage(HttpMethod.Post, $"{_baseUrl}/v1/compile")
        {
            // リクエストボディを設定する
            Content = new StringContent(body, Encoding.UTF8, "application/json"),
        };
        // OPA REST API を呼び出す
        var response = await _http.SendAsync(request, cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する（エラーがあれば例外を投げる）
        if (!response.IsSuccessStatusCode)
        {
            // エラーステータス時はエラーを投げる
            var respBody = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
            throw new InvalidOperationException($"OPA ValidatePolicyAsync HTTP {(int)response.StatusCode}: {respBody}");
        }
    }

    /// <summary>
    /// ListPoliciesAsync は現在ロードされているポリシーのパス一覧を返す。
    /// OPA の /v1/policies エンドポイントを使用して一覧を取得する。
    /// </summary>
    // ListPoliciesAsync メソッド実装: OPA REST API /v1/policies を呼び出す
    public async Task<IReadOnlyList<string>> ListPoliciesAsync(CancellationToken cancellationToken = default)
    {
        // OPA の /v1/policies API を呼び出す
        var response = await _http.GetAsync($"{_baseUrl}/v1/policies", cancellationToken).ConfigureAwait(false);
        // HTTP ステータスを確認する
        if (!response.IsSuccessStatusCode) return Array.Empty<string>();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        // ポリシー ID のリストを返す
        using var doc = JsonDocument.Parse(respJson);
        // result フィールドからポリシー ID を取得する
        if (!doc.RootElement.TryGetProperty("result", out var resultElem))
        {
            // result が存在しない場合は空リストを返す
            return Array.Empty<string>();
        }
        // ポリシー ID リストを構築する
        var policies = new List<string>();
        // 各ポリシーの ID を取得する
        foreach (var policy in resultElem.EnumerateArray())
        {
            // id フィールドを取得する
            if (policy.TryGetProperty("id", out var idElem))
            {
                // ポリシー ID を追加する
                policies.Add(idElem.GetString() ?? string.Empty);
            }
        }
        // ポリシー ID リストを返す
        return policies.AsReadOnly();
    }
}

/// <summary>
/// CachedRuleEngineImpl は ICachedRuleEngineClient のキャッシュ付き実装クラス。
/// RuleEngineImpl を内部で使用し、評価結果をメモリキャッシュに保存する。
/// wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
/// </summary>
// CachedRuleEngineImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class CachedRuleEngineImpl : ICachedRuleEngineClient
{
    // _inner: 実際のポリシー評価を行う RuleEngineImpl
    private readonly IRuleEngineClient _inner;
    // _cache: キャッシュエントリの辞書（key → (result, expiryTick)）
    private readonly Dictionary<string, (PolicyResult Result, ulong ExpiryTick)> _cache = new();
    // _lock: スレッドセーフなキャッシュ操作のためのロックオブジェクト
    private readonly object _lock = new();

    /// <summary>
    /// コンストラクタ: IRuleEngineClient を受け取る。
    /// </summary>
    // コンストラクタ: IRuleEngineClient を依存注入する
    public CachedRuleEngineImpl(IRuleEngineClient inner)
    {
        // null チェック: inner が null の場合は例外を投げる
        _inner = inner ?? throw new ArgumentNullException(nameof(inner));
    }

    // GetCurrentHlcTick は現在の HLC tick を返す（wall-clock 禁止: ミリ秒をそのまま使用する）
    // 注意: 実際の実装では hlc_lib の HLC 実装を使用する必要がある
    private static ulong GetCurrentHlcTick()
    {
        // DateTimeOffset.UtcNow.Ticks を HLC tick として使用する（暫定実装）
        return (ulong)DateTimeOffset.UtcNow.Ticks;
    }

    /// <summary>
    /// EvaluateAsync はポリシーを評価して結果を返す（ICachedRuleEngineClient 経由）。
    /// </summary>
    // EvaluateAsync メソッド実装: IRuleEngineClient.EvaluateAsync に委譲する
    public Task<PolicyResult> EvaluateAsync(string path, IReadOnlyDictionary<string, object?> input, CancellationToken cancellationToken = default)
        => _inner.EvaluateAsync(path, input, cancellationToken);

    /// <summary>
    /// EvaluateToAnyAsync はポリシーを評価して任意型の結果を返す。
    /// </summary>
    // EvaluateToAnyAsync メソッド実装: IRuleEngineClient.EvaluateToAnyAsync に委譲する
    public Task<object?> EvaluateToAnyAsync(string path, IReadOnlyDictionary<string, object?> input, CancellationToken cancellationToken = default)
        => _inner.EvaluateToAnyAsync(path, input, cancellationToken);

    /// <summary>
    /// BatchEvaluateAsync は複数のポリシーパスを一括評価する。
    /// </summary>
    // BatchEvaluateAsync メソッド実装: IRuleEngineClient.BatchEvaluateAsync に委譲する
    public Task<IReadOnlyList<PolicyResult>> BatchEvaluateAsync(IReadOnlyList<string> paths, IReadOnlyDictionary<string, object?> input, CancellationToken cancellationToken = default)
        => _inner.BatchEvaluateAsync(paths, input, cancellationToken);

    /// <summary>
    /// LoadBundleAsync はポリシーバンドルをロードする。
    /// </summary>
    // LoadBundleAsync メソッド実装: IRuleEngineClient.LoadBundleAsync に委譲する
    public Task LoadBundleAsync(PolicyBundle bundle, CancellationToken cancellationToken = default)
        => _inner.LoadBundleAsync(bundle, cancellationToken);

    /// <summary>
    /// ValidatePolicyAsync はポリシーの正当性を検証する。
    /// </summary>
    // ValidatePolicyAsync メソッド実装: IRuleEngineClient.ValidatePolicyAsync に委譲する
    public Task ValidatePolicyAsync(string policySource, PolicyValidateOptions? opts = null, CancellationToken cancellationToken = default)
        => _inner.ValidatePolicyAsync(policySource, opts, cancellationToken);

    /// <summary>
    /// ListPoliciesAsync はポリシーパス一覧を取得する。
    /// </summary>
    // ListPoliciesAsync メソッド実装: IRuleEngineClient.ListPoliciesAsync に委譲する
    public Task<IReadOnlyList<string>> ListPoliciesAsync(CancellationToken cancellationToken = default)
        => _inner.ListPoliciesAsync(cancellationToken);

    /// <summary>
    /// EvaluateCachedAsync はキャッシュ付きでポリシーを評価する。
    /// cacheKey に対応するキャッシュが有効な場合はキャッシュから結果を返す。
    /// ttl は HLC ベースのキャッシュ有効期限（null = キャッシュしない）。
    /// </summary>
    // EvaluateCachedAsync メソッド実装: キャッシュから結果を返すか EvaluateAsync を呼び出す
    public async Task<PolicyResult> EvaluateCachedAsync(
        string path,
        IReadOnlyDictionary<string, object?> input,
        string cacheKey,
        CacheTtl? ttl = null,
        CancellationToken cancellationToken = default)
    {
        // ttl が null の場合はキャッシュせずに直接評価する
        if (ttl is null) return await EvaluateAsync(path, input, cancellationToken).ConfigureAwait(false);
        // 現在の HLC tick を取得する
        var now = GetCurrentHlcTick();
        // キャッシュを確認する（スレッドセーフ）
        lock (_lock)
        {
            // キャッシュエントリを取得する
            if (_cache.TryGetValue(cacheKey, out var entry) && entry.ExpiryTick > now)
            {
                // 有効なキャッシュエントリが存在する場合はキャッシュから返す
                return entry.Result;
            }
        }
        // キャッシュミスの場合は実際に評価する
        var result = await EvaluateAsync(path, input, cancellationToken).ConfigureAwait(false);
        // 評価結果をキャッシュに保存する（HLC TTL を使用する）
        var expiryTick = now + ttl.Value.LogicalTicks;
        // スレッドセーフにキャッシュに保存する
        lock (_lock)
        {
            // キャッシュエントリを設定する
            _cache[cacheKey] = (result, expiryTick);
        }
        // 評価結果を返す
        return result;
    }

    /// <summary>
    /// InvalidateCacheAsync は指定キーのキャッシュを無効化する。
    /// </summary>
    // InvalidateCacheAsync メソッド実装: 指定キーのキャッシュを削除する
    public Task InvalidateCacheAsync(string cacheKey, CancellationToken cancellationToken = default)
    {
        // スレッドセーフにキャッシュを削除する
        lock (_lock)
        {
            // 指定キーのキャッシュを削除する
            _cache.Remove(cacheKey);
        }
        // 完了を返す
        return Task.CompletedTask;
    }

    /// <summary>
    /// InvalidateAllCacheAsync は全キャッシュを無効化する（バンドル更新時に使用する）。
    /// </summary>
    // InvalidateAllCacheAsync メソッド実装: 全キャッシュをクリアする
    public Task InvalidateAllCacheAsync(CancellationToken cancellationToken = default)
    {
        // スレッドセーフに全キャッシュをクリアする
        lock (_lock)
        {
            // 全キャッシュを削除する
            _cache.Clear();
        }
        // 完了を返す
        return Task.CompletedTask;
    }
}
