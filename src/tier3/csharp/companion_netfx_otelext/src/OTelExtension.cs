// k1s0 Companion .NET Framework OTel 拡張エントリーポイント
// OTLP exporter の初期化と OpenTelemetry TracerProvider の構築を担う
// .NET Framework 4.6.2 向けの OpenTelemetry.Exporter.OpenTelemetryProtocol 相当実装

// System 名前空間: Uri / Environment 等の基本型に使用する
using System;
// OpenTelemetry コアライブラリ: ITracerProvider 取得に使用する
using OpenTelemetry;
// OpenTelemetry トレース: TracerProviderBuilder に使用する
using OpenTelemetry.Trace;
// OpenTelemetry OTLP exporter: Exporter 設定に使用する
using OpenTelemetry.Exporter;
// OpenTelemetry HTTP instrumentation: HttpWebRequest / WebClient のトレース計装に使用する
using OpenTelemetry.Instrumentation.Http;

namespace K1s0.Companion.NetFx.OTelExt
{
    /// <summary>
    /// k1s0 .NET Framework OTel 拡張エントリーポイント
    /// OTLP exporter を使って OpenTelemetry TracerProvider を構築し、
    /// CLR profiler hooks と JWT claim processor を組み合わせて動作する
    /// </summary>
    public static class OTelExtension
    {
        // TracerProvider のシングルトンインスタンスを保持するフィールド
        // アプリケーションのライフサイクルと同期して管理する
        private static TracerProvider? _tracerProvider;

        // 初期化済みフラグ: 二重初期化を防止するために使用する
        private static bool _initialized = false;

        // ロックオブジェクト: スレッドセーフな初期化のために使用する
        private static readonly object _lock = new object();

        /// <summary>
        /// OTel 拡張を初期化して OTLP exporter を設定する
        /// エンドポイントが指定されない場合は環境変数 OTEL_EXPORTER_OTLP_ENDPOINT を参照する
        /// </summary>
        /// <param name="serviceName">サービス名（OTel resource attribute service.name に設定される）</param>
        /// <param name="otlpEndpoint">OTLP gRPC エンドポイント（省略時は環境変数から取得する）</param>
        /// <returns>初期化済みの TracerProvider インスタンス</returns>
        public static TracerProvider Initialize(
            // サービス名: OTel resource に埋め込む識別子
            string serviceName,
            // OTLP エンドポイント: null の場合は環境変数から取得する
            string? otlpEndpoint = null)
        {
            // ロックを取得してスレッドセーフに初期化する
            lock (_lock)
            {
                // 既に初期化済みの場合は既存の TracerProvider を返す
                if (_initialized && _tracerProvider != null)
                {
                    // 二重初期化を防ぐためそのまま返す
                    return _tracerProvider;
                }

                // OTLP エンドポイントを決定する（引数 → 環境変数 → デフォルト値の順に参照する）
                var endpoint = otlpEndpoint
                    ?? Environment.GetEnvironmentVariable("OTEL_EXPORTER_OTLP_ENDPOINT")
                    ?? "http://localhost:4317";

                // TracerProvider を構築する
                // HTTP instrumentation（HttpWebRequest / WebClient / HttpClient）を有効化する
                _tracerProvider = Sdk.CreateTracerProviderBuilder()
                    // サービス名を resource として設定する
                    .SetResourceBuilder(
                        OpenTelemetry.Resources.ResourceBuilder
                            .CreateDefault()
                            // service.name attribute を設定する
                            .AddService(serviceName))
                    // HttpWebRequest / WebClient / HttpClient のトレース計装を有効化する
                    .AddHttpClientInstrumentation()
                    // k1s0 OTel ソース（ActivitySource）を登録する
                    .AddSource(K1s0ActivitySources.CompanionSource)
                    // OTLP exporter を設定する（gRPC プロトコルを使用する）
                    .AddOtlpExporter(opts =>
                    {
                        // エンドポイント URI を設定する
                        opts.Endpoint = new Uri(endpoint);
                        // プロトコルを grpc に設定する（.NET Framework 互換の gRPC 実装を使用する）
                        opts.Protocol = OtlpExportProtocol.Grpc;
                    })
                    // TracerProvider を構築して返す
                    .Build();

                // 初期化済みフラグを立てる
                _initialized = true;

                // 構築した TracerProvider を返す
                return _tracerProvider;
            }
        }

        /// <summary>
        /// TracerProvider を破棄してリソースを解放する
        /// アプリケーション終了時に呼び出すこと
        /// </summary>
        public static void Shutdown()
        {
            // ロックを取得してスレッドセーフに破棄する
            lock (_lock)
            {
                // TracerProvider が初期化済みの場合のみ破棄する
                if (_tracerProvider != null)
                {
                    // TracerProvider を破棄してリソースを解放する
                    _tracerProvider.Dispose();
                    // フィールドを null に戻す
                    _tracerProvider = null;
                    // 初期化済みフラグを解除する
                    _initialized = false;
                }
            }
        }

        /// <summary>
        /// 現在の TracerProvider インスタンスを取得する
        /// 初期化前に呼び出した場合は null を返す
        /// </summary>
        public static TracerProvider? Current
        {
            // 現在の TracerProvider を取得するプロパティ
            get
            {
                // ロックなしで読み取る（参照型の読み取りはアトミック）
                return _tracerProvider;
            }
        }
    }

    /// <summary>
    /// k1s0 ActivitySource 定数クラス
    /// OpenTelemetry の手動計装で使用する ActivitySource 名を一元管理する
    /// </summary>
    public static class K1s0ActivitySources
    {
        // k1s0 Companion の主要 ActivitySource 名
        // TracerProvider.AddSource() に渡すために使用する
        public const string CompanionSource = "k1s0.companion.netfx";

        // JWT claim processor の ActivitySource 名
        // JwtClaimProcessor が生成するスパンに使用する
        public const string JwtClaimSource = "k1s0.companion.netfx.jwt";

        // CLR profiler hook の ActivitySource 名
        // ProfilerHooks が生成するスパンに使用する
        public const string ProfilerSource = "k1s0.companion.netfx.profiler";
    }
}
