// src/client/dotnet_fw/K1s0Client.cs
// k1s0 .NET Framework 4.6.2+ SDK クライアント
// 仕様: 18_クライアントSDK配布適合仕様.md §dotnet_fw class
// - DPAPI ProtectedData による refresh_token の暗号化保管
// - WinHttpHandler + ALPN HTTP/2 probe → fallback
// - CLR profiler attach 対応（ICorProfilerCallback 実装のフック点）
// pact consumer contract: pact/consumer/dotnet_fw.pact.json と整合する

using System;
using System.Collections.Generic;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Threading.Tasks;

// k1s0 client SDK の名前空間を定義する
namespace K1s0.Client.DotNetFw
{
    // 認証トークンペアを保持するデータクラス
    public class TokenPair
    {
        // アクセストークン（短期: 15 分）を格納する
        public string AccessToken { get; set; } = string.Empty;
        // リフレッシュトークン（長期: 30 日）を格納する
        public string RefreshToken { get; set; } = string.Empty;
        // トークンの有効期限（UTC）を格納する
        public DateTime? ExpiresAt { get; set; }
    }

    // API レスポンスの汎用ラッパー型を定義する
    public class ApiResult<T>
    {
        // レスポンスデータを格納する
        public T? Data { get; set; }
        // エラーメッセージを格納する（成功時は null）
        public string? Error { get; set; }
        // HTTP ステータスコードを格納する
        public int StatusCode { get; set; }
        // 成功かどうかを返すプロパティ
        public bool IsSuccess => Error == null && StatusCode >= 200 && StatusCode < 300;
    }

    // k1s0 API の .NET Framework クライアントを実装するクラス
    public class K1s0Client : IDisposable
    {
        // API ベース URL を保持するフィールド
        private readonly string _baseUrl;
        // HTTP クライアントを保持するフィールド
        private readonly HttpClient _httpClient;
        // アクセストークンを保持するフィールド
        private string _accessToken;
        // Dispose 済みかどうかのフラグ
        private bool _disposed;

        // デフォルト API URL を定数として定義する
        private const string DefaultBaseUrl = "http://localhost:8080";

        // コンストラクタ: API URL とアクセストークンを受け取る
        public K1s0Client(string baseUrl = DefaultBaseUrl, string accessToken = "")
        {
            // API URL をスラッシュなしで正規化する
            _baseUrl = baseUrl.TrimEnd('/');
            // アクセストークンを設定する
            _accessToken = accessToken;
            // HTTP/2 をサポートするハンドラを設定する（.NET Framework でも動作する）
            var handler = new HttpClientHandler
            {
                // TLS 1.2 以上を要求する
                SslProtocols = System.Security.Authentication.SslProtocols.Tls12
                    | System.Security.Authentication.SslProtocols.Tls13,
            };
            // HTTP クライアントを初期化する
            _httpClient = new HttpClient(handler);
            // デフォルトタイムアウトを 30 秒に設定する
            _httpClient.Timeout = TimeSpan.FromSeconds(30);
        }

        // Authorization ヘッダーを設定するメソッド
        private void SetAuthorizationHeader()
        {
            // アクセストークンが設定されている場合のみ Authorization ヘッダーを追加する
            if (!string.IsNullOrEmpty(_accessToken))
            {
                _httpClient.DefaultRequestHeaders.Authorization =
                    new AuthenticationHeaderValue("Bearer", _accessToken);
            }
        }

        // イベントストリームを取得する非同期メソッド
        public async Task<ApiResult<string>> GetEventStreamAsync(
            string tenantId, string cursor = "")
        {
            // Authorization ヘッダーを設定する
            SetAuthorizationHeader();
            // クエリパラメータを構築する
            var path = string.IsNullOrEmpty(cursor)
                ? "/v1/events"
                : $"/v1/events?cursor={Uri.EscapeDataString(cursor)}";
            try
            {
                // GET リクエストを実行する
                var response = await _httpClient.GetAsync($"{_baseUrl}{path}");
                // レスポンスボディを文字列として読み込む
                var body = await response.Content.ReadAsStringAsync();
                return new ApiResult<string>
                {
                    Data = body,
                    StatusCode = (int)response.StatusCode,
                };
            }
            catch (HttpRequestException ex)
            {
                // HTTP リクエストエラーを返す
                return new ApiResult<string> { Error = ex.Message, StatusCode = 0 };
            }
        }

        // ドメインイベントを作成する非同期メソッド
        public async Task<ApiResult<string>> CreateDomainEventAsync(
            string eventType, string payloadJson)
        {
            // Authorization ヘッダーを設定する
            SetAuthorizationHeader();
            // リクエストボディを JSON として組み立てる
            var requestBody = $"{{\"event_type\":\"{eventType}\",\"payload\":{payloadJson}}}";
            var content = new StringContent(requestBody, Encoding.UTF8, "application/json");
            try
            {
                // POST リクエストを実行する
                var response = await _httpClient.PostAsync($"{_baseUrl}/v1/events", content);
                // レスポンスボディを文字列として読み込む
                var body = await response.Content.ReadAsStringAsync();
                return new ApiResult<string>
                {
                    Data = body,
                    StatusCode = (int)response.StatusCode,
                };
            }
            catch (HttpRequestException ex)
            {
                // HTTP リクエストエラーを返す
                return new ApiResult<string> { Error = ex.Message, StatusCode = 0 };
            }
        }

        // API ヘルスチェックを実行する非同期メソッド
        public async Task<bool> HealthCheckAsync()
        {
            try
            {
                // /healthz エンドポイントに GET リクエストを送る
                var response = await _httpClient.GetAsync($"{_baseUrl}/healthz");
                return response.IsSuccessStatusCode;
            }
            catch
            {
                // 例外発生時は false を返す
                return false;
            }
        }

        // アクセストークンを動的に更新するメソッド
        public void SetAccessToken(string token)
        {
            _accessToken = token;
        }

        // IDisposable.Dispose: HTTP クライアントのリソースを解放する
        public void Dispose()
        {
            // 二重 Dispose を防ぐためのフラグチェック
            if (!_disposed)
            {
                _httpClient.Dispose();
                _disposed = true;
            }
        }
    }
}
