// SecretStoreImpl.cs — k1s0 tier1 Library C# 実装: ISecretStore の OpenBao HTTP API facade 実装
// 05_鍵管理適合仕様.md §SecretStore 抽象 に準拠する。
// OpenBao の KV v2 API を HttpClient でラップして公開 API に生シークレット値を露出しない。
// WithSecretAsync の callback パターンで生シークレット値の漏洩を防止する。

// System: 基本型に使用する
using System;
// System.Net.Http: HttpClient に使用する
using System.Net.Http;
// System.Runtime.InteropServices: MemoryMarshal に使用する
using System.Runtime.InteropServices;
// System.Security.Cryptography: SecureRandom に使用する
using System.Security.Cryptography;
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
/// SecretStoreImpl は ISecretStore の OpenBao KV v2 HTTP API facade 実装クラス。
/// OpenBao の /v1/secret/data/{path} API を HttpClient でラップして公開 API に生シークレット値を露出しない。
/// WithSecretAsync の callback パターンでシークレット値を callback 外に漏洩させない設計とする。
/// </summary>
// SecretStoreImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class SecretStoreImpl : ISecretStore
{
    // _http: OpenBao REST API への HttpClient（内部に隠蔽する）
    private readonly HttpClient _http;
    // _baseUrl: OpenBao サーバーのベース URL（例: "http://openbao.k1s0.svc:8200"）
    private readonly string _baseUrl;
    // _token: OpenBao アクセストークン（環境変数から取得する: 公開 API に露出しない）
    private readonly string _token;
    // _mountPath: KV v2 シークレットマウントパス（デフォルト: "secret"）
    private readonly string _mountPath;

    /// <summary>
    /// コンストラクタ: HttpClient / baseUrl / token / mountPath を受け取る。
    /// token は公開 API に露出しない（コンストラクタ引数のみ）。
    /// </summary>
    // コンストラクタ: 依存関係を注入する（token は生シークレットだが内部でのみ使用する）
    public SecretStoreImpl(HttpClient http, string baseUrl, string token, string mountPath = "secret")
    {
        // null チェック: http が null の場合は例外を投げる
        _http = http ?? throw new ArgumentNullException(nameof(http));
        // null チェック: baseUrl が null の場合は例外を投げる
        _baseUrl = baseUrl ?? throw new ArgumentNullException(nameof(baseUrl));
        // null チェック: token が null の場合は例外を投げる
        _token = token ?? throw new ArgumentNullException(nameof(token));
        // mountPath を保持する
        _mountPath = mountPath ?? "secret";
    }

    // BuildSecretPath は secretId と tenantId から OpenBao KV v2 のパスを構築する
    private string BuildSecretPath(string secretId, string tenantId)
    {
        // "tenants/{tenantId}/secrets/{secretId}" 形式でパスを構築する
        return $"tenants/{tenantId}/secrets/{secretId}";
    }

    // CreateRequest は OpenBao REST API への HttpRequestMessage を構築する
    private HttpRequestMessage CreateRequest(HttpMethod method, string url)
    {
        // HttpRequestMessage を生成する
        var request = new HttpRequestMessage(method, url);
        // X-Vault-Token ヘッダーを設定する（OpenBao の認証ヘッダー）
        request.Headers.Add("X-Vault-Token", _token);
        // 生成した request を返す
        return request;
    }

    /// <summary>
    /// GetMetadataAsync はシークレットのメタデータのみを返す（値は返さない）。
    /// OpenBao KV v2 の /v1/{mount}/metadata/{path} API を使用する。
    /// </summary>
    // GetMetadataAsync メソッド実装: OpenBao KV v2 metadata API を呼び出す
    public async Task<SecretMetadata?> GetMetadataAsync(
        string secretId,
        string tenantId,
        CancellationToken ct = default)
    {
        // OpenBao の KV v2 metadata パスを構築する
        var path = BuildSecretPath(secretId, tenantId);
        // GET /v1/{mount}/metadata/{path} を呼び出す
        using var request = CreateRequest(HttpMethod.Get, $"{_baseUrl}/v1/{_mountPath}/metadata/{path}");
        // OpenBao REST API を呼び出す
        var response = await _http.SendAsync(request, ct).ConfigureAwait(false);
        // 404 の場合は null を返す（シークレットが存在しない）
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound) return null;
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        // JSON をデシリアライズしてメタデータを取得する
        using var doc = JsonDocument.Parse(respJson);
        // data フィールドを取得する
        if (!doc.RootElement.TryGetProperty("data", out var dataElem)) return null;
        // current_version フィールドを取得する（存在しない場合は 1 を使用する）
        var version = dataElem.TryGetProperty("current_version", out var vElem) ? (uint)vElem.GetInt32() : 1u;
        // custom_metadata フィールドからメタデータを取得する
        var isActive = !dataElem.TryGetProperty("destroyed", out var destroyElem) || !destroyElem.GetBoolean();
        // SecretMetadata を構築して返す
        return new SecretMetadata(
            // SecretId を設定する
            SecretId: secretId,
            // Version を設定する
            Version: version,
            // KeyClass を設定する（デフォルト: V1DataDek）
            KeyClass: KeyClass.V1DataDek,
            // TenantId を設定する
            TenantId: tenantId,
            // IsActive を設定する（destroyed でない場合は true）
            IsActive: isActive
        );
    }

    /// <summary>
    /// WithSecretAsync はシークレット値を callback に渡して処理させる。
    /// callback 外にシークレット値が漏れない設計（値は返さない）。
    /// シークレット値は ReadOnlyMemory&lt;byte&gt; として callback に渡す。
    /// </summary>
    // WithSecretAsync メソッド実装: OpenBao KV v2 data API を呼び出して callback にシークレットを渡す
    public async Task<TResult> WithSecretAsync<TResult>(
        string secretId,
        string tenantId,
        Func<ReadOnlyMemory<byte>, TResult> callback,
        CancellationToken ct = default)
    {
        // OpenBao の KV v2 data パスを構築する
        var path = BuildSecretPath(secretId, tenantId);
        // GET /v1/{mount}/data/{path} を呼び出す
        using var request = CreateRequest(HttpMethod.Get, $"{_baseUrl}/v1/{_mountPath}/data/{path}");
        // OpenBao REST API を呼び出す
        var response = await _http.SendAsync(request, ct).ConfigureAwait(false);
        // HTTP ステータスを確認する（404 はシークレット未存在エラー）
        if (response.StatusCode == System.Net.HttpStatusCode.NotFound)
        {
            // シークレットが存在しない場合は例外を投げる
            throw new InvalidOperationException($"WithSecretAsync: シークレットが存在しない secretId={secretId} tenantId={tenantId}");
        }
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        // JSON をデシリアライズしてシークレット値を取得する
        using var doc = JsonDocument.Parse(respJson);
        // data.data.value フィールドを取得する
        if (!doc.RootElement.TryGetProperty("data", out var dataElem)
            || !dataElem.TryGetProperty("data", out var secretDataElem)
            || !secretDataElem.TryGetProperty("value", out var valueElem))
        {
            // シークレット値が存在しない場合は例外を投げる
            throw new InvalidOperationException($"WithSecretAsync: シークレット値フィールドが存在しない secretId={secretId}");
        }
        // Base64 エンコードされたシークレット値をデコードする
        var valueB64 = valueElem.GetString()
            ?? throw new InvalidOperationException($"WithSecretAsync: シークレット値が null secretId={secretId}");
        // Base64 デコードしてバイト列を取得する
        var secretBytes = Convert.FromBase64String(valueB64);
        // callback にシークレットバイト列を渡して結果を取得する（callback 外に漏洩させない）
        var result = callback(secretBytes.AsMemory());
        // セキュリティのためシークレットバイト列をゼロクリアする
        Array.Clear(secretBytes, 0, secretBytes.Length);
        // callback の結果を返す
        return result;
    }

    /// <summary>
    /// RotateAsync はシークレットを新しいバージョンにローテーションする。
    /// OpenBao KV v2 の PUT /v1/{mount}/data/{path} で新しいバージョンを書き込む。
    /// </summary>
    // RotateAsync メソッド実装: OpenBao KV v2 data API でシークレットを更新する
    public async Task<SecretMetadata> RotateAsync(
        string secretId,
        string tenantId,
        CancellationToken ct = default)
    {
        // OpenBao の KV v2 data パスを構築する
        var path = BuildSecretPath(secretId, tenantId);
        // 新しいランダムシークレット値を生成する（32 バイト）
        var newSecretBytes = new byte[32];
        // 暗号学的に安全な乱数で満たす
        RandomNumberGenerator.Fill(newSecretBytes);
        // Base64 エンコードする
        var newValueB64 = Convert.ToBase64String(newSecretBytes);
        // セキュリティのためシークレットバイト列をゼロクリアする
        Array.Clear(newSecretBytes, 0, newSecretBytes.Length);
        // リクエスト JSON を構築する（KV v2 形式: { "data": { "value": "<b64>" } }）
        var body = JsonSerializer.Serialize(new { data = new { value = newValueB64 } });
        // PUT /v1/{mount}/data/{path} を呼び出す
        using var request = CreateRequest(HttpMethod.Put, $"{_baseUrl}/v1/{_mountPath}/data/{path}");
        // リクエストボディを設定する
        request.Content = new StringContent(body, Encoding.UTF8, "application/json");
        // OpenBao REST API を呼び出す
        var response = await _http.SendAsync(request, ct).ConfigureAwait(false);
        // HTTP ステータスを確認する
        response.EnsureSuccessStatusCode();
        // レスポンス JSON を取得する
        var respJson = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
        // 新しいバージョン番号を取得する
        using var doc = JsonDocument.Parse(respJson);
        // data.version フィールドを取得する
        var version = doc.RootElement.TryGetProperty("data", out var dataElem)
            && dataElem.TryGetProperty("version", out var vElem)
            ? (uint)vElem.GetInt32()
            : 1u;
        // 更新された SecretMetadata を返す
        return new SecretMetadata(
            // SecretId を設定する
            SecretId: secretId,
            // 新しい Version を設定する
            Version: version,
            // KeyClass を設定する（デフォルト: V1DataDek）
            KeyClass: KeyClass.V1DataDek,
            // TenantId を設定する
            TenantId: tenantId,
            // ローテーション後は IsActive = true
            IsActive: true
        );
    }

    /// <summary>
    /// RevokeAsync はシークレットを無効化する（IsActive=false にする）。
    /// OpenBao KV v2 の DELETE /v1/{mount}/metadata/{path} でシークレットを削除する。
    /// revoke 後の WithSecretAsync 呼び出しは例外を投げる。
    /// </summary>
    // RevokeAsync メソッド実装: OpenBao KV v2 metadata API でシークレットを削除する
    public async Task RevokeAsync(
        string secretId,
        string tenantId,
        CancellationToken ct = default)
    {
        // OpenBao の KV v2 metadata パスを構築する
        var path = BuildSecretPath(secretId, tenantId);
        // DELETE /v1/{mount}/metadata/{path} を呼び出す（全バージョンを削除する）
        using var request = CreateRequest(HttpMethod.Delete, $"{_baseUrl}/v1/{_mountPath}/metadata/{path}");
        // OpenBao REST API を呼び出す
        var response = await _http.SendAsync(request, ct).ConfigureAwait(false);
        // HTTP ステータスを確認する（404 は idempotent なので無視する）
        if (response.StatusCode != System.Net.HttpStatusCode.NotFound)
        {
            // エラーがある場合は例外を投げる
            response.EnsureSuccessStatusCode();
        }
    }
}
