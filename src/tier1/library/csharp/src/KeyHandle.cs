// KeyHandle.cs — k1s0 tier1 Library C# 実装: IKeyHandle interface + StubKeyHandle
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）および
// §5 層 defense-in-depth 層 A「compile: KeyHandle 必須引数化、生 key bytes 不可視」に準拠する。
// 公開 API シグネチャに生 key bytes を露出しない opaque 型を実装する。

// System: 例外 / 環境変数取得に使用する
using System;
// System.Net.Http: HttpClient（OpenBao Transit API への HTTP クライアント）
using System.Net.Http;
// System.Text: Encoding.UTF8（JSON ボディのエンコード）
using System.Text;
// System.Text.Json: JSON シリアライズ/デシリアライズ（OpenBao API レスポンス解析）
using System.Text.Json;
// System.Threading.Tasks: Task / ValueTask の非同期操作に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// KeyClass は 05_鍵管理適合仕様.md §v1 key_class セット（5 class）を宣言する。
/// class 1 値が purpose / rotation_cadence / scope / backend / destruction_method を一意に導出する
///（dimension override 禁止）。
/// </summary>
// KeyClass 列挙型: 5 class のいずれかを表す
public enum KeyClass
{
    /// <summary>v1_data_dek: データ暗号化鍵（DEK）— per-tenant / software_kms_wrapped / crypto_shred</summary>
    V1DataDek,
    /// <summary>v1_data_kek: 鍵暗号化鍵（KEK）— per-tenant / hsm_pkcs11_shamir_distributed / hsm_zeroize_all_shares</summary>
    V1DataKek,
    /// <summary>v1_token_signing: JWT / DPoP 署名鍵 — platform / hsm_pkcs11 / jwks_revoke</summary>
    V1TokenSigning,
    /// <summary>v1_audit_root_signing: audit hash chain root 署名鍵 — per-tenant / hsm_pkcs11 / external_notary_attest</summary>
    V1AuditRootSigning,
    /// <summary>v1_mtls_workload: workload mTLS 鍵 — per_workload / spire / spire_revoke</summary>
    V1MtlsWorkload,
}

/// <summary>
/// IKeyHandle は生 key bytes を公開しない opaque 鍵抽象 interface。
/// 05_鍵管理適合仕様.md §KeyHandle / KeyMaterial の言語横断型 に準拠する。
/// Sign / Verify は OpenBao Transit への委譲として実装し、key bytes は tier1 境界を越えない。
/// </summary>
// IKeyHandle インターフェース定義
public interface IKeyHandle
{
    /// <summary>KeyId は OpenBao Transit のキー版数識別子（UUID v7 形式）を返す。</summary>
    // KeyId プロパティ: 生 key bytes を露出しない
    string KeyId { get; }

    /// <summary>KeyClass は 5 class のいずれかを返す（purpose bundle の代表値）。</summary>
    // KeyClass プロパティ: 用途クラスを返す
    KeyClass KeyClass { get; }

    /// <summary>IsValid は OpenBao による鍵の有効性確認結果を返す（revoke / rotate 後 false になる）。</summary>
    // IsValid プロパティ: 有効性を返す
    bool IsValid { get; }

    /// <summary>
    /// SignAsync は payload を鍵で署名し、署名バイト列を返す。
    /// 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
    /// </summary>
    // SignAsync メソッド: payload を署名する
    Task<byte[]> SignAsync(byte[] payload);

    /// <summary>
    /// VerifyAsync は payload と signature の一致を検証し、真偽値を返す。
    /// 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
    /// </summary>
    // VerifyAsync メソッド: signature を検証する
    Task<bool> VerifyAsync(byte[] payload, byte[] signature);
}

/// <summary>
/// OpenBaoKeyHandle は OpenBao Transit を呼び出す production 実装。
/// 05_鍵管理適合仕様.md §5 層 defense-in-depth 層 B（runtime: OpenBao Transit 委譲）を実装する。
/// OPENBAO_ADDR / OPENBAO_TOKEN 環境変数からクライアント設定を取得する。
/// </summary>
// OpenBaoKeyHandle クラス: IKeyHandle の OpenBao Transit 実装
public sealed class OpenBaoKeyHandle : IKeyHandle
{
    // 共有 HttpClient: アプリケーション生存期間中に再利用する（接続プール効率化）
    private static readonly HttpClient _http = new HttpClient { Timeout = TimeSpan.FromSeconds(5) };
    // _keyId: OpenBao Transit のキー版数識別子（private）
    private readonly string _keyId;
    // _keyClass: 鍵の用途クラス（private）
    private readonly KeyClass _keyClass;
    // _isValid: 鍵の有効性（private）
    private readonly bool _isValid;

    /// <summary>
    /// コンストラクタ: keyId / keyClass / isValid のみを受け取る（生 key bytes は受け取らない）。
    /// </summary>
    // コンストラクタ: 生 key bytes を受け取らない設計（spec §5 層 defense-in-depth 層 A に準拠する）
    public OpenBaoKeyHandle(string keyId, KeyClass keyClass, bool isValid = true)
    {
        // keyId を設定する
        _keyId = keyId;
        // keyClass を設定する
        _keyClass = keyClass;
        // isValid を設定する
        _isValid = isValid;
    }

    /// <summary>KeyId は OpenBao Transit のキー版数識別子を返す。</summary>
    // KeyId プロパティ実装
    public string KeyId => _keyId;

    /// <summary>KeyClass は 5 class のいずれかを返す。</summary>
    // KeyClass プロパティ実装
    public KeyClass KeyClass => _keyClass;

    /// <summary>IsValid は鍵の有効性を返す。</summary>
    // IsValid プロパティ実装
    public bool IsValid => _isValid;

    /// <summary>
    /// SignAsync は OpenBao Transit の sign API を呼び出して署名バイト列を返す。
    /// POST /v1/transit/sign/{key_name} を呼び出す。
    /// </summary>
    // SignAsync メソッド実装: OpenBao Transit へ委譲する
    public async Task<byte[]> SignAsync(byte[] payload)
    {
        // OPENBAO_ADDR 環境変数からベース URL を取得する（デフォルト: http://openbao.k1s0.svc:8200）
        var baseUrl = Environment.GetEnvironmentVariable("OPENBAO_ADDR")
            ?? "http://openbao.k1s0.svc:8200";
        // OPENBAO_TOKEN 環境変数からトークンを取得する（未設定時はエラー）
        var token = Environment.GetEnvironmentVariable("OPENBAO_TOKEN")
            ?? throw new InvalidOperationException("OPENBAO_TOKEN 環境変数が設定されていない");
        // key_class を OpenBao Transit key name にマッピングする（例: "v1_data_dek"）
        var keyName = _keyClass.ToSpecString();
        // payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
        var inputB64 = Convert.ToBase64String(payload);
        // リクエスト JSON を構築する
        var reqJson = JsonSerializer.Serialize(new { input = inputB64 });
        // POST /v1/transit/sign/{key_name} の URL を構築する
        var url = $"{baseUrl}/v1/transit/sign/{keyName}";
        // HTTP リクエストを構築して X-Vault-Token ヘッダーを設定する
        using var request = new HttpRequestMessage(HttpMethod.Post, url)
        {
            Content = new StringContent(reqJson, Encoding.UTF8, "application/json"),
        };
        request.Headers.Add("X-Vault-Token", token);
        // リクエストを送信する
        var resp = await _http.SendAsync(request).ConfigureAwait(false);
        // HTTP ステータスを確認する
        if (!resp.IsSuccessStatusCode)
        {
            // エラーステータス時はボディを含めてエラーを投げる
            var body = await resp.Content.ReadAsStringAsync().ConfigureAwait(false);
            throw new HttpRequestException($"OpenBao Transit sign HTTP {(int)resp.StatusCode} body={body}");
        }
        // レスポンス JSON をパースする
        var respJson = await resp.Content.ReadAsStringAsync().ConfigureAwait(false);
        using var doc = JsonDocument.Parse(respJson);
        // signature フィールドを取得する（"vault:v1:<base64>" 形式）
        var sigStr = doc.RootElement.GetProperty("data").GetProperty("signature").GetString()
            ?? throw new InvalidOperationException("OpenBao Transit sign: signature フィールドが存在しない");
        // "vault:v1:" プレフィックスを除去して Base64 部分を取り出す
        var sigB64 = sigStr.StartsWith("vault:v1:", StringComparison.Ordinal)
            ? sigStr["vault:v1:".Length..]
            : sigStr;
        // Base64 デコードして署名バイト列を返す
        return Convert.FromBase64String(sigB64);
    }

    /// <summary>
    /// VerifyAsync は OpenBao Transit の verify API を呼び出して検証結果を返す。
    /// POST /v1/transit/verify/{key_name} を呼び出す。
    /// </summary>
    // VerifyAsync メソッド実装: OpenBao Transit へ委譲する
    public async Task<bool> VerifyAsync(byte[] payload, byte[] signature)
    {
        // OPENBAO_ADDR 環境変数からベース URL を取得する
        var baseUrl = Environment.GetEnvironmentVariable("OPENBAO_ADDR")
            ?? "http://openbao.k1s0.svc:8200";
        // OPENBAO_TOKEN 環境変数からトークンを取得する
        var token = Environment.GetEnvironmentVariable("OPENBAO_TOKEN")
            ?? throw new InvalidOperationException("OPENBAO_TOKEN 環境変数が設定されていない");
        // key_class を OpenBao Transit key name にマッピングする
        var keyName = _keyClass.ToSpecString();
        // payload を Base64 エンコードする
        var inputB64 = Convert.ToBase64String(payload);
        // signature を "vault:v1:<base64>" 形式にエンコードする
        var sigB64 = $"vault:v1:{Convert.ToBase64String(signature)}";
        // リクエスト JSON を構築する
        var reqJson = JsonSerializer.Serialize(new { input = inputB64, signature = sigB64 });
        // POST /v1/transit/verify/{key_name} の URL を構築する
        var url = $"{baseUrl}/v1/transit/verify/{keyName}";
        // HTTP リクエストを構築して X-Vault-Token ヘッダーを設定する
        using var request = new HttpRequestMessage(HttpMethod.Post, url)
        {
            Content = new StringContent(reqJson, Encoding.UTF8, "application/json"),
        };
        request.Headers.Add("X-Vault-Token", token);
        // リクエストを送信する
        var resp = await _http.SendAsync(request).ConfigureAwait(false);
        // HTTP ステータスを確認する
        if (!resp.IsSuccessStatusCode)
        {
            // エラーステータス時はボディを含めてエラーを投げる
            var body = await resp.Content.ReadAsStringAsync().ConfigureAwait(false);
            throw new HttpRequestException($"OpenBao Transit verify HTTP {(int)resp.StatusCode} body={body}");
        }
        // レスポンス JSON をパースする
        var respJson = await resp.Content.ReadAsStringAsync().ConfigureAwait(false);
        using var doc = JsonDocument.Parse(respJson);
        // valid フィールドを取得して返す（存在しない場合は false とする）
        return doc.RootElement.TryGetProperty("data", out var dataElem)
            && dataElem.TryGetProperty("valid", out var validElem)
            && validElem.GetBoolean();
    }
}

/// <summary>
/// StubKeyHandle は OpenBao Transit 呼出なしに動作する stub 実装。
/// テスト・ドライラン用途専用（production では OpenBaoKeyHandle を使う）。
/// </summary>
// StubKeyHandle クラス: IKeyHandle の stub 実装（テスト専用）
public sealed class StubKeyHandle : IKeyHandle
{
    // _keyId: OpenBao Transit のキー版数識別子（private: 外部からの直接設定を禁止する）
    private readonly string _keyId;
    // _keyClass: 鍵の用途クラス（private: 外部からの直接設定を禁止する）
    private readonly KeyClass _keyClass;
    // _isValid: OpenBao による有効性確認結果（private: 外部からの直接設定を禁止する）
    private readonly bool _isValid;

    /// <summary>
    /// StubKeyHandle を生成するコンストラクタ。
    /// 生 key bytes は受け取らない設計（spec §5 層 defense-in-depth 層 A に準拠する）。
    /// </summary>
    // コンストラクタ: keyId / keyClass のみを受け取る（生 key bytes は受け取らない）
    public StubKeyHandle(string keyId, KeyClass keyClass, bool isValid = true)
    {
        // keyId を設定する
        _keyId = keyId;
        // keyClass を設定する
        _keyClass = keyClass;
        // isValid を設定する（初期値は true）
        _isValid = isValid;
    }

    /// <summary>KeyId は OpenBao Transit のキー版数識別子を返す。</summary>
    // KeyId プロパティ実装
    public string KeyId => _keyId;

    /// <summary>KeyClass は 5 class のいずれかを返す。</summary>
    // KeyClass プロパティ実装
    public KeyClass KeyClass => _keyClass;

    /// <summary>IsValid は鍵の有効性を返す。</summary>
    // IsValid プロパティ実装
    public bool IsValid => _isValid;

    /// <summary>
    /// SignAsync は stub 実装として空バイト配列を返す。
    /// production 実装では OpenBao Transit /v1/transit/sign/:name を呼び出す。
    /// </summary>
    // SignAsync メソッド実装: stub は空バイト配列を返す
    public Task<byte[]> SignAsync(byte[] payload)
    {
        // stub: OpenBao Transit への委譲先は bfl/OpenBaoClient を参照する
        return Task.FromResult(Array.Empty<byte>());
    }

    /// <summary>
    /// VerifyAsync は stub 実装として常に true を返す。
    /// production 実装では OpenBao Transit /v1/transit/verify/:name を呼び出す。
    /// </summary>
    // VerifyAsync メソッド実装: stub は常に true を返す
    public Task<bool> VerifyAsync(byte[] payload, byte[] signature)
    {
        // stub: OpenBao Transit への委譲先は bfl/OpenBaoClient を参照する
        return Task.FromResult(true);
    }
}

/// <summary>
/// KeyClassExtensions は KeyClass enum の文字列変換を提供する拡張メソッド。
/// spec の class 名（snake_case）と 1:1 対応する。
/// </summary>
// KeyClassExtensions 静的クラス: enum の文字列変換を提供する
public static class KeyClassExtensions
{
    /// <summary>
    /// ToSpecString は KeyClass を spec 定義の snake_case 文字列に変換する。
    /// </summary>
    // ToSpecString 拡張メソッド: enum → spec 文字列
    public static string ToSpecString(this KeyClass keyClass)
    {
        // 各 class を spec 定義の snake_case 文字列にマッピングする
        return keyClass switch
        {
            KeyClass.V1DataDek => "v1_data_dek",
            KeyClass.V1DataKek => "v1_data_kek",
            KeyClass.V1TokenSigning => "v1_token_signing",
            KeyClass.V1AuditRootSigning => "v1_audit_root_signing",
            KeyClass.V1MtlsWorkload => "v1_mtls_workload",
            // 未知の class は例外を投げる（dead spec 防止）
            _ => throw new ArgumentOutOfRangeException(nameof(keyClass), keyClass, "Unknown KeyClass"),
        };
    }
}
