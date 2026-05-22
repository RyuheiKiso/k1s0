// k1s0-impl: IMPL-cross_edge-0004 realizes=FR-cross_edge-004
// k1s0 Connect-RPC .NET 8 inhouse 実装
// Connect Conformance Suite 全 case green を目標とする C# 実装
// Unary / Server-Streaming / Client-Streaming / Bidi-Streaming を全て実装する
using System;
// System.Buffers は高パフォーマンスバッファ操作に使用する
using System.Buffers;
// System.Collections.Generic はコレクション型に使用する
using System.Collections.Generic;
// System.IO は Stream 操作に使用する
using System.IO;
// System.Net.Http は HTTP クライアントに使用する
using System.Net.Http;
// System.Net.Http.Headers は HTTP ヘッダーに使用する
using System.Net.Http.Headers;
// System.Runtime.CompilerServices は非同期ストリームに使用する
using System.Runtime.CompilerServices;
// System.Text は UTF-8 エンコードに使用する
using System.Text;
// System.Text.Json は JSON シリアライズに使用する
using System.Text.Json;
// System.Threading は同期/非同期制御に使用する
using System.Threading;
// System.Threading.Tasks は非同期処理に使用する
using System.Threading.Tasks;
// System.IO.Pipelines は高パフォーマンス I/O に使用する
using System.IO.Pipelines;

// k1s0 Connect-RPC inhouse 実装の名前空間を定義する
namespace K1s0.Connect.NetCore
{
    // Connect-RPC のコンテンツタイプを定義する定数クラス
    public static class ConnectContentType
    {
        // Connect Unary レスポンスの JSON コンテンツタイプ
        public const string ConnectJson = "application/connect+json";
        // Connect Unary レスポンスの Protobuf コンテンツタイプ
        public const string ConnectProto = "application/connect+proto";
        // Connect Streaming の JSON コンテンツタイプ
        public const string ConnectStreamingJson = "application/connect+json";
        // Connect Streaming の Protobuf コンテンツタイプ
        public const string ConnectStreamingProto = "application/connect+proto";
        // gRPC のコンテンツタイプ
        public const string GrpcProto = "application/grpc+proto";
        // gRPC-Web のコンテンツタイプ
        public const string GrpcWebProto = "application/grpc-web+proto";
    }

    // Connect-RPC のエラーコードを定義する列挙型
    public enum ConnectErrorCode
    {
        // キャンセル: 操作がキャンセルされた
        Canceled = 0,
        // 不明なエラー
        Unknown = 1,
        // 引数が無効
        InvalidArgument = 2,
        // 期限切れ
        DeadlineExceeded = 3,
        // リソースが見つからない
        NotFound = 4,
        // リソースがすでに存在する
        AlreadyExists = 5,
        // 権限が不足している
        PermissionDenied = 6,
        // リソースが不足している
        ResourceExhausted = 7,
        // 事前条件が満たされていない
        FailedPrecondition = 8,
        // 操作が中断された
        Aborted = 9,
        // 範囲外の値
        OutOfRange = 10,
        // 実装されていない
        Unimplemented = 11,
        // 内部エラー
        Internal = 12,
        // サービスが利用できない
        Unavailable = 13,
        // データが破損している
        DataLoss = 14,
        // 認証が必要
        Unauthenticated = 15,
    }

    // Connect-RPC のエラーを表す例外クラス
    public class ConnectException : Exception
    {
        // エラーコード: Connect-RPC のエラーコード
        public ConnectErrorCode Code { get; }

        // エラー詳細: エラーの詳細情報 (nullable)
        public IReadOnlyList<object>? Details { get; }

        // コンストラクタ: エラーコードとメッセージを受け取る
        public ConnectException(ConnectErrorCode code, string message)
            // 基底クラスのコンストラクタを呼び出す
            : base(message)
        {
            // エラーコードを設定する
            Code = code;
            // 詳細を null に設定する
            Details = null;
        }

        // コンストラクタ: エラーコード、メッセージ、詳細を受け取る
        public ConnectException(ConnectErrorCode code, string message, IReadOnlyList<object>? details)
            // 基底クラスのコンストラクタを呼び出す
            : base(message)
        {
            // エラーコードを設定する
            Code = code;
            // 詳細を設定する
            Details = details;
        }
    }

    // Connect-RPC の Unary レスポンスを表すジェネリッククラス
    public sealed class ConnectUnaryResponse<TResponse>
    {
        // レスポンスメッセージ: Unary 呼び出しの戻り値
        public TResponse Message { get; }

        // レスポンスヘッダー: gRPC/Connect ヘッダー
        public IReadOnlyDictionary<string, string> Headers { get; }

        // レスポンストレーラー: gRPC/Connect トレーラー (ステータスを含む)
        public IReadOnlyDictionary<string, string> Trailers { get; }

        // コンストラクタ: メッセージ、ヘッダー、トレーラーを受け取る
        public ConnectUnaryResponse(
            // レスポンスメッセージを受け取る
            TResponse message,
            // ヘッダーを受け取る
            IReadOnlyDictionary<string, string> headers,
            // トレーラーを受け取る
            IReadOnlyDictionary<string, string> trailers)
        {
            // メッセージを設定する
            Message = message;
            // ヘッダーを設定する
            Headers = headers;
            // トレーラーを設定する
            Trailers = trailers;
        }
    }

    // Connect-RPC メッセージのシリアライズ/デシリアライズを担当するクラス
    public static class ConnectSerializer
    {
        // SerializeToJsonBytes はメッセージを JSON バイト配列にシリアライズする
        public static byte[] SerializeToJsonBytes<T>(T message)
        {
            // System.Text.Json で JSON にシリアライズする
            return JsonSerializer.SerializeToUtf8Bytes(message);
        }

        // DeserializeFromJsonBytes は JSON バイト配列をメッセージにデシリアライズする
        public static T DeserializeFromJsonBytes<T>(byte[] bytes)
        {
            // System.Text.Json で JSON からデシリアライズする
            var result = JsonSerializer.Deserialize<T>(bytes);
            // デシリアライズ結果が null の場合は例外を発生させる
            if (result == null)
            {
                // デシリアライズ失敗の場合は ConnectException を発生させる
                throw new ConnectException(ConnectErrorCode.Internal, "JSON デシリアライズに失敗した");
            }
            // デシリアライズ結果を返す
            return result;
        }

        // WriteEnvelopedMessage は Connect-RPC のエンベロープ形式でメッセージを書き込む
        // 5 バイトのヘッダー (flags + length) に続いてメッセージボディを書き込む
        public static async Task WriteEnvelopedMessageAsync(
            // 書き込み先の Stream
            Stream stream,
            // メッセージバイト配列
            byte[] messageBytes,
            // エンドストリームフラグ: true の場合は EOS フラグを設定する
            bool endStream = false,
            // キャンセルトークン
            CancellationToken cancellationToken = default)
        {
            // 5 バイトのエンベロープヘッダーを作成する
            var header = new byte[5];
            // フラグバイト: 通常メッセージは 0x00、EOS フラグは 0x02 を設定する
            header[0] = endStream ? (byte)0x02 : (byte)0x00;
            // メッセージ長を 4 バイトのビッグエンディアンで書き込む
            var length = messageBytes.Length;
            // 上位バイトから順に書き込む
            header[1] = (byte)(length >> 24);
            // 2 番目のバイトを書き込む
            header[2] = (byte)(length >> 16);
            // 3 番目のバイトを書き込む
            header[3] = (byte)(length >> 8);
            // 最下位バイトを書き込む
            header[4] = (byte)(length & 0xFF);
            // ヘッダーを Stream に書き込む
            await stream.WriteAsync(header, 0, 5, cancellationToken).ConfigureAwait(false);
            // メッセージボディを Stream に書き込む
            await stream.WriteAsync(messageBytes, 0, messageBytes.Length, cancellationToken).ConfigureAwait(false);
            // Stream をフラッシュして送信を確実にする
            await stream.FlushAsync(cancellationToken).ConfigureAwait(false);
        }

        // ReadEnvelopedMessageAsync は Connect-RPC のエンベロープ形式でメッセージを読み取る
        public static async Task<(byte[] message, bool endStream)> ReadEnvelopedMessageAsync(
            // 読み取り元の Stream
            Stream stream,
            // キャンセルトークン
            CancellationToken cancellationToken = default)
        {
            // 5 バイトのヘッダーバッファを確保する
            var header = new byte[5];
            // ヘッダーを読み取る
            var headerBytesRead = await stream.ReadAsync(header, 0, 5, cancellationToken).ConfigureAwait(false);
            // ヘッダーが読み取れなかった場合はストリームが終了したとして扱う
            if (headerBytesRead == 0)
            {
                // 空メッセージと EOS フラグを返す
                return (Array.Empty<byte>(), true);
            }
            // フラグバイトを取得する
            var flags = header[0];
            // EOS フラグが設定されているかどうかを確認する
            var endStream = (flags & 0x02) != 0;
            // メッセージ長を 4 バイトのビッグエンディアンから取得する
            var length = (header[1] << 24) | (header[2] << 16) | (header[3] << 8) | header[4];
            // メッセージバッファを確保する
            var messageBytes = new byte[length];
            // メッセージボディを読み取る
            var totalRead = 0;
            // メッセージを全部読み取るまでループする
            while (totalRead < length)
            {
                // 残りのバイト数を計算する
                var remaining = length - totalRead;
                // メッセージボディの続きを読み取る
                var read = await stream.ReadAsync(messageBytes, totalRead, remaining, cancellationToken).ConfigureAwait(false);
                // 読み取りが終了した場合はループを抜ける
                if (read == 0) break;
                // 読み取ったバイト数を累積する
                totalRead += read;
            }
            // メッセージと EOS フラグを返す
            return (messageBytes, endStream);
        }
    }

    // IConnectClient は Connect-RPC クライアントのインターフェイスを定義する
    public interface IConnectClient
    {
        // UnaryAsync は Unary 呼び出しを非同期で実行する
        Task<ConnectUnaryResponse<TResponse>> UnaryAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージ
            TRequest request,
            // キャンセルトークン
            CancellationToken cancellationToken = default);

        // ServerStreamAsync はサーバー側ストリーミングを非同期で実行する
        IAsyncEnumerable<TResponse> ServerStreamAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージ
            TRequest request,
            // キャンセルトークン
            CancellationToken cancellationToken = default);

        // ClientStreamAsync はクライアント側ストリーミングを非同期で実行する
        Task<ConnectUnaryResponse<TResponse>> ClientStreamAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージのストリーム
            IAsyncEnumerable<TRequest> requests,
            // キャンセルトークン
            CancellationToken cancellationToken = default);

        // BidiStreamAsync は双方向ストリーミングを非同期で実行する
        IAsyncEnumerable<TResponse> BidiStreamAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージのストリーム
            IAsyncEnumerable<TRequest> requests,
            // キャンセルトークン
            CancellationToken cancellationToken = default);
    }

    // K1s0ConnectClient は Connect-RPC クライアントの inhouse 実装クラス
    public class K1s0ConnectClient : IConnectClient
    {
        // HTTP クライアント: Connect-RPC の HTTP/2 通信に使用する
        private readonly HttpClient _httpClient;

        // サービスベース URL: Connect-RPC サービスのエンドポイント
        private readonly string _baseUrl;

        // コンストラクタ: HttpClient とサービスベース URL を受け取る
        public K1s0ConnectClient(HttpClient httpClient, string baseUrl)
        {
            // HttpClient を設定する
            _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
            // ベース URL を設定する (末尾のスラッシュを除去する)
            _baseUrl = baseUrl.TrimEnd('/');
        }

        // UnaryAsync は Unary 呼び出しを非同期で実行する
        public async Task<ConnectUnaryResponse<TResponse>> UnaryAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージ
            TRequest request,
            // キャンセルトークン
            CancellationToken cancellationToken = default)
        {
            // リクエストを JSON にシリアライズする
            var requestBytes = ConnectSerializer.SerializeToJsonBytes(request);
            // HTTP リクエストを作成する
            using var httpRequest = new HttpRequestMessage(
                // POST メソッドを使用する
                HttpMethod.Post,
                // メソッドのフルパスを組み立てる
                $"{_baseUrl}/{methodPath.TrimStart('/')}")
            {
                // リクエストボディを設定する
                Content = new ByteArrayContent(requestBytes),
            };
            // Connect-JSON コンテンツタイプを設定する
            httpRequest.Content.Headers.ContentType = new MediaTypeHeaderValue(ConnectContentType.ConnectJson)
            {
                // UTF-8 文字セットを設定する
                CharSet = "utf-8",
            };
            // Accept ヘッダーを設定する
            httpRequest.Headers.Accept.Add(new MediaTypeWithQualityHeaderValue(ConnectContentType.ConnectJson));
            // HTTP/2 バージョンを要求する
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信する
            using var httpResponse = await _httpClient.SendAsync(
                // HTTP リクエストを送信する
                httpRequest,
                // レスポンスヘッダーのみ受信してボディは後で読み取る
                HttpCompletionOption.ResponseHeadersRead,
                // キャンセルトークンを渡す
                cancellationToken).ConfigureAwait(false);

            // レスポンスが成功かどうかを確認する
            if (!httpResponse.IsSuccessStatusCode)
            {
                // エラーステータスコードの場合は ConnectException を発生させる
                var errorCode = MapHttpStatusToConnectError(httpResponse.StatusCode);
                // ConnectException を発生させる
                throw new ConnectException(errorCode, $"HTTP エラー: {(int)httpResponse.StatusCode}");
            }

            // レスポンスボディを読み取る
            var responseBytes = await httpResponse.Content.ReadAsByteArrayAsync(cancellationToken).ConfigureAwait(false);
            // レスポンスをデシリアライズする
            var responseMessage = ConnectSerializer.DeserializeFromJsonBytes<TResponse>(responseBytes);
            // レスポンスヘッダーを取得する
            var responseHeaders = ExtractHeaders(httpResponse.Headers);
            // レスポンストレーラーを取得する
            var responseTrailers = ExtractTrailers(httpResponse.TrailingHeaders);

            // ConnectUnaryResponse を返す
            return new ConnectUnaryResponse<TResponse>(responseMessage, responseHeaders, responseTrailers);
        }

        // ServerStreamAsync はサーバー側ストリーミングを非同期で実行する
        public async IAsyncEnumerable<TResponse> ServerStreamAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージ
            TRequest request,
            // キャンセルトークン (EnumeratorCancellation 属性を付与する)
            [EnumeratorCancellation] CancellationToken cancellationToken = default)
        {
            // リクエストを JSON にシリアライズする
            var requestBytes = ConnectSerializer.SerializeToJsonBytes(request);
            // HTTP リクエストを作成する
            using var httpRequest = new HttpRequestMessage(
                // POST メソッドを使用する
                HttpMethod.Post,
                // メソッドのフルパスを組み立てる
                $"{_baseUrl}/{methodPath.TrimStart('/')}")
            {
                // リクエストボディを設定する
                Content = new ByteArrayContent(requestBytes),
            };
            // Connect-JSON コンテンツタイプを設定する
            httpRequest.Content.Headers.ContentType = new MediaTypeHeaderValue(ConnectContentType.ConnectJson);
            // HTTP/2 バージョンを要求する
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信する (レスポンスヘッダーのみ受信)
            using var httpResponse = await _httpClient.SendAsync(
                // HTTP リクエストを送信する
                httpRequest,
                // レスポンスヘッダーのみ受信してボディは後で読み取る
                HttpCompletionOption.ResponseHeadersRead,
                // キャンセルトークンを渡す
                cancellationToken).ConfigureAwait(false);

            // レスポンスストリームを取得する
            var responseStream = await httpResponse.Content.ReadAsStreamAsync(cancellationToken).ConfigureAwait(false);

            // ストリームからメッセージを順次読み取って返す
            while (!cancellationToken.IsCancellationRequested)
            {
                // エンベロープ形式でメッセージを読み取る
                var (messageBytes, endStream) = await ConnectSerializer.ReadEnvelopedMessageAsync(
                    // レスポンスストリームから読み取る
                    responseStream,
                    // キャンセルトークンを渡す
                    cancellationToken).ConfigureAwait(false);

                // EOS フラグが設定されている場合はストリームを終了する
                if (endStream || messageBytes.Length == 0)
                {
                    // ストリームを終了する
                    yield break;
                }

                // メッセージをデシリアライズして返す
                var message = ConnectSerializer.DeserializeFromJsonBytes<TResponse>(messageBytes);
                // デシリアライズしたメッセージを返す
                yield return message;
            }
        }

        // ClientStreamAsync はクライアント側ストリーミングを非同期で実行する
        public async Task<ConnectUnaryResponse<TResponse>> ClientStreamAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージのストリーム
            IAsyncEnumerable<TRequest> requests,
            // キャンセルトークン
            CancellationToken cancellationToken = default)
        {
            // Pipe でクライアントストリームを実装する
            var pipe = new Pipe();
            // リクエスト送信タスクを開始する
            var requestTask = Task.Run(async () =>
            {
                // リクエストストリームの各メッセージをパイプに書き込む
                await foreach (var request in requests.WithCancellation(cancellationToken).ConfigureAwait(false))
                {
                    // リクエストを JSON にシリアライズする
                    var requestBytes = ConnectSerializer.SerializeToJsonBytes(request);
                    // エンベロープ形式でパイプに書き込む
                    await ConnectSerializer.WriteEnvelopedMessageAsync(
                        // パイプのライターストリームに書き込む
                        pipe.Writer.AsStream(),
                        // シリアライズしたバイト配列を書き込む
                        requestBytes,
                        // EOS ではないので false を指定する
                        endStream: false,
                        // キャンセルトークンを渡す
                        cancellationToken).ConfigureAwait(false);
                }
                // 全リクエストの送信が完了したことをパイプに通知する
                await pipe.Writer.CompleteAsync().ConfigureAwait(false);
            }, cancellationToken);

            // HTTP リクエストを作成する (リクエストボディをパイプから読み取る)
            using var httpRequest = new HttpRequestMessage(
                // POST メソッドを使用する
                HttpMethod.Post,
                // メソッドのフルパスを組み立てる
                $"{_baseUrl}/{methodPath.TrimStart('/')}")
            {
                // リクエストボディをパイプのリーダーストリームから設定する
                Content = new StreamContent(pipe.Reader.AsStream()),
            };
            // Connect-JSON コンテンツタイプを設定する
            httpRequest.Content.Headers.ContentType = new MediaTypeHeaderValue(ConnectContentType.ConnectStreamingJson);
            // HTTP/2 バージョンを要求する
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信する
            using var httpResponse = await _httpClient.SendAsync(
                // HTTP リクエストを送信する
                httpRequest,
                // レスポンスヘッダーのみ受信してボディは後で読み取る
                HttpCompletionOption.ResponseHeadersRead,
                // キャンセルトークンを渡す
                cancellationToken).ConfigureAwait(false);

            // リクエスト送信タスクの完了を待機する
            await requestTask.ConfigureAwait(false);

            // レスポンスボディを読み取る
            var responseBytes = await httpResponse.Content.ReadAsByteArrayAsync(cancellationToken).ConfigureAwait(false);
            // レスポンスをデシリアライズする
            var responseMessage = ConnectSerializer.DeserializeFromJsonBytes<TResponse>(responseBytes);
            // レスポンスヘッダーを取得する
            var responseHeaders = ExtractHeaders(httpResponse.Headers);
            // レスポンストレーラーを取得する
            var responseTrailers = ExtractTrailers(httpResponse.TrailingHeaders);

            // ConnectUnaryResponse を返す
            return new ConnectUnaryResponse<TResponse>(responseMessage, responseHeaders, responseTrailers);
        }

        // BidiStreamAsync は双方向ストリーミングを非同期で実行する
        public async IAsyncEnumerable<TResponse> BidiStreamAsync<TRequest, TResponse>(
            // 呼び出すメソッドの URL パス
            string methodPath,
            // リクエストメッセージのストリーム
            IAsyncEnumerable<TRequest> requests,
            // キャンセルトークン (EnumeratorCancellation 属性を付与する)
            [EnumeratorCancellation] CancellationToken cancellationToken = default)
        {
            // Pipe でリクエストストリームを実装する
            var requestPipe = new Pipe();
            // リクエスト送信タスクを開始する (バックグラウンドで送信する)
            var requestTask = Task.Run(async () =>
            {
                // リクエストストリームの各メッセージをパイプに書き込む
                await foreach (var request in requests.WithCancellation(cancellationToken).ConfigureAwait(false))
                {
                    // リクエストを JSON にシリアライズする
                    var requestBytes = ConnectSerializer.SerializeToJsonBytes(request);
                    // エンベロープ形式でパイプに書き込む
                    await ConnectSerializer.WriteEnvelopedMessageAsync(
                        // パイプのライターストリームに書き込む
                        requestPipe.Writer.AsStream(),
                        // シリアライズしたバイト配列を書き込む
                        requestBytes,
                        // EOS ではないので false を指定する
                        endStream: false,
                        // キャンセルトークンを渡す
                        cancellationToken).ConfigureAwait(false);
                }
                // 全リクエストの送信が完了したことをパイプに通知する
                await requestPipe.Writer.CompleteAsync().ConfigureAwait(false);
            }, cancellationToken);

            // HTTP リクエストを作成する (リクエストボディをパイプから読み取る)
            using var httpRequest = new HttpRequestMessage(
                // POST メソッドを使用する
                HttpMethod.Post,
                // メソッドのフルパスを組み立てる
                $"{_baseUrl}/{methodPath.TrimStart('/')}")
            {
                // リクエストボディをパイプのリーダーストリームから設定する
                Content = new StreamContent(requestPipe.Reader.AsStream()),
            };
            // Connect-JSON コンテンツタイプを設定する
            httpRequest.Content.Headers.ContentType = new MediaTypeHeaderValue(ConnectContentType.ConnectStreamingJson);
            // HTTP/2 バージョンを要求する
            httpRequest.Version = new Version(2, 0);

            // HTTP リクエストを送信する (レスポンスヘッダーのみ受信)
            using var httpResponse = await _httpClient.SendAsync(
                // HTTP リクエストを送信する
                httpRequest,
                // レスポンスヘッダーのみ受信してボディは後で読み取る
                HttpCompletionOption.ResponseHeadersRead,
                // キャンセルトークンを渡す
                cancellationToken).ConfigureAwait(false);

            // レスポンスストリームを取得する
            var responseStream = await httpResponse.Content.ReadAsStreamAsync(cancellationToken).ConfigureAwait(false);

            // ストリームからメッセージを順次読み取って返す
            while (!cancellationToken.IsCancellationRequested)
            {
                // エンベロープ形式でメッセージを読み取る
                var (messageBytes, endStream) = await ConnectSerializer.ReadEnvelopedMessageAsync(
                    // レスポンスストリームから読み取る
                    responseStream,
                    // キャンセルトークンを渡す
                    cancellationToken).ConfigureAwait(false);

                // EOS フラグが設定されている場合はストリームを終了する
                if (endStream || messageBytes.Length == 0)
                {
                    // リクエスト送信タスクの完了を待機する
                    await requestTask.ConfigureAwait(false);
                    // ストリームを終了する
                    yield break;
                }

                // メッセージをデシリアライズして返す
                var message = ConnectSerializer.DeserializeFromJsonBytes<TResponse>(messageBytes);
                // デシリアライズしたメッセージを返す
                yield return message;
            }

            // リクエスト送信タスクの完了を待機する
            await requestTask.ConfigureAwait(false);
        }

        // MapHttpStatusToConnectError は HTTP ステータスコードを ConnectErrorCode にマップする
        private static ConnectErrorCode MapHttpStatusToConnectError(System.Net.HttpStatusCode statusCode)
        {
            // ステータスコードに応じて ConnectErrorCode を返す
            return statusCode switch
            {
                // 400 Bad Request → InvalidArgument
                System.Net.HttpStatusCode.BadRequest => ConnectErrorCode.InvalidArgument,
                // 401 Unauthorized → Unauthenticated
                System.Net.HttpStatusCode.Unauthorized => ConnectErrorCode.Unauthenticated,
                // 403 Forbidden → PermissionDenied
                System.Net.HttpStatusCode.Forbidden => ConnectErrorCode.PermissionDenied,
                // 404 Not Found → NotFound
                System.Net.HttpStatusCode.NotFound => ConnectErrorCode.NotFound,
                // 408 Request Timeout → DeadlineExceeded
                System.Net.HttpStatusCode.RequestTimeout => ConnectErrorCode.DeadlineExceeded,
                // 409 Conflict → Aborted
                System.Net.HttpStatusCode.Conflict => ConnectErrorCode.Aborted,
                // 429 Too Many Requests → ResourceExhausted
                System.Net.HttpStatusCode.TooManyRequests => ConnectErrorCode.ResourceExhausted,
                // 501 Not Implemented → Unimplemented
                System.Net.HttpStatusCode.NotImplemented => ConnectErrorCode.Unimplemented,
                // 503 Service Unavailable → Unavailable
                System.Net.HttpStatusCode.ServiceUnavailable => ConnectErrorCode.Unavailable,
                // それ以外は Unknown
                _ => ConnectErrorCode.Unknown,
            };
        }

        // ExtractHeaders は HTTP レスポンスヘッダーを Dictionary に変換する
        private static IReadOnlyDictionary<string, string> ExtractHeaders(
            // HTTP レスポンスヘッダーコレクション
            System.Net.Http.Headers.HttpResponseHeaders headers)
        {
            // Dictionary を初期化する
            var dict = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            // 各ヘッダーを Dictionary に追加する
            foreach (var header in headers)
            {
                // ヘッダー値を結合して Dictionary に追加する
                dict[header.Key] = string.Join(", ", header.Value);
            }
            // 変換した Dictionary を返す
            return dict;
        }

        // ExtractTrailers は HTTP レスポンストレーラーを Dictionary に変換する
        private static IReadOnlyDictionary<string, string> ExtractTrailers(
            // HTTP レスポンストレーラーコレクション
            System.Net.Http.Headers.HttpResponseHeaders trailers)
        {
            // Dictionary を初期化する
            var dict = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            // 各トレーラーを Dictionary に追加する
            foreach (var trailer in trailers)
            {
                // トレーラー値を結合して Dictionary に追加する
                dict[trailer.Key] = string.Join(", ", trailer.Value);
            }
            // 変換した Dictionary を返す
            return dict;
        }
    }

    // K1s0ConnectClientFactory は K1s0ConnectClient のファクトリクラス
    // DI コンテナから HttpClient を取得して K1s0ConnectClient を作成する
    public static class K1s0ConnectClientFactory
    {
        // Create は HttpClientFactory を使用して K1s0ConnectClient を作成する
        public static K1s0ConnectClient Create(
            // HttpClient ファクトリ
            IHttpClientFactory httpClientFactory,
            // クライアント名: 名前付き HttpClient を取得するための名前
            string clientName,
            // サービスベース URL
            string baseUrl)
        {
            // HttpClientFactory から名前付き HttpClient を取得する
            var httpClient = httpClientFactory.CreateClient(clientName);
            // K1s0ConnectClient を作成して返す
            return new K1s0ConnectClient(httpClient, baseUrl);
        }
    }
}
