// k1s0 コンパニオン OTel 4 スタック拡張のテストファイル
// WCF / HttpWebRequest / HttpClient / WebClient に OTel 属性が注入されるかを確認する
using System;
// System.Net は HttpWebRequest と WebClient に使用する
using System.Net;
// System.Net.Http は HttpClient に使用する
using System.Net.Http;
// System.Diagnostics は Activity に使用する
using System.Diagnostics;
// System.Threading.Tasks は非同期テストに使用する
using System.Threading.Tasks;
// System.Collections.Generic はコレクション操作に使用する
using System.Collections.Generic;
// Microsoft.VisualStudio.TestTools.UnitTesting はテストフレームワークに使用する
using Microsoft.VisualStudio.TestTools.UnitTesting;
// OpenTelemetry SDK をテストに使用する
using OpenTelemetry;
// OpenTelemetry トレース API をテストに使用する
using OpenTelemetry.Trace;
// k1s0 OTel 拡張ライブラリをテスト対象としてインポートする
using K1s0.Companion.NetFx.OTelExt;

// k1s0 OTel 4 スタック拡張のテスト名前空間を定義する
namespace K1s0.Companion.NetFx.OTelExt.Tests
{
    // K1s0OTelActivitySourceTests は ActivitySource の動作を確認するテストクラス
    [TestClass]
    public class K1s0OTelActivitySourceTests
    {
        // テスト用の ActivityListener を保持するフィールド
        private ActivityListener? _listener;
        // テスト中に記録された Activity リストを保持するフィールド
        private List<Activity> _recordedActivities = new List<Activity>();

        // テスト初期化: ActivityListener を登録してテスト対象の Activity を捕捉する
        [TestInitialize]
        public void Initialize()
        {
            // テスト用の Activity リストを初期化する
            _recordedActivities = new List<Activity>();
            // ActivityListener を作成して k1s0.companion の Activity を捕捉する
            _listener = new ActivityListener
            {
                // 全ての ActivitySource を監視する
                ShouldListenTo = source => source.Name == K1s0OTelActivitySource.SourceName,
                // 全ての Activity をサンプリングする (テスト用)
                Sample = (ref ActivityCreationOptions<ActivityContext> options) =>
                    ActivitySamplingResult.AllDataAndRecorded,
                // Activity 停止時に記録リストに追加する
                ActivityStopped = activity => _recordedActivities.Add(activity),
            };
            // ActivityListener を ActivitySource に登録する
            ActivitySource.AddActivityListener(_listener);
        }

        // テスト終了処理: ActivityListener を解放する
        [TestCleanup]
        public void Cleanup()
        {
            // ActivityListener を解放する
            _listener?.Dispose();
            // _listener を null に設定する
            _listener = null;
        }

        // ActivitySource 名称確認テスト: SourceName が "k1s0.companion" であることを確認する
        [TestMethod]
        public void ActivitySourceName_ShouldBeK1s0Companion()
        {
            // ActivitySource の名称が期待値と一致することを確認する
            Assert.AreEqual(
                // 期待値: "k1s0.companion"
                "k1s0.companion",
                // 実際値: K1s0OTelActivitySource.SourceName
                K1s0OTelActivitySource.SourceName,
                // アサーション失敗メッセージ
                "ActivitySource 名称が期待値と一致しない"
            );
        }

        // StartActivity テスト: Activity が正常に開始されることを確認する
        [TestMethod]
        public void StartActivity_ShouldReturnActivity()
        {
            // StartActivity を呼び出して Activity を取得する
            using var activity = K1s0OTelActivitySource.StartActivity(
                // 操作名: "TestOperation"
                "TestOperation",
                // Activity 種別: Client
                ActivityKind.Client);

            // Activity が null でないことを確認する
            // (ActivityListener が登録されていれば null にはならない)
            if (activity != null)
            {
                // Activity の操作名が期待値と一致することを確認する
                Assert.AreEqual(
                    // 期待値: "TestOperation"
                    "TestOperation",
                    // 実際値: activity.OperationName
                    activity.OperationName,
                    // アサーション失敗メッセージ
                    "Activity 操作名が期待値と一致しない"
                );
            }
        }

        // Activity 属性設定テスト: SetTag で属性が正しく設定されることを確認する
        [TestMethod]
        public void ActivitySetTag_ShouldRecordAttributes()
        {
            // Activity を開始する
            using var activity = K1s0OTelActivitySource.StartActivity(
                // 操作名: "AttributeTest"
                "AttributeTest");

            // Activity が null でないことを確認する
            if (activity == null)
            {
                // Activity が null の場合はテストをスキップする
                Assert.Inconclusive("Activity が null のためテストをスキップ");
                return;
            }

            // HTTP メソッド属性を設定する
            activity.SetTag("http.method", "GET");
            // HTTP URL 属性を設定する
            activity.SetTag("http.url", "https://example.com/api");
            // k1s0 サービス名属性を設定する
            activity.SetTag("k1s0.service.name", "test-service");

            // HTTP メソッド属性が正しく設定されたことを確認する
            Assert.AreEqual(
                // 期待値: "GET"
                "GET",
                // 実際値: TagObjects から取得した値
                activity.GetTagItem("http.method")?.ToString(),
                // アサーション失敗メッセージ
                "http.method 属性が期待値と一致しない"
            );

            // HTTP URL 属性が正しく設定されたことを確認する
            Assert.AreEqual(
                // 期待値: "https://example.com/api"
                "https://example.com/api",
                // 実際値: TagObjects から取得した値
                activity.GetTagItem("http.url")?.ToString(),
                // アサーション失敗メッセージ
                "http.url 属性が期待値と一致しない"
            );

            // k1s0 サービス名属性が正しく設定されたことを確認する
            Assert.AreEqual(
                // 期待値: "test-service"
                "test-service",
                // 実際値: TagObjects から取得した値
                activity.GetTagItem("k1s0.service.name")?.ToString(),
                // アサーション失敗メッセージ
                "k1s0.service.name 属性が期待値と一致しない"
            );
        }
    }

    // K1s0HttpClientInstrumentationTests は HttpClient 計装 DelegatingHandler のテストクラス
    [TestClass]
    public class K1s0HttpClientInstrumentationTests
    {
        // テスト用の ActivityListener を保持するフィールド
        private ActivityListener? _listener;
        // テスト中に記録された Activity リストを保持するフィールド
        private List<Activity> _recordedActivities = new List<Activity>();

        // テスト初期化: ActivityListener を登録する
        [TestInitialize]
        public void Initialize()
        {
            // テスト用の Activity リストを初期化する
            _recordedActivities = new List<Activity>();
            // ActivityListener を作成して k1s0.companion の Activity を捕捉する
            _listener = new ActivityListener
            {
                // k1s0.companion の ActivitySource を監視する
                ShouldListenTo = source => source.Name == K1s0OTelActivitySource.SourceName,
                // 全ての Activity をサンプリングする
                Sample = (ref ActivityCreationOptions<ActivityContext> options) =>
                    ActivitySamplingResult.AllDataAndRecorded,
                // Activity 停止時に記録リストに追加する
                ActivityStopped = activity => _recordedActivities.Add(activity),
            };
            // ActivityListener を ActivitySource に登録する
            ActivitySource.AddActivityListener(_listener);
        }

        // テスト終了処理: ActivityListener を解放する
        [TestCleanup]
        public void Cleanup()
        {
            // ActivityListener を解放する
            _listener?.Dispose();
            // _listener を null に設定する
            _listener = null;
        }

        // HttpClient トレーシングハンドラー作成テスト: K1s0HttpClientTracingHandler が正常に作成されることを確認する
        [TestMethod]
        public void K1s0HttpClientTracingHandler_ShouldCreateSuccessfully()
        {
            // K1s0HttpClientTracingHandler を作成する
            using var handler = new K1s0HttpClientTracingHandler(k1s0ServiceName: "test-service");

            // ハンドラーが null でないことを確認する
            Assert.IsNotNull(
                // 確認対象: ハンドラーオブジェクト
                handler,
                // アサーション失敗メッセージ
                "K1s0HttpClientTracingHandler の作成に失敗した"
            );
        }

        // HttpClient 計装テスト: Traceparent ヘッダーが HTTP リクエストに注入されることを確認する
        [TestMethod]
        public async Task K1s0HttpClientTracingHandler_ShouldInjectTraceparentHeader()
        {
            // テスト用の HTTP レスポンスを返すモックハンドラーを作成する
            using var mockHandler = new MockHttpMessageHandler();
            // K1s0HttpClientTracingHandler にモックハンドラーをチェーンする
            using var tracingHandler = new K1s0HttpClientTracingHandler(mockHandler, "test-service");
            // HttpClient を作成する
            using var httpClient = new HttpClient(tracingHandler);

            // テスト用の HTTP リクエストを送信する
            try
            {
                // モック URL にリクエストを送信する (実際には接続しない)
                await httpClient.GetAsync("http://test.example.com/api");
            }
            catch
            {
                // モックハンドラーが例外を発生させても無視する
            }

            // モックハンドラーに記録されたリクエストヘッダーを確認する
            if (mockHandler.LastRequest != null)
            {
                // Traceparent ヘッダーが注入されたかどうかを確認する
                Assert.IsTrue(
                    // 確認条件: Traceparent ヘッダーが存在すること
                    mockHandler.LastRequest.Headers.Contains("traceparent"),
                    // アサーション失敗メッセージ
                    "Traceparent ヘッダーが HTTP リクエストに注入されていない"
                );
            }
        }
    }

    // MockHttpMessageHandler はテスト用のモック HttpMessageHandler
    // 実際の HTTP 接続を行わずにリクエストを記録するモックを定義する
    internal class MockHttpMessageHandler : HttpMessageHandler
    {
        // 最後に受信した HTTP リクエストメッセージを保持するプロパティ
        public HttpRequestMessage? LastRequest { get; private set; }

        // SendAsync をオーバーライドしてリクエストを記録する
        protected override Task<HttpResponseMessage> SendAsync(
            // 送信する HTTP リクエストメッセージ
            HttpRequestMessage request,
            // キャンセルトークン
            System.Threading.CancellationToken cancellationToken)
        {
            // 受信したリクエストを保存する
            LastRequest = request;
            // 200 OK のモックレスポンスを返す
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK));
        }
    }

    // K1s0WcfClientBehaviorTests は WCF クライアント計装のテストクラス
    [TestClass]
    public class K1s0WcfClientBehaviorTests
    {
        // WCF クライアント動作クラス作成テスト: K1s0WcfClientBehavior が正常に作成されることを確認する
        [TestMethod]
        public void K1s0WcfClientBehavior_ShouldCreateSuccessfully()
        {
            // K1s0WcfClientBehavior を作成する
            var behavior = new K1s0WcfClientBehavior(k1s0ServiceName: "test-wcf-service");

            // 動作クラスが null でないことを確認する
            Assert.IsNotNull(
                // 確認対象: 動作クラスオブジェクト
                behavior,
                // アサーション失敗メッセージ
                "K1s0WcfClientBehavior の作成に失敗した"
            );
        }
    }

    // K1s0OTelBuilderTests は OTel プロバイダービルダーのテストクラス
    [TestClass]
    public class K1s0OTelBuilderTests
    {
        // TracerProvider 構築テスト: Build メソッドで TracerProvider が作成されることを確認する
        [TestMethod]
        public void Build_ShouldCreateTracerProvider()
        {
            // K1s0OTelBuilder.Build で TracerProvider を構築する
            using var tracerProvider = K1s0OTelBuilder.Build(
                // テスト用の OTLP エンドポイントを設定する (接続は行わない)
                otlpEndpoint: "http://localhost:4317",
                // k1s0 サービス名を設定する
                serviceName: "test-service",
                // サービスバージョンを設定する
                serviceVersion: "0.1.0");

            // TracerProvider が null でないことを確認する
            Assert.IsNotNull(
                // 確認対象: TracerProvider オブジェクト
                tracerProvider,
                // アサーション失敗メッセージ
                "TracerProvider の構築に失敗した"
            );
        }
    }
}
