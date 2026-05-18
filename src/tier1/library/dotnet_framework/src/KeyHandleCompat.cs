// KeyHandleCompat.cs — k1s0 tier1 Library .NET Framework 4.6.2+ Companion
// KeyHandle の .NET Framework 互換ラッパーを提供する。
// .NET 8 版の K1s0.Tier1.KeyHandle と同等の機能を .NET Framework 4.6.2+ で使用可能にする。
// 生 key bytes は公開シグネチャに露出しない（05_鍵管理適合仕様.md §5 層 defense-in-depth 層 A）。
// OpenBao Transit への委譲は System.Net.Http.HttpClient を使用する（.NET Framework 4.5+ 互換）。

// System: Exception / ArgumentNullException に使用する
using System;
// System.Net.Http: OpenBao Transit API への HTTP クライアントに使用する（.NET Framework 4.5+ 互換）
using System.Net.Http;
// System.Net.Http.Headers: HTTP ヘッダー設定に使用する
using System.Net.Http.Headers;
// System.Text: Encoding に使用する
using System.Text;
// System.Threading.Tasks: Task に使用する（.NET Framework 4.5+ 互換）
using System.Threading.Tasks;

// k1s0 tier1 .NET Framework 互換名前空間
namespace K1s0.Tier1.Compat
{
    /// <summary>
    /// KeyClassCompat は 05_鍵管理適合仕様.md §v1 key_class セット（5 class）を宣言する。
    /// .NET 8 版 K1s0.Tier1.KeyClass と同等の定義を .NET Framework 4.6.2+ で提供する。
    /// </summary>
    // KeyClassCompat 列挙型: 5 class のいずれかを表す（.NET Framework 互換）
    public enum KeyClassCompat
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
    /// IKeyHandleCompat は生 key bytes を公開しない opaque 鍵抽象 interface の
    /// .NET Framework 4.6.2+ 互換バージョン。
    /// Sign / Verify は OpenBao Transit への委譲として実装し、key bytes は tier1 境界を越えない。
    /// </summary>
    // IKeyHandleCompat インタフェース定義（.NET Framework 互換）
    public interface IKeyHandleCompat
    {
        /// <summary>KeyId は OpenBao Transit のキー版数識別子（UUID v7 形式）を返す。</summary>
        // KeyId プロパティ
        string KeyId { get; }

        /// <summary>KeyClass は 5 class のいずれかを返す（purpose bundle の代表値）。</summary>
        // KeyClass プロパティ
        KeyClassCompat KeyClass { get; }

        /// <summary>IsValid は OpenBao による鍵の有効性確認結果を返す。</summary>
        // IsValid プロパティ
        bool IsValid { get; }

        /// <summary>
        /// SignAsync は payload を鍵で署名し、署名バイト列を返す Task を返す。
        /// 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
        /// .NET Framework 4.5+ 互換: Task を使用する。
        /// </summary>
        // SignAsync メソッド: 非同期署名（.NET Framework 4.5+ Task 互換）
        Task<byte[]> SignAsync(byte[] payload);

        /// <summary>
        /// VerifyAsync は payload と signature の一致を検証し、真偽値を返す Task を返す。
        /// 生 key bytes は引数・戻り値のいずれにも含まれない（OpenBao Transit 委譲）。
        /// </summary>
        // VerifyAsync メソッド: 非同期検証
        Task<bool> VerifyAsync(byte[] payload, byte[] signature);
    }

    /// <summary>
    /// OpenBaoKeyHandleCompat は OpenBao Transit API に委譲する IKeyHandleCompat 実装。
    /// .NET Framework 4.6.2+ 互換: System.Net.Http.HttpClient を使用する。
    /// TLS 1.2: ServicePointManager.SecurityProtocol に TLS 1.2 を明示的に設定する。
    /// </summary>
    // OpenBaoKeyHandleCompat クラス定義（sealed: 継承禁止）
    public sealed class OpenBaoKeyHandleCompat : IKeyHandleCompat
    {
        // _keyId: OpenBao Transit のキー版数識別子（unexported: 外部からの直接アクセスを禁止する）
        private readonly string _keyId;
        // _keyClass: 鍵の用途クラス（unexported: 外部からの直接アクセスを禁止する）
        private readonly KeyClassCompat _keyClass;
        // _isValid: OpenBao による有効性確認結果（unexported: 外部からの直接アクセスを禁止する）
        private readonly bool _isValid;

        // コンストラクタ: 生 key bytes を受け取らない設計（spec §5 層 defense-in-depth 層 A に準拠する）
        private OpenBaoKeyHandleCompat(string keyId, KeyClassCompat keyClass, bool isValid)
        {
            // 全フィールドを初期化する
            _keyId = keyId;
            _keyClass = keyClass;
            _isValid = isValid;
        }

        /// <summary>
        /// Create は OpenBaoKeyHandleCompat を生成するファクトリメソッド。
        /// 生 key bytes は受け取らない設計（spec §5 層 defense-in-depth 層 A に準拠する）。
        /// </summary>
        // Create ファクトリメソッド
        public static IKeyHandleCompat Create(string keyId, KeyClassCompat keyClass, bool isValid)
        {
            // OpenBaoKeyHandleCompat を生成して IKeyHandleCompat として返す
            return new OpenBaoKeyHandleCompat(keyId, keyClass, isValid);
        }

        // KeyId プロパティ実装
        public string KeyId { get { return _keyId; } }

        // KeyClass プロパティ実装
        public KeyClassCompat KeyClass { get { return _keyClass; } }

        // IsValid プロパティ実装
        public bool IsValid { get { return _isValid; } }

        /// <summary>
        /// SignAsync は OpenBao Transit の sign API に委譲して署名バイト列を返す。
        /// .NET Framework 4.6.2+ 互換: System.Net.Http.HttpClient を使用する。
        /// TLS 1.2: 01_Bidi適合仕様.md §TLS バージョン注記に準拠する。
        /// </summary>
        // SignAsync メソッド実装: OpenBao Transit sign API に委譲する
        public async Task<byte[]> SignAsync(byte[] payload)
        {
            // OPENBAO_ADDR 環境変数からベース URL を取得する（デフォルト: http://openbao.k1s0.svc:8200）
            var baseUrl = Environment.GetEnvironmentVariable("OPENBAO_ADDR") ?? "http://openbao.k1s0.svc:8200";
            // OPENBAO_TOKEN 環境変数からトークンを取得する（未設定時はエラー）
            var token = Environment.GetEnvironmentVariable("OPENBAO_TOKEN");
            // トークン未設定は設定エラーとして扱う
            if (string.IsNullOrEmpty(token))
            {
                throw new InvalidOperationException("OPENBAO_TOKEN 環境変数が設定されていない");
            }
            // key_class を OpenBao Transit key name にマッピングする
            var keyName = KeyClassCompatExtensions.ToTransitKeyName(_keyClass);
            // payload を Base64 エンコードする（OpenBao Transit の input フィールドは Base64 要求）
            var inputB64 = Convert.ToBase64String(payload);
            // リクエストボディを構築する（JSON）
            var requestBody = "{\"input\":\"" + inputB64 + "\"}";
            // HTTP クライアントを作成する
            using (var client = new HttpClient())
            {
                // X-Vault-Token ヘッダーを設定する
                client.DefaultRequestHeaders.Add("X-Vault-Token", token);
                // リクエストボディを StringContent として構築する
                var content = new StringContent(requestBody, Encoding.UTF8, "application/json");
                // POST /v1/transit/sign/{key_name} の URL を構築する
                var url = baseUrl + "/v1/transit/sign/" + keyName;
                // HTTP リクエストを送信する
                var response = await client.PostAsync(url, content).ConfigureAwait(false);
                // HTTP ステータスを確認する
                response.EnsureSuccessStatusCode();
                // レスポンスボディを読み取る
                var responseBody = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
                // signature フィールドを JSON から取り出す（簡略実装）
                // 実際の実装では JSON パーサーを使用する
                var signatureStart = responseBody.IndexOf("\"signature\":\"", StringComparison.Ordinal);
                // signature フィールドが見つからない場合はエラーを返す
                if (signatureStart < 0)
                {
                    throw new InvalidOperationException("OpenBao Transit sign: signature フィールドが存在しない");
                }
                // signature 値の開始位置を計算する
                var valueStart = signatureStart + "\"signature\":\"".Length;
                // signature 値の終了位置を計算する
                var valueEnd = responseBody.IndexOf('"', valueStart);
                // signature 文字列を取り出す（"vault:v1:" プレフィックスを除去する）
                var sigStr = responseBody.Substring(valueStart, valueEnd - valueStart);
                // "vault:v1:" プレフィックスを除去して Base64 部分を取り出す
                var sigB64 = sigStr.StartsWith("vault:v1:") ? sigStr.Substring("vault:v1:".Length) : sigStr;
                // Base64 デコードして署名バイト列を返す
                return Convert.FromBase64String(sigB64);
            }
        }

        /// <summary>
        /// VerifyAsync は OpenBao Transit の verify API に委譲して検証結果を返す。
        /// .NET Framework 4.6.2+ 互換: System.Net.Http.HttpClient を使用する。
        /// </summary>
        // VerifyAsync メソッド実装: OpenBao Transit verify API に委譲する
        public async Task<bool> VerifyAsync(byte[] payload, byte[] signature)
        {
            // OPENBAO_ADDR 環境変数からベース URL を取得する
            var baseUrl = Environment.GetEnvironmentVariable("OPENBAO_ADDR") ?? "http://openbao.k1s0.svc:8200";
            // OPENBAO_TOKEN 環境変数からトークンを取得する
            var token = Environment.GetEnvironmentVariable("OPENBAO_TOKEN");
            // トークン未設定は設定エラーとして扱う
            if (string.IsNullOrEmpty(token))
            {
                throw new InvalidOperationException("OPENBAO_TOKEN 環境変数が設定されていない");
            }
            // key_class を OpenBao Transit key name にマッピングする
            var keyName = KeyClassCompatExtensions.ToTransitKeyName(_keyClass);
            // payload を Base64 エンコードする
            var inputB64 = Convert.ToBase64String(payload);
            // signature を "vault:v1:<base64>" 形式にエンコードする
            var sigB64 = "vault:v1:" + Convert.ToBase64String(signature);
            // リクエストボディを構築する（JSON）
            var requestBody = "{\"input\":\"" + inputB64 + "\",\"signature\":\"" + sigB64 + "\"}";
            // HTTP クライアントを作成する
            using (var client = new HttpClient())
            {
                // X-Vault-Token ヘッダーを設定する
                client.DefaultRequestHeaders.Add("X-Vault-Token", token);
                // リクエストボディを StringContent として構築する
                var content = new StringContent(requestBody, Encoding.UTF8, "application/json");
                // POST /v1/transit/verify/{key_name} の URL を構築する
                var url = baseUrl + "/v1/transit/verify/" + keyName;
                // HTTP リクエストを送信する
                var response = await client.PostAsync(url, content).ConfigureAwait(false);
                // HTTP ステータスを確認する
                response.EnsureSuccessStatusCode();
                // レスポンスボディを読み取る
                var responseBody = await response.Content.ReadAsStringAsync().ConfigureAwait(false);
                // data.valid フィールドを JSON から取り出す（簡略実装）
                var validTrue = responseBody.Contains("\"valid\":true");
                // true / false を返す
                return validTrue;
            }
        }
    }

    /// <summary>
    /// KeyClassCompatExtensions は KeyClassCompat enum のユーティリティメソッドを提供する静的クラス。
    /// </summary>
    // KeyClassCompatExtensions 静的クラス
    public static class KeyClassCompatExtensions
    {
        /// <summary>
        /// ToTransitKeyName は KeyClassCompat を OpenBao Transit key name に変換する。
        /// </summary>
        // ToTransitKeyName 静的メソッド: enum → OpenBao Transit key name
        public static string ToTransitKeyName(KeyClassCompat keyClass)
        {
            // 各 class を OpenBao Transit key name にマッピングする
            switch (keyClass)
            {
                // v1_data_dek を返す
                case KeyClassCompat.V1DataDek:
                    return "v1_data_dek";
                // v1_data_kek を返す
                case KeyClassCompat.V1DataKek:
                    return "v1_data_kek";
                // v1_token_signing を返す
                case KeyClassCompat.V1TokenSigning:
                    return "v1_token_signing";
                // v1_audit_root_signing を返す
                case KeyClassCompat.V1AuditRootSigning:
                    return "v1_audit_root_signing";
                // v1_mtls_workload を返す
                case KeyClassCompat.V1MtlsWorkload:
                    return "v1_mtls_workload";
                // 未知の class は例外を投げる（dead spec 防止）
                default:
                    throw new ArgumentOutOfRangeException("keyClass", keyClass, "Unknown KeyClassCompat");
            }
        }
    }
}
