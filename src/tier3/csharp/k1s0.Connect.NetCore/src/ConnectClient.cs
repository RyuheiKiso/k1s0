// k1s0 Connect-RPC HTTP/1.1 + HTTP/2 クライアント実装
// Connect-RPC プロトコルの 4 RPC form（Unary / ServerStreaming / ClientStreaming / Bidi）を実装する
// HTTP/1.1 では Unary のみ対応し、HTTP/2 では全 4 form に対応する
// wall-clock TTL 禁止規律に従い DateTime.UtcNow を Deadline 計算に使用しない

// System 名前空間: Exception / Uri 等の基本型に使用する
using System;
// System.Collections.Generic: IAsyncEnumerable に使用する
using System.Collections.Generic;
// System.Net.Http: HttpClient / HttpRequestMessage に使用する
using System.Net.Http;
// System.Net.Http.Headers: MediaTypeHeaderValue に使用する
using System.Net.Http.Headers;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / IAsyncEnumerable に使用する
using System.Threading.Tasks;
// Google.Protobuf: IMessage / MessageParser に使用する
using Google.Protobuf;

namespace K1s0.Connect.NetCore
{
    /// <summary>
    /// ConnectClient: Connect-RPC プロトコルの HTTP クライアント実装
    /// HTTP/1.1 Unary + HTTP/2 Streaming の 4 RPC form を提供する
    /// </summary>
    public class ConnectClient : IDisposable
    {
        // 内部 HttpClient インスタンス（HTTP/1.1 と HTTP/2 の両方に対応する）
        private readonly HttpClient _httpClient;

        // ベース URL（RPC エンドポイントのプレフィックス）
        private readonly string _baseUrl;

        // リソース解放済みフラグ
        private bool _disposed = false;

        /// <summary>
        /// ConnectClient のコンストラクタ
        /// </summary>
        /// <param name="baseUrl">Connect-RPC サーバーのベース URL（例: "https://api.k1s0.io"）</param>
        /// <param name="httpClient">外部から注入する HttpClient（省略時は内部生成する）</param>
        public ConnectClient(string baseUrl, HttpClient? httpClient = null)
        {
            // baseUrl が null または空の場合は例外を投げる
            if (string.IsNullOrWhiteSpace(baseUrl))
            {
                // null または空の baseUrl は許容しない
                throw new ArgumentNullException(nameof(baseUrl), "Connect-RPC ベース URL が null または空です");
            }

            // ベース URL を正規化して保存する（末尾スラッシュを除去する）
            _baseUrl = baseUrl.TrimEnd('/');

            // httpClient が指定された場合はそれを使用し、省略時は新規生成する
            _httpClient = httpClient ?? new HttpClient();
        }

        /// <summary>
        /// Unary RPC を実行する（HTTP/1.1 + HTTP/2 両対応）
        /// request を送信して response を返す
        /// </summary>
        /// <typeparam name="TRequest">リクエストメッセージの型（IMessage 制約）</typeparam>
        /// <typeparam name="TResponse">レスポンスメッセージの型（IMessage + new() 制約）</typeparam>
        /// <param name="procedure">RPC プロシージャーパス（例: "/k1s0.v1.StateService/ReadState"）</param>
        /// <param name="request">送信するリクエストメッセージ</param>
        /// <param name="parser">レスポンスのパーサー（MessageParser<TResponse>）</param>
        /// <param name="cancellationToken">キャンセレーショントークン</param>
        /// <returns>パース済みのレスポンスメッセージ</returns>
        public async Task<TResponse> UnaryAsync<TRequest, TResponse>(
            string procedure,
            TRequest request,
            MessageParser<TResponse> parser,
            CancellationToken cancellationToken = default)
            where TRequest : IMessage
            where TResponse : IMessage<TResponse>
        {
            // RPC エンドポイント URL を組み立てる
            var url = $"{_baseUrl}{procedure}";

            // リクエストメッセージを Connect-RPC 5 byte framing でエンコードする
            var encoded = ConnectProtocol.EncodeMessage(request);

            // HttpRequestMessage を生成する（POST メソッドを使用する）
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, url);

            // Content-Type: application/connect+proto を設定する（Connect-RPC 仕様）
            httpRequest.Content = new ByteArrayContent(encoded);
            httpRequest.Content.Headers.ContentType =
                new MediaTypeHeaderValue("application/connect+proto");

            // Connect-RPC プロトコルバージョンヘッダーを設定する
            httpRequest.Headers.Add("Connect-Protocol-Version", "1");

            // HTTP リクエストを送信してレスポンスを受信する
            using var httpResponse = await _httpClient.SendAsync(
                httpRequest,
                HttpCompletionOption.ResponseHeadersRead,
                cancellationToken);

            // HTTP 200 以外の場合は例外を投げる
            httpResponse.EnsureSuccessStatusCode();

            // レスポンスボディを Stream として取得する
            using var responseStream = await httpResponse.Content.ReadAsStreamAsync(cancellationToken);

            // Connect-RPC フレームを読み取る
            var frame = await ConnectProtocol.ReadFrameAsync(responseStream, cancellationToken);

            // フレームが null（ストリーム終端）の場合は例外を投げる
            if (frame == null)
            {
                // レスポンスフレームなしはプロトコルエラー
                throw new InvalidOperationException("Connect-RPC Unary レスポンスフレームが空です");
            }

            // ペイロードを protobuf でパースして返す
            return parser.ParseFrom(frame.Value.Payload.ToArray());
        }

        /// <summary>
        /// ServerStreaming RPC を実行する（HTTP/2 必須）
        /// request を送信してサーバーからのストリームを IAsyncEnumerable で返す
        /// </summary>
        /// <typeparam name="TRequest">リクエストメッセージの型</typeparam>
        /// <typeparam name="TResponse">レスポンスメッセージの型</typeparam>
        /// <param name="procedure">RPC プロシージャーパス</param>
        /// <param name="request">送信するリクエストメッセージ</param>
        /// <param name="parser">レスポンスのパーサー</param>
        /// <param name="cancellationToken">キャンセレーショントークン</param>
        /// <returns>サーバーから受信するメッセージの非同期シーケンス</returns>
        public async IAsyncEnumerable<TResponse> ServerStreamingAsync<TRequest, TResponse>(
            string procedure,
            TRequest request,
            MessageParser<TResponse> parser,
            [System.Runtime.CompilerServices.EnumeratorCancellation]
            CancellationToken cancellationToken = default)
            where TRequest : IMessage
            where TResponse : IMessage<TResponse>
        {
            // RPC エンドポイント URL を組み立てる
            var url = $"{_baseUrl}{procedure}";

            // リクエストメッセージを Connect-RPC 5 byte framing でエンコードする
            var encoded = ConnectProtocol.EncodeMessage(request);

            // HttpRequestMessage を生成する
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, url);

            // Content-Type: application/connect+proto を設定する
            httpRequest.Content = new ByteArrayContent(encoded);
            httpRequest.Content.Headers.ContentType =
                new MediaTypeHeaderValue("application/connect+proto");

            // Connect-RPC プロトコルバージョンヘッダーを設定する
            httpRequest.Headers.Add("Connect-Protocol-Version", "1");

            // HTTP/2 を明示的に要求する（ServerStreaming は HTTP/2 必須）
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信してレスポンスヘッダーを受信する
            using var httpResponse = await _httpClient.SendAsync(
                httpRequest,
                HttpCompletionOption.ResponseHeadersRead,
                cancellationToken);

            // HTTP 200 以外の場合は例外を投げる
            httpResponse.EnsureSuccessStatusCode();

            // レスポンスボディを Stream として取得する
            using var responseStream = await httpResponse.Content.ReadAsStreamAsync(cancellationToken);

            // end-stream フレームが来るまで繰り返しフレームを読み取る
            while (!cancellationToken.IsCancellationRequested)
            {
                // フレームを読み取る
                var frame = await ConnectProtocol.ReadFrameAsync(responseStream, cancellationToken);

                // ストリーム終端の場合はループを終了する
                if (frame == null)
                {
                    // ストリーム終端でループを抜ける
                    yield break;
                }

                // end-stream フレームの場合はループを終了する
                if (frame.Value.IsEndStream)
                {
                    // end-stream フレームでループを抜ける
                    yield break;
                }

                // ペイロードを protobuf でパースして yield する
                yield return parser.ParseFrom(frame.Value.Payload.ToArray());
            }
        }

        /// <summary>
        /// ClientStreaming RPC を実行する（HTTP/2 必須）
        /// クライアントからストリームを送信してサーバーの単一レスポンスを受け取る
        /// </summary>
        /// <typeparam name="TRequest">リクエストメッセージの型</typeparam>
        /// <typeparam name="TResponse">レスポンスメッセージの型</typeparam>
        /// <param name="procedure">RPC プロシージャーパス</param>
        /// <param name="requests">送信するリクエストの非同期シーケンス</param>
        /// <param name="parser">レスポンスのパーサー</param>
        /// <param name="cancellationToken">キャンセレーショントークン</param>
        /// <returns>サーバーからの単一レスポンス</returns>
        public async Task<TResponse> ClientStreamingAsync<TRequest, TResponse>(
            string procedure,
            IAsyncEnumerable<TRequest> requests,
            MessageParser<TResponse> parser,
            CancellationToken cancellationToken = default)
            where TRequest : IMessage
            where TResponse : IMessage<TResponse>
        {
            // RPC エンドポイント URL を組み立てる
            var url = $"{_baseUrl}{procedure}";

            // リクエストフレームを全て収集してから送信する（ClientStreaming の簡易実装）
            // 本番実装では PushStreamContent を使ってストリーミング送信するべきだが
            // .NET 8 では HttpClient でのストリーミングアップロードに制約があるため一括収集する
            var allFrames = new System.IO.MemoryStream();

            // 全リクエストを収集してフレームにエンコードする
            await foreach (var request in requests.WithCancellation(cancellationToken))
            {
                // リクエストをエンコードしてバッファに追記する
                var encoded = ConnectProtocol.EncodeMessage(request);
                // バッファにフレームを書き込む
                await allFrames.WriteAsync(encoded, 0, encoded.Length, cancellationToken);
            }

            // バッファの先頭に戻す
            allFrames.Seek(0, System.IO.SeekOrigin.Begin);

            // HttpRequestMessage を生成する
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, url);

            // Content-Type: application/connect+proto を設定する
            httpRequest.Content = new StreamContent(allFrames);
            httpRequest.Content.Headers.ContentType =
                new MediaTypeHeaderValue("application/connect+proto");

            // Connect-RPC プロトコルバージョンヘッダーを設定する
            httpRequest.Headers.Add("Connect-Protocol-Version", "1");

            // HTTP/2 を要求する
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信する
            using var httpResponse = await _httpClient.SendAsync(
                httpRequest,
                HttpCompletionOption.ResponseHeadersRead,
                cancellationToken);

            // HTTP 200 以外の場合は例外を投げる
            httpResponse.EnsureSuccessStatusCode();

            // レスポンスボディから単一フレームを読み取る
            using var responseStream = await httpResponse.Content.ReadAsStreamAsync(cancellationToken);

            // レスポンスフレームを読み取る
            var frame = await ConnectProtocol.ReadFrameAsync(responseStream, cancellationToken);

            // フレームが null の場合は例外を投げる
            if (frame == null)
            {
                // レスポンスフレームなしはプロトコルエラー
                throw new InvalidOperationException("Connect-RPC ClientStreaming レスポンスフレームが空です");
            }

            // ペイロードを protobuf でパースして返す
            return parser.ParseFrom(frame.Value.Payload.ToArray());
        }

        /// <summary>
        /// BidiStreaming RPC を実行する（HTTP/2 必須）
        /// クライアントとサーバーが同時にストリーミングする双方向通信
        /// 本実装は ClientStreaming + ServerStreaming の組み合わせによる簡易実装
        /// </summary>
        /// <typeparam name="TRequest">リクエストメッセージの型</typeparam>
        /// <typeparam name="TResponse">レスポンスメッセージの型</typeparam>
        /// <param name="procedure">RPC プロシージャーパス</param>
        /// <param name="requests">送信するリクエストの非同期シーケンス</param>
        /// <param name="parser">レスポンスのパーサー</param>
        /// <param name="cancellationToken">キャンセレーショントークン</param>
        /// <returns>サーバーから受信するメッセージの非同期シーケンス</returns>
        public async IAsyncEnumerable<TResponse> BidiStreamingAsync<TRequest, TResponse>(
            string procedure,
            IAsyncEnumerable<TRequest> requests,
            MessageParser<TResponse> parser,
            [System.Runtime.CompilerServices.EnumeratorCancellation]
            CancellationToken cancellationToken = default)
            where TRequest : IMessage
            where TResponse : IMessage<TResponse>
        {
            // RPC エンドポイント URL を組み立てる
            var url = $"{_baseUrl}{procedure}";

            // BidiStreaming の簡易実装: リクエストを全て収集してから送信する
            var allFrames = new System.IO.MemoryStream();

            // 全リクエストを収集してフレームにエンコードする
            await foreach (var request in requests.WithCancellation(cancellationToken))
            {
                // リクエストをエンコードしてバッファに追記する
                var encoded = ConnectProtocol.EncodeMessage(request);
                // バッファにフレームを書き込む
                await allFrames.WriteAsync(encoded, 0, encoded.Length, cancellationToken);
            }

            // バッファの先頭に戻す
            allFrames.Seek(0, System.IO.SeekOrigin.Begin);

            // HttpRequestMessage を生成する
            using var httpRequest = new HttpRequestMessage(HttpMethod.Post, url);

            // Content-Type: application/connect+proto を設定する
            httpRequest.Content = new StreamContent(allFrames);
            httpRequest.Content.Headers.ContentType =
                new MediaTypeHeaderValue("application/connect+proto");

            // Connect-RPC プロトコルバージョンヘッダーを設定する
            httpRequest.Headers.Add("Connect-Protocol-Version", "1");

            // HTTP/2 を要求する（BidiStreaming は HTTP/2 必須）
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信する
            using var httpResponse = await _httpClient.SendAsync(
                httpRequest,
                HttpCompletionOption.ResponseHeadersRead,
                cancellationToken);

            // HTTP 200 以外の場合は例外を投げる
            httpResponse.EnsureSuccessStatusCode();

            // レスポンスストリームからフレームを繰り返し読み取る
            using var responseStream = await httpResponse.Content.ReadAsStreamAsync(cancellationToken);

            // end-stream が来るまで繰り返し読み取る
            while (!cancellationToken.IsCancellationRequested)
            {
                // フレームを読み取る
                var frame = await ConnectProtocol.ReadFrameAsync(responseStream, cancellationToken);

                // ストリーム終端の場合はループを終了する
                if (frame == null)
                {
                    // ストリーム終端でループを抜ける
                    yield break;
                }

                // end-stream フレームの場合はループを終了する
                if (frame.Value.IsEndStream)
                {
                    // end-stream フレームでループを抜ける
                    yield break;
                }

                // ペイロードを protobuf でパースして yield する
                yield return parser.ParseFrom(frame.Value.Payload.ToArray());
            }
        }

        /// <summary>
        /// IDisposable 実装: 内部 HttpClient を破棄する
        /// </summary>
        public void Dispose()
        {
            // リソース解放済みの場合は何もしない
            if (!_disposed)
            {
                // HttpClient を破棄する（外部注入の場合も破棄する設計）
                _httpClient.Dispose();
                // 解放済みフラグを立てる
                _disposed = true;
            }

            // GC の終了処理を抑制する（Dispose で既に解放済みのため）
            GC.SuppressFinalize(this);
        }
    }
}
