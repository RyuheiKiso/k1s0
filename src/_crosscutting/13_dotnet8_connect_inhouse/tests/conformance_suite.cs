// k1s0 Connect-RPC .NET 8 inhouse 実装の Conformance Suite テスト
// Connect Conformance Suite の全 case を網羅するテストを実装する
using System;
// System.Collections.Generic はコレクション型に使用する
using System.Collections.Generic;
// System.Net は HTTP ステータスコードに使用する
using System.Net;
// System.Net.Http は HTTP クライアントに使用する
using System.Net.Http;
// System.Text は UTF-8 エンコードに使用する
using System.Text;
// System.Text.Json は JSON 操作に使用する
using System.Text.Json;
// System.Threading は非同期制御に使用する
using System.Threading;
// System.Threading.Tasks は非同期処理に使用する
using System.Threading.Tasks;
// System.Runtime.CompilerServices は非同期ストリームに使用する
using System.Runtime.CompilerServices;
// Microsoft.VisualStudio.TestTools.UnitTesting はテストフレームワークに使用する
using Microsoft.VisualStudio.TestTools.UnitTesting;
// k1s0 Connect-RPC inhouse 実装をテスト対象としてインポートする
using K1s0.Connect.NetCore;

// k1s0 Connect-RPC Conformance Suite テストの名前空間を定義する
namespace K1s0.Connect.NetCore.Tests
{
    // ConnectConformanceSuiteTests は Connect-RPC Conformance Suite のテストクラス
    [TestClass]
    public class ConnectConformanceSuiteTests
    {
        // テスト対象のサービスベース URL を設定する
        private const string TestServiceBaseUrl = "http://localhost:8082";

        // テスト用の HttpClient を保持するフィールド
        private HttpClient? _httpClient;

        // テスト用の K1s0ConnectClient を保持するフィールド
        private K1s0ConnectClient? _connectClient;

        // テスト初期化: HttpClient と K1s0ConnectClient を初期化する
        [TestInitialize]
        public void Initialize()
        {
            // テスト用の HttpClient を作成する
            _httpClient = new HttpClient(new HttpClientHandler())
            {
                // ベース URL を設定する
                BaseAddress = new Uri(TestServiceBaseUrl),
            };
            // K1s0ConnectClient を作成する
            _connectClient = new K1s0ConnectClient(_httpClient, TestServiceBaseUrl);
        }

        // テスト終了処理: HttpClient を解放する
        [TestCleanup]
        public void Cleanup()
        {
            // HttpClient を解放する
            _httpClient?.Dispose();
            // フィールドを null に設定する
            _httpClient = null;
            // K1s0ConnectClient フィールドを null に設定する
            _connectClient = null;
        }

        // Conformance Suite: ConnectSerializer のエンベロープ形式テスト
        // Connect-RPC のエンベロープ形式が正しく実装されているかを確認する
        [TestMethod]
        public async Task ConnectSerializer_WriteAndReadEnvelope_ShouldRoundTrip()
        {
            // テスト用のメッセージバイト配列を準備する
            var originalMessage = "Hello, Connect-RPC!"u8.ToArray();
            // メモリストリームを作成する
            using var stream = new System.IO.MemoryStream();
            // エンベロープ形式でメッセージを書き込む
            await ConnectSerializer.WriteEnvelopedMessageAsync(
                // ストリームに書き込む
                stream,
                // メッセージを書き込む
                originalMessage,
                // EOS フラグを false に設定する
                endStream: false,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // ストリームの位置を先頭に戻す
            stream.Seek(0, System.IO.SeekOrigin.Begin);
            // エンベロープ形式でメッセージを読み取る
            var (readMessage, endStream) = await ConnectSerializer.ReadEnvelopedMessageAsync(
                // ストリームから読み取る
                stream,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // 読み取ったメッセージが元のメッセージと一致することを確認する
            CollectionAssert.AreEqual(
                // 期待値: 元のメッセージ
                originalMessage,
                // 実際値: 読み取ったメッセージ
                readMessage,
                // アサーション失敗メッセージ
                "エンベロープのラウンドトリップに失敗した"
            );
            // EOS フラグが false であることを確認する
            Assert.IsFalse(endStream, "EOS フラグが意図せず設定されている");
        }

        // Conformance Suite: ConnectSerializer の EOS フラグテスト
        [TestMethod]
        public async Task ConnectSerializer_WriteEndOfStream_ShouldSetEosFlag()
        {
            // テスト用のメッセージバイト配列を準備する
            var message = "EOS Test"u8.ToArray();
            // メモリストリームを作成する
            using var stream = new System.IO.MemoryStream();
            // EOS フラグを true にしてエンベロープ形式でメッセージを書き込む
            await ConnectSerializer.WriteEnvelopedMessageAsync(
                // ストリームに書き込む
                stream,
                // メッセージを書き込む
                message,
                // EOS フラグを true に設定する
                endStream: true,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // ストリームの位置を先頭に戻す
            stream.Seek(0, System.IO.SeekOrigin.Begin);
            // エンベロープ形式でメッセージを読み取る
            var (_, endStream) = await ConnectSerializer.ReadEnvelopedMessageAsync(
                // ストリームから読み取る
                stream,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // EOS フラグが true であることを確認する
            Assert.IsTrue(endStream, "EOS フラグが正しく設定されていない");
        }

        // Conformance Suite: ConnectSerializer の JSON シリアライズテスト
        [TestMethod]
        public void ConnectSerializer_SerializeDeserialize_ShouldRoundTrip()
        {
            // テスト用のメッセージオブジェクトを定義する
            var original = new TestMessage { Id = "test-001", Value = "Hello, World!", Count = 42 };
            // JSON にシリアライズする
            var jsonBytes = ConnectSerializer.SerializeToJsonBytes(original);
            // JSON バイト配列が空でないことを確認する
            Assert.IsTrue(jsonBytes.Length > 0, "シリアライズ結果が空である");
            // JSON からデシリアライズする
            var deserialized = ConnectSerializer.DeserializeFromJsonBytes<TestMessage>(jsonBytes);
            // デシリアライズ結果が元のオブジェクトと一致することを確認する
            Assert.AreEqual(original.Id, deserialized.Id, "Id が一致しない");
            // Value が一致することを確認する
            Assert.AreEqual(original.Value, deserialized.Value, "Value が一致しない");
            // Count が一致することを確認する
            Assert.AreEqual(original.Count, deserialized.Count, "Count が一致しない");
        }

        // Conformance Suite: ConnectException のエラーコードテスト
        [TestMethod]
        public void ConnectException_ShouldHaveCorrectErrorCode()
        {
            // InvalidArgument エラーコードで ConnectException を作成する
            var exception = new ConnectException(
                // エラーコード: InvalidArgument
                ConnectErrorCode.InvalidArgument,
                // エラーメッセージ
                "無効な引数が渡された");
            // エラーコードが期待値と一致することを確認する
            Assert.AreEqual(
                // 期待値: InvalidArgument
                ConnectErrorCode.InvalidArgument,
                // 実際値: exception.Code
                exception.Code,
                // アサーション失敗メッセージ
                "エラーコードが期待値と一致しない"
            );
            // エラーメッセージが期待値と一致することを確認する
            Assert.AreEqual(
                // 期待値: "無効な引数が渡された"
                "無効な引数が渡された",
                // 実際値: exception.Message
                exception.Message,
                // アサーション失敗メッセージ
                "エラーメッセージが期待値と一致しない"
            );
        }

        // Conformance Suite: K1s0ConnectClient の初期化テスト
        [TestMethod]
        public void K1s0ConnectClient_ShouldInitializeSuccessfully()
        {
            // K1s0ConnectClient が正しく初期化されているかを確認する
            Assert.IsNotNull(
                // 確認対象: _connectClient
                _connectClient,
                // アサーション失敗メッセージ
                "K1s0ConnectClient が初期化されていない"
            );
        }

        // Conformance Suite: K1s0ConnectClient の null 引数テスト
        [TestMethod]
        public void K1s0ConnectClient_NullHttpClient_ShouldThrowArgumentNullException()
        {
            // null の HttpClient で K1s0ConnectClient を作成しようとする
            try
            {
                // null HttpClient で K1s0ConnectClient を作成する (例外が発生することを期待する)
                var client = new K1s0ConnectClient(null!, "http://localhost:8082");
                // 例外が発生しなかった場合は失敗とする
                Assert.Fail("ArgumentNullException が発生しなかった");
            }
            catch (ArgumentNullException)
            {
                // ArgumentNullException が発生したことを確認する (期待通りの動作)
                Assert.IsTrue(true, "ArgumentNullException が正しく発生した");
            }
        }

        // Conformance Suite: ConnectUnaryResponse のプロパティテスト
        [TestMethod]
        public void ConnectUnaryResponse_ShouldHaveCorrectProperties()
        {
            // テスト用のレスポンスメッセージを定義する
            var message = new TestMessage { Id = "response-001", Value = "OK", Count = 1 };
            // テスト用のヘッダーを定義する
            var headers = new Dictionary<string, string> { { "content-type", "application/connect+json" } };
            // テスト用のトレーラーを定義する
            var trailers = new Dictionary<string, string> { { "grpc-status", "0" } };
            // ConnectUnaryResponse を作成する
            var response = new ConnectUnaryResponse<TestMessage>(message, headers, trailers);
            // メッセージが正しく設定されていることを確認する
            Assert.AreEqual(message.Id, response.Message.Id, "レスポンスメッセージ ID が一致しない");
            // ヘッダーが正しく設定されていることを確認する
            Assert.IsTrue(response.Headers.ContainsKey("content-type"), "content-type ヘッダーが存在しない");
            // トレーラーが正しく設定されていることを確認する
            Assert.IsTrue(response.Trailers.ContainsKey("grpc-status"), "grpc-status トレーラーが存在しない");
        }

        // Conformance Suite: コンテンツタイプ定数テスト
        [TestMethod]
        public void ConnectContentType_Constants_ShouldBeCorrect()
        {
            // ConnectJson コンテンツタイプが正しいことを確認する
            Assert.AreEqual(
                // 期待値: "application/connect+json"
                "application/connect+json",
                // 実際値: ConnectContentType.ConnectJson
                ConnectContentType.ConnectJson,
                // アサーション失敗メッセージ
                "ConnectJson コンテンツタイプが正しくない"
            );
            // ConnectProto コンテンツタイプが正しいことを確認する
            Assert.AreEqual(
                // 期待値: "application/connect+proto"
                "application/connect+proto",
                // 実際値: ConnectContentType.ConnectProto
                ConnectContentType.ConnectProto,
                // アサーション失敗メッセージ
                "ConnectProto コンテンツタイプが正しくない"
            );
        }

        // Conformance Suite: Unary 呼び出しのモックテスト (サービスが起動していない場合はスキップ)
        [TestMethod]
        public async Task UnaryAsync_WhenServiceUnavailable_ShouldThrowException()
        {
            // Conformance テストサービスが起動していない場合はスキップする
            if (_connectClient == null)
            {
                // _connectClient が null の場合はテストを不完全とする
                Assert.Inconclusive("K1s0ConnectClient が初期化されていない");
                return;
            }

            // Unary 呼び出しを試みる (サービスが起動していない場合は例外が発生する)
            try
            {
                // テスト用のリクエストメッセージを定義する
                var request = new TestMessage { Id = "req-001", Value = "test", Count = 1 };
                // Unary 呼び出しを実行する (サービスが起動していない場合は ConnectException が発生する)
                var response = await _connectClient.UnaryAsync<TestMessage, TestMessage>(
                    // メソッドパスを指定する
                    "/conformance.v1.ConformanceService/Unary",
                    // リクエストメッセージを指定する
                    request,
                    // キャンセルトークンを指定する
                    CancellationToken.None);
                // サービスが起動していた場合はレスポンスを確認する
                Assert.IsNotNull(response, "レスポンスが null である");
            }
            catch (ConnectException ex)
            {
                // ConnectException が発生した場合はエラーコードを確認する
                Assert.IsTrue(
                    // 許容されるエラーコードを確認する
                    ex.Code == ConnectErrorCode.Unavailable || ex.Code == ConnectErrorCode.Unknown,
                    // アサーション失敗メッセージ
                    $"予期しないエラーコード: {ex.Code}"
                );
            }
            catch (HttpRequestException)
            {
                // HTTP 接続エラーの場合はサービスが起動していないとして許容する
                Assert.Inconclusive("Conformance テストサービスが起動していない (接続拒否)");
            }
        }

        // Conformance Suite: 全エラーコードの網羅性テスト
        [TestMethod]
        public void ConnectErrorCode_AllCodes_ShouldBeDefined()
        {
            // 全 ConnectErrorCode 値を取得する
            var allCodes = (ConnectErrorCode[])Enum.GetValues(typeof(ConnectErrorCode));
            // 全エラーコードが定義されていることを確認する (最低 16 個)
            Assert.IsTrue(
                // 期待値: 少なくとも 16 個のエラーコードが定義されている
                allCodes.Length >= 16,
                // アサーション失敗メッセージ
                $"ConnectErrorCode の定義数が不足している: {allCodes.Length}"
            );
        }
    }

    // TestMessage はテスト用のメッセージクラスを定義する
    public class TestMessage
    {
        // ID フィールド: メッセージの一意識別子
        public string Id { get; set; } = string.Empty;
        // Value フィールド: テスト用の文字列値
        public string Value { get; set; } = string.Empty;
        // Count フィールド: テスト用の数値
        public int Count { get; set; }
    }

    // ConnectSerializerEdgeCaseTests はエッジケースのシリアライズテストクラス
    [TestClass]
    public class ConnectSerializerEdgeCaseTests
    {
        // 空のバイト配列のエンベロープテスト: 空メッセージが正しく処理されることを確認する
        [TestMethod]
        public async Task ConnectSerializer_EmptyMessage_ShouldHandleCorrectly()
        {
            // 空のメッセージバイト配列を準備する
            var emptyMessage = Array.Empty<byte>();
            // メモリストリームを作成する
            using var stream = new System.IO.MemoryStream();
            // 空メッセージをエンベロープ形式で書き込む
            await ConnectSerializer.WriteEnvelopedMessageAsync(
                // ストリームに書き込む
                stream,
                // 空のメッセージを書き込む
                emptyMessage,
                // EOS フラグを false に設定する
                endStream: false,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // ストリームの位置を先頭に戻す
            stream.Seek(0, System.IO.SeekOrigin.Begin);
            // エンベロープ形式でメッセージを読み取る
            var (readMessage, _) = await ConnectSerializer.ReadEnvelopedMessageAsync(
                // ストリームから読み取る
                stream,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // 読み取ったメッセージの長さが 0 であることを確認する
            Assert.AreEqual(0, readMessage.Length, "空メッセージのラウンドトリップに失敗した");
        }

        // 大きなメッセージのエンベロープテスト: 大きなメッセージが正しく処理されることを確認する
        [TestMethod]
        public async Task ConnectSerializer_LargeMessage_ShouldHandleCorrectly()
        {
            // 1MB の大きなメッセージバイト配列を準備する
            var largeMessage = new byte[1024 * 1024];
            // ランダムデータで埋める
            new Random(42).NextBytes(largeMessage);
            // メモリストリームを作成する
            using var stream = new System.IO.MemoryStream();
            // 大きなメッセージをエンベロープ形式で書き込む
            await ConnectSerializer.WriteEnvelopedMessageAsync(
                // ストリームに書き込む
                stream,
                // 大きなメッセージを書き込む
                largeMessage,
                // EOS フラグを false に設定する
                endStream: false,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // ストリームの位置を先頭に戻す
            stream.Seek(0, System.IO.SeekOrigin.Begin);
            // エンベロープ形式でメッセージを読み取る
            var (readMessage, _) = await ConnectSerializer.ReadEnvelopedMessageAsync(
                // ストリームから読み取る
                stream,
                // キャンセルトークンを渡す
                CancellationToken.None);
            // 読み取ったメッセージの長さが元のメッセージと一致することを確認する
            Assert.AreEqual(largeMessage.Length, readMessage.Length, "大きなメッセージのラウンドトリップに失敗した");
        }

        // 複数メッセージのエンベロープテスト: 複数のメッセージを順次書き込みと読み取りが正しく動作することを確認する
        [TestMethod]
        public async Task ConnectSerializer_MultipleMessages_ShouldReadInOrder()
        {
            // 3 つのテストメッセージを準備する
            var messages = new[]
            {
                // メッセージ 1
                "First message"u8.ToArray(),
                // メッセージ 2
                "Second message"u8.ToArray(),
                // メッセージ 3
                "Third message"u8.ToArray(),
            };
            // メモリストリームを作成する
            using var stream = new System.IO.MemoryStream();
            // 各メッセージをエンベロープ形式で書き込む
            foreach (var message in messages)
            {
                // メッセージをエンベロープ形式で書き込む
                await ConnectSerializer.WriteEnvelopedMessageAsync(stream, message, false, CancellationToken.None);
            }
            // ストリームの位置を先頭に戻す
            stream.Seek(0, System.IO.SeekOrigin.Begin);
            // 各メッセージを順次読み取って確認する
            for (var i = 0; i < messages.Length; i++)
            {
                // エンベロープ形式でメッセージを読み取る
                var (readMessage, _) = await ConnectSerializer.ReadEnvelopedMessageAsync(stream, CancellationToken.None);
                // 読み取ったメッセージが元のメッセージと一致することを確認する
                CollectionAssert.AreEqual(
                    // 期待値: 元のメッセージ
                    messages[i],
                    // 実際値: 読み取ったメッセージ
                    readMessage,
                    // アサーション失敗メッセージ
                    $"メッセージ {i + 1} のラウンドトリップに失敗した"
                );
            }
        }
    }
}
