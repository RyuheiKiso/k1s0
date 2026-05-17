// k1s0 コンパニオン .NET Framework OTel 4 スタック拡張
// WCF / HttpWebRequest / HttpClient / WebClient の 4 スタックに OTel を計装する
using System;
// System.Net は HttpWebRequest と WebClient に使用する
using System.Net;
// System.Net.Http は HttpClient に使用する
using System.Net.Http;
// System.ServiceModel は WCF クライアントに使用する
using System.ServiceModel;
// System.ServiceModel.Channels は WCF メッセージに使用する
using System.ServiceModel.Channels;
// Activity は OTel トレースの中心概念に使用する
using System.Diagnostics;
// Collections.Generic はコレクション操作に使用する
using System.Collections.Generic;
// Thread は同期的な HTTP 呼び出しに使用する
using System.Threading;
// Task は非同期処理に使用する
using System.Threading.Tasks;
// OpenTelemetry API: トレース計装に使用する
using OpenTelemetry;
// OpenTelemetry トレース API: Tracer 取得に使用する
using OpenTelemetry.Trace;

// k1s0 コンパニオン OTel 拡張の名前空間を定義する
namespace K1s0.Companion.NetFx.OTelExt
{
    // K1s0OTelActivitySource は OTel ActivitySource の中央管理クラス
    // 4 スタック全体で共通の ActivitySource を使用してトレースを記録する
    public static class K1s0OTelActivitySource
    {
        // ActivitySource 名称: k1s0.companion を使用する
        public const string SourceName = "k1s0.companion";

        // ActivitySource バージョン: ライブラリバージョンと一致させる
        public const string SourceVersion = "0.1.0";

        // ActivitySource インスタンス: 全トレースで共有する
        private static readonly ActivitySource Source = new ActivitySource(SourceName, SourceVersion);

        // 新しい Activity を開始するメソッド: 操作名と種別を受け取る
        public static Activity? StartActivity(
            // トレースの操作名: HTTP GET / WCF Call 等の操作名を指定する
            string operationName,
            // Activity の種別: Client / Server / Internal を指定する
            ActivityKind kind = ActivityKind.Client)
        {
            // ActivitySource から新しい Activity を開始して返す
            return Source.StartActivity(operationName, kind);
        }

        // ActivitySource を取得するプロパティ: 外部計装ライブラリに渡す用途
        public static ActivitySource ActivitySource => Source;
    }

    // K1s0HttpWebRequestInstrumentation は HttpWebRequest の OTel 計装クラス
    // .NET Framework の古い HTTP スタック (HttpWebRequest) を OTel でトレースする
    public class K1s0HttpWebRequestInstrumentation
    {
        // HTTP メソッド属性名: OTel セマンティクス規約に従う
        private const string HttpMethodAttribute = "http.method";

        // HTTP URL 属性名: OTel セマンティクス規約に従う
        private const string HttpUrlAttribute = "http.url";

        // HTTP ステータスコード属性名: OTel セマンティクス規約に従う
        private const string HttpStatusCodeAttribute = "http.status_code";

        // k1s0 サービス名属性: カスタム属性としてサービス名を記録する
        private const string K1s0ServiceAttribute = "k1s0.service.name";

        // ExecuteWithTracing は HttpWebRequest をトレース付きで実行するメソッド
        public static HttpWebResponse ExecuteWithTracing(
            // トレース対象の HttpWebRequest オブジェクト
            HttpWebRequest request,
            // k1s0 サービス名 (カスタム属性として記録する)
            string? k1s0ServiceName = null)
        {
            // OTel Activity を開始する (HTTP クライアント種別)
            using var activity = K1s0OTelActivitySource.StartActivity(
                // 操作名: "HttpWebRequest {METHOD}" の形式で設定する
                $"HttpWebRequest {request.Method}",
                // Activity 種別: Client (外部サービス呼び出し)
                ActivityKind.Client);

            // Activity が開始された場合は属性を設定する
            if (activity != null)
            {
                // HTTP メソッド属性を設定する
                activity.SetTag(HttpMethodAttribute, request.Method);
                // HTTP URL 属性を設定する (クエリ文字列を含む完全な URL)
                activity.SetTag(HttpUrlAttribute, request.RequestUri?.ToString());
                // k1s0 サービス名属性を設定する (指定がある場合のみ)
                if (k1s0ServiceName != null)
                {
                    // k1s0 サービス名を属性として記録する
                    activity.SetTag(K1s0ServiceAttribute, k1s0ServiceName);
                }

                // W3C TraceContext を Traceparent ヘッダーとして HTTP リクエストに注入する
                var traceParent = $"00-{activity.TraceId}-{activity.SpanId}-01";
                // Traceparent ヘッダーを設定する
                request.Headers["traceparent"] = traceParent;
            }

            // HttpWebRequest を実行してレスポンスを取得する
            try
            {
                // レスポンスを取得する
                var response = (HttpWebResponse)request.GetResponse();

                // Activity が開始されている場合はステータスコードを記録する
                if (activity != null)
                {
                    // HTTP ステータスコードを整数で設定する
                    activity.SetTag(HttpStatusCodeAttribute, (int)response.StatusCode);
                    // OTel ステータスを Success に設定する
                    activity.SetStatus(ActivityStatusCode.Ok);
                }

                // レスポンスを返す
                return response;
            }
            catch (WebException ex)
            {
                // 例外が発生した場合はエラーを Activity に記録する
                if (activity != null)
                {
                    // OTel ステータスをエラーに設定する
                    activity.SetStatus(ActivityStatusCode.Error, ex.Message);
                    // エラーの例外タイプを記録する
                    activity.SetTag("exception.type", ex.GetType().FullName);
                    // エラーメッセージを記録する
                    activity.SetTag("exception.message", ex.Message);
                }
                // 例外を再スローする
                throw;
            }
        }
    }

    // K1s0HttpClientInstrumentation は HttpClient の OTel 計装 DelegatingHandler
    // .NET Framework の HttpClient を OTel でトレースする DelegatingHandler を定義する
    public class K1s0HttpClientTracingHandler : DelegatingHandler
    {
        // k1s0 サービス名: カスタム属性として記録するサービス名
        private readonly string? _k1s0ServiceName;

        // コンストラクタ: k1s0 サービス名を受け取る
        public K1s0HttpClientTracingHandler(string? k1s0ServiceName = null)
            // 内部ハンドラを HttpClientHandler で初期化する
            : base(new HttpClientHandler())
        {
            // k1s0 サービス名を保存する
            _k1s0ServiceName = k1s0ServiceName;
        }

        // コンストラクタ: 内部ハンドラを受け取る (チェーン可能なハンドラ構成)
        public K1s0HttpClientTracingHandler(
            // チェーン先の DelegatingHandler を受け取る
            HttpMessageHandler innerHandler,
            // k1s0 サービス名を受け取る
            string? k1s0ServiceName = null)
            // 内部ハンドラを設定する
            : base(innerHandler)
        {
            // k1s0 サービス名を保存する
            _k1s0ServiceName = k1s0ServiceName;
        }

        // SendAsync をオーバーライドして OTel 計装を追加する
        protected override async Task<HttpResponseMessage> SendAsync(
            // 送信する HTTP リクエストメッセージ
            HttpRequestMessage request,
            // キャンセルトークン
            CancellationToken cancellationToken)
        {
            // OTel Activity を開始する (HTTP クライアント種別)
            using var activity = K1s0OTelActivitySource.StartActivity(
                // 操作名: "HttpClient {METHOD}" の形式で設定する
                $"HttpClient {request.Method.Method}",
                // Activity 種別: Client (外部サービス呼び出し)
                ActivityKind.Client);

            // Activity が開始された場合は属性とヘッダーを設定する
            if (activity != null)
            {
                // HTTP メソッド属性を設定する
                activity.SetTag("http.method", request.Method.Method);
                // HTTP URL 属性を設定する
                activity.SetTag("http.url", request.RequestUri?.ToString());
                // k1s0 サービス名属性を設定する (指定がある場合のみ)
                if (_k1s0ServiceName != null)
                {
                    // k1s0 サービス名を属性として記録する
                    activity.SetTag("k1s0.service.name", _k1s0ServiceName);
                }

                // W3C TraceContext を Traceparent ヘッダーとして HTTP リクエストに注入する
                var traceParent = $"00-{activity.TraceId}-{activity.SpanId}-01";
                // Traceparent ヘッダーを設定する (重複する場合は上書きする)
                request.Headers.Remove("traceparent");
                // 新しい Traceparent ヘッダーを追加する
                request.Headers.Add("traceparent", traceParent);
            }

            // リクエストを送信してレスポンスを取得する
            try
            {
                // 内部ハンドラにリクエストを委譲する
                var response = await base.SendAsync(request, cancellationToken).ConfigureAwait(false);

                // Activity が開始されている場合はステータスコードを記録する
                if (activity != null)
                {
                    // HTTP ステータスコードを整数で設定する
                    activity.SetTag("http.status_code", (int)response.StatusCode);
                    // ステータスコードに応じて OTel ステータスを設定する
                    if (response.IsSuccessStatusCode)
                    {
                        // 成功の場合は OTel ステータスを Success に設定する
                        activity.SetStatus(ActivityStatusCode.Ok);
                    }
                    else
                    {
                        // 失敗の場合は OTel ステータスをエラーに設定する
                        activity.SetStatus(ActivityStatusCode.Error, $"HTTP {(int)response.StatusCode}");
                    }
                }

                // レスポンスを返す
                return response;
            }
            catch (Exception ex)
            {
                // 例外が発生した場合はエラーを Activity に記録する
                if (activity != null)
                {
                    // OTel ステータスをエラーに設定する
                    activity.SetStatus(ActivityStatusCode.Error, ex.Message);
                    // エラーの例外タイプを記録する
                    activity.SetTag("exception.type", ex.GetType().FullName);
                    // エラーメッセージを記録する
                    activity.SetTag("exception.message", ex.Message);
                }
                // 例外を再スローする
                throw;
            }
        }
    }

    // K1s0WebClientInstrumentation は WebClient の OTel 計装クラス
    // .NET Framework の WebClient (旧来 API) を OTel でトレースするラッパーを定義する
    public class K1s0InstrumentedWebClient : WebClient
    {
        // k1s0 サービス名: カスタム属性として記録するサービス名
        private readonly string? _k1s0ServiceName;

        // 現在の Activity: リクエスト中の OTel Activity を保持する
        private Activity? _currentActivity;

        // コンストラクタ: k1s0 サービス名を受け取る
        public K1s0InstrumentedWebClient(string? k1s0ServiceName = null)
        {
            // k1s0 サービス名を保存する
            _k1s0ServiceName = k1s0ServiceName;
        }

        // GetWebRequest をオーバーライドして OTel 計装を追加する
        protected override WebRequest GetWebRequest(Uri address)
        {
            // 基底クラスの WebRequest を取得する
            var request = base.GetWebRequest(address);

            // OTel Activity を開始する (HTTP クライアント種別)
            _currentActivity = K1s0OTelActivitySource.StartActivity(
                // 操作名: "WebClient GET" の形式で設定する
                $"WebClient {request.Method}",
                // Activity 種別: Client (外部サービス呼び出し)
                ActivityKind.Client);

            // Activity が開始された場合は属性を設定する
            if (_currentActivity != null)
            {
                // HTTP メソッド属性を設定する
                _currentActivity.SetTag("http.method", request.Method);
                // HTTP URL 属性を設定する
                _currentActivity.SetTag("http.url", address.ToString());
                // k1s0 サービス名属性を設定する (指定がある場合のみ)
                if (_k1s0ServiceName != null)
                {
                    // k1s0 サービス名を属性として記録する
                    _currentActivity.SetTag("k1s0.service.name", _k1s0ServiceName);
                }

                // W3C TraceContext を Traceparent ヘッダーとして HTTP リクエストに注入する
                var traceParent = $"00-{_currentActivity.TraceId}-{_currentActivity.SpanId}-01";
                // Traceparent ヘッダーを設定する
                request.Headers["traceparent"] = traceParent;
            }

            // WebRequest を返す
            return request;
        }

        // GetWebResponse をオーバーライドしてレスポンス情報を Activity に記録する
        protected override WebResponse GetWebResponse(WebRequest request)
        {
            // 基底クラスの WebResponse を取得する
            try
            {
                // レスポンスを取得する
                var response = base.GetWebResponse(request);

                // Activity が開始されている場合はステータスコードを記録する
                if (_currentActivity != null && response is HttpWebResponse httpResponse)
                {
                    // HTTP ステータスコードを整数で設定する
                    _currentActivity.SetTag("http.status_code", (int)httpResponse.StatusCode);
                    // OTel ステータスを Success に設定する
                    _currentActivity.SetStatus(ActivityStatusCode.Ok);
                    // Activity を終了する
                    _currentActivity.Dispose();
                    // Activity 参照をクリアする
                    _currentActivity = null;
                }

                // レスポンスを返す
                return response;
            }
            catch (Exception ex)
            {
                // 例外が発生した場合はエラーを Activity に記録する
                if (_currentActivity != null)
                {
                    // OTel ステータスをエラーに設定する
                    _currentActivity.SetStatus(ActivityStatusCode.Error, ex.Message);
                    // エラーの例外タイプを記録する
                    _currentActivity.SetTag("exception.type", ex.GetType().FullName);
                    // Activity を終了する
                    _currentActivity.Dispose();
                    // Activity 参照をクリアする
                    _currentActivity = null;
                }
                // 例外を再スローする
                throw;
            }
        }
    }

    // K1s0WcfClientBehavior は WCF クライアントの OTel 計装 IClientMessageInspector
    // WCF クライアントのリクエスト/レスポンスを OTel でトレースする
    public class K1s0WcfClientBehavior : System.ServiceModel.Description.IEndpointBehavior, System.ServiceModel.Dispatcher.IClientMessageInspector
    {
        // k1s0 サービス名: カスタム属性として記録するサービス名
        private readonly string? _k1s0ServiceName;

        // 現在の Activity: WCF 呼び出し中の OTel Activity を保持する
        [ThreadStatic]
        private static Activity? _currentActivity;

        // コンストラクタ: k1s0 サービス名を受け取る
        public K1s0WcfClientBehavior(string? k1s0ServiceName = null)
        {
            // k1s0 サービス名を保存する
            _k1s0ServiceName = k1s0ServiceName;
        }

        // IEndpointBehavior.AddBindingParameters: バインディングパラメータを追加する (未使用)
        public void AddBindingParameters(
            // エンドポイント記述子
            System.ServiceModel.Description.ServiceEndpoint endpoint,
            // バインディングパラメータコレクション
            System.ServiceModel.Channels.BindingParameterCollection bindingParameters)
        {
            // バインディングパラメータは変更しないため何もしない
        }

        // IEndpointBehavior.ApplyClientBehavior: クライアントランタイムに計装を適用する
        public void ApplyClientBehavior(
            // エンドポイント記述子
            System.ServiceModel.Description.ServiceEndpoint endpoint,
            // クライアントランタイム
            System.ServiceModel.Dispatcher.ClientRuntime clientRuntime)
        {
            // クライアントランタイムのメッセージインスペクターリストに追加する
            clientRuntime.ClientMessageInspectors.Add(this);
        }

        // IEndpointBehavior.ApplyDispatchBehavior: ディスパッチ動作を適用する (クライアントでは未使用)
        public void ApplyDispatchBehavior(
            // エンドポイント記述子
            System.ServiceModel.Description.ServiceEndpoint endpoint,
            // エンドポイントディスパッチャー
            System.ServiceModel.Dispatcher.EndpointDispatcher endpointDispatcher)
        {
            // クライアント側ではディスパッチ動作は使用しないため何もしない
        }

        // IEndpointBehavior.Validate: エンドポイントを検証する (未使用)
        public void Validate(
            // エンドポイント記述子
            System.ServiceModel.Description.ServiceEndpoint endpoint)
        {
            // 検証は行わないため何もしない
        }

        // IClientMessageInspector.BeforeSendRequest: WCF リクエスト送信前に呼ばれるフック
        public object? BeforeSendRequest(
            // 送信する WCF メッセージ (ref で変更可能)
            ref System.ServiceModel.Channels.Message request,
            // WCF チャネル
            System.ServiceModel.IClientChannel channel)
        {
            // WCF アクション名を取得する
            var action = request.Headers.Action ?? "Unknown";

            // OTel Activity を開始する (WCF クライアント種別)
            _currentActivity = K1s0OTelActivitySource.StartActivity(
                // 操作名: "WCF {action}" の形式で設定する
                $"WCF {action}",
                // Activity 種別: Client (外部サービス呼び出し)
                ActivityKind.Client);

            // Activity が開始された場合は属性を設定する
            if (_currentActivity != null)
            {
                // WCF アクション属性を設定する
                _currentActivity.SetTag("rpc.method", action);
                // WCF エンドポイント URI を設定する
                _currentActivity.SetTag("rpc.service", channel.RemoteAddress?.Uri?.ToString());
                // k1s0 サービス名属性を設定する (指定がある場合のみ)
                if (_k1s0ServiceName != null)
                {
                    // k1s0 サービス名を属性として記録する
                    _currentActivity.SetTag("k1s0.service.name", _k1s0ServiceName);
                }

                // W3C TraceContext を SOAP ヘッダーとして注入する
                var traceParent = $"00-{_currentActivity.TraceId}-{_currentActivity.SpanId}-01";
                // SOAP メッセージヘッダーに traceparent を追加する
                var traceHeader = System.ServiceModel.Channels.MessageHeader.CreateHeader(
                    // ヘッダー名: traceparent
                    "traceparent",
                    // 名前空間: W3C TraceContext 名前空間
                    "https://www.w3.org/TR/trace-context/",
                    // 値: traceParent 文字列
                    traceParent);
                // リクエストヘッダーに追加する
                request.Headers.Add(traceHeader);
            }

            // コリレーション状態として Activity を返す (AfterReceiveReply で使用する)
            return _currentActivity;
        }

        // IClientMessageInspector.AfterReceiveReply: WCF レスポンス受信後に呼ばれるフック
        public void AfterReceiveReply(
            // 受信した WCF レスポンスメッセージ (ref で変更可能)
            ref System.ServiceModel.Channels.Message reply,
            // BeforeSendRequest で返したコリレーション状態
            object? correlationState)
        {
            // コリレーション状態から Activity を取得する
            var activity = correlationState as Activity;

            // Activity が存在する場合はレスポンス情報を記録する
            if (activity != null)
            {
                // WCF レスポンスのアクション名を記録する
                if (reply.Headers.Action != null)
                {
                    // WCF レスポンスアクション属性を設定する
                    activity.SetTag("rpc.response_action", reply.Headers.Action);
                }

                // OTel ステータスを Success に設定する
                activity.SetStatus(ActivityStatusCode.Ok);
                // Activity を終了する
                activity.Dispose();
                // スレッドローカルの Activity をクリアする
                _currentActivity = null;
            }
        }
    }

    // K1s0OTelBuilder は OTel プロバイダーを構築するビルダークラス
    // アプリケーションの起動時に OTel を設定するためのファサードを提供する
    public static class K1s0OTelBuilder
    {
        // Build は TracerProvider を構築して返すファクトリメソッド
        public static TracerProvider Build(
            // OTLP エンドポイント URL: Collector エンドポイントを指定する
            string otlpEndpoint = "http://localhost:4317",
            // k1s0 サービス名: OTel リソース属性として設定するサービス名
            string serviceName = "k1s0-companion",
            // サービスバージョン: OTel リソース属性として設定するバージョン
            string serviceVersion = "0.1.0")
        {
            // Sdk.CreateTracerProviderBuilder で TracerProvider を構築する
            return Sdk.CreateTracerProviderBuilder()
                // ActivitySource を登録する
                .AddSource(K1s0OTelActivitySource.SourceName)
                // OTLP エクスポーターを設定する
                .AddOtlpExporter(opt =>
                {
                    // OTLP エンドポイント URI を設定する
                    opt.Endpoint = new Uri(otlpEndpoint);
                })
                // TracerProvider を構築して返す
                .Build();
        }
    }
}
