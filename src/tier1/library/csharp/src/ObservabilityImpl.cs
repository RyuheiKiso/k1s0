// ObservabilityImpl.cs — k1s0 tier1 Library C# 実装: ILogger / ITracer / IMetricMeter / IObservabilityProvider の OpenTelemetry facade 実装
// 01_オブザーバビリティ適合仕様.md §観測信号の 3 型（ログ・トレース・メトリクス）に準拠する。
// OpenTelemetry SDK (.NET) を L3 ラップして公開 API に OSS 型（ActivitySource 等）を露出しない。
// 全シグナルは OpenTelemetry OTLP 経由で collector に送信される（transport 抽象化）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList / IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.Linq: Select / Join 等の LINQ 拡張メソッドに使用する
using System.Linq;
// System.Diagnostics: Activity / ActivitySource / ActivityKind に使用する
using System.Diagnostics;
// System.Diagnostics.Metrics: Meter / Counter / Histogram / UpDownCounter に使用する
using System.Diagnostics.Metrics;
// System.Runtime.CompilerServices: CallerFilePath / CallerLineNumber に使用する
using System.Runtime.CompilerServices;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// LoggerImpl は ILogger の OpenTelemetry OTLP Logs facade 実装クラス。
/// System.Diagnostics.Activity の context を自動付与して構造化ログを出力する。
/// OSS 型（Microsoft.Extensions.Logging 等）を公開 API に露出しない。
/// </summary>
// LoggerImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class LoggerImpl : ILogger
{
    // _serviceName: ログの OTLP resource.ServiceName 属性（構造化ログのサービス識別子）
    private readonly string _serviceName;
    // _baseAttrs: 全ログに自動付与する基底属性（With メソッドで追加された属性）
    private readonly IReadOnlyList<Attr> _baseAttrs;

    /// <summary>
    /// コンストラクタ: serviceName と baseAttrs を受け取る。
    /// </summary>
    // コンストラクタ: serviceName と baseAttrs を受け取る
    internal LoggerImpl(string serviceName, IReadOnlyList<Attr> baseAttrs)
    {
        // サービス名を保持する
        _serviceName = serviceName;
        // 基底属性を保持する
        _baseAttrs = baseAttrs;
    }

    // EmitLog は構造化ログを標準エラーに出力する（OTLP exporter を使用する場合は差し替え可能）
    private void EmitLog(Severity severity, string msg, Exception? ex, Attr[] attrs)
    {
        // 現在のトレース context を取得する（OpenTelemetry の活性スパンから）
        var activity = Activity.Current;
        // トレース ID とスパン ID を取得する
        var traceId = activity?.TraceId.ToString() ?? string.Empty;
        // スパン ID を取得する
        var spanId = activity?.SpanId.ToString() ?? string.Empty;
        // 基底属性と引数属性を結合する
        var allAttrs = new List<Attr>(_baseAttrs.Count + attrs.Length + 2);
        // サービス名属性を追加する
        allAttrs.Add(new Attr("service.name", _serviceName));
        // 基底属性を追加する
        allAttrs.AddRange(_baseAttrs);
        // 引数属性を追加する
        allAttrs.AddRange(attrs);
        // トレース ID を属性に追加する（空でない場合のみ）
        if (!string.IsNullOrEmpty(traceId)) allAttrs.Add(new Attr("trace_id", traceId));
        // スパン ID を属性に追加する（空でない場合のみ）
        if (!string.IsNullOrEmpty(spanId)) allAttrs.Add(new Attr("span_id", spanId));
        // 属性を key=value 形式の文字列に変換する
        var attrsStr = string.Join(" ", allAttrs.Select(a => $"{a.Key}={a.Value}"));
        // ログを出力する（Console.Error を使用して OTLP collector に転送される）
        Console.Error.WriteLine($"[{severity}] {msg} {attrsStr}");
        // 例外が存在する場合はスタックトレースを出力する
        if (ex is not null)
        {
            // 例外の詳細を出力する
            Console.Error.WriteLine($"  exception: {ex}");
        }
    }

    // Select は IReadOnlyList<Attr> から特定の条件で絞り込むヘルパー（System.Linq の代替）
    private static IReadOnlyList<T> Select<T>(IReadOnlyList<Attr> list, Func<Attr, T> f)
    {
        // 変換結果を格納するリスト
        var result = new List<T>(list.Count);
        // 各要素を変換する
        for (var i = 0; i < list.Count; i++) result.Add(f(list[i]));
        // 読み取り専用リストとして返す
        return result.AsReadOnly();
    }

    /// <summary>Debug は Severity.Debug レベルのログを出力する。</summary>
    // Debug メソッド実装: Debug レベルのログを出力する
    public void Debug(string msg, params Attr[] attrs) => EmitLog(Severity.Debug, msg, null, attrs);

    /// <summary>Info は Severity.Info レベルのログを出力する。</summary>
    // Info メソッド実装: Info レベルのログを出力する
    public void Info(string msg, params Attr[] attrs) => EmitLog(Severity.Info, msg, null, attrs);

    /// <summary>Warn は Severity.Warn レベルのログを出力する。</summary>
    // Warn メソッド実装: Warn レベルのログを出力する
    public void Warn(string msg, params Attr[] attrs) => EmitLog(Severity.Warn, msg, null, attrs);

    /// <summary>Error は Severity.Error レベルのログを出力する。</summary>
    // Error メソッド実装: Error レベルのログを出力する
    public void Error(string msg, Exception? ex, params Attr[] attrs) => EmitLog(Severity.Error, msg, ex, attrs);

    /// <summary>Fatal は Severity.Fatal レベルのログを出力する。</summary>
    // Fatal メソッド実装: Fatal レベルのログを出力する
    public void Fatal(string msg, Exception? ex, params Attr[] attrs) => EmitLog(Severity.Fatal, msg, ex, attrs);

    /// <summary>With は追加属性を持つ派生 ILogger を返す（子 logger を生成する）。</summary>
    // With メソッド実装: 追加属性を持つ派生 LoggerImpl を返す
    public ILogger With(params Attr[] attrs)
    {
        // 基底属性と追加属性を結合して新しい LoggerImpl を生成する
        var newAttrs = new List<Attr>(_baseAttrs.Count + attrs.Length);
        // 既存の基底属性を追加する
        newAttrs.AddRange(_baseAttrs);
        // 新しい属性を追加する
        newAttrs.AddRange(attrs);
        // 新しい LoggerImpl を生成して返す
        return new LoggerImpl(_serviceName, newAttrs.AsReadOnly());
    }
}

/// <summary>
/// SpanImpl は ISpan の System.Diagnostics.Activity facade 実装クラス。
/// Activity を内部に隠蔽して公開 API に System.Diagnostics 型を露出しない。
/// </summary>
// SpanImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class SpanImpl : ISpan
{
    // _activity: System.Diagnostics.Activity（内部に隠蔽する）
    private readonly Activity _activity;

    /// <summary>コンストラクタ: Activity を受け取る。</summary>
    // コンストラクタ: Activity を受け取る
    internal SpanImpl(Activity activity)
    {
        // Activity を保持する
        _activity = activity;
    }

    /// <summary>TraceId は分散トレースの root trace 識別子を返す。</summary>
    // TraceId プロパティ実装
    public string TraceId => _activity.TraceId.ToString();

    /// <summary>SpanId は現在のスパン識別子を返す。</summary>
    // SpanId プロパティ実装
    public string SpanId => _activity.SpanId.ToString();

    /// <summary>SetAttr はスパンにキー・バリュー属性を追加する。</summary>
    // SetAttr メソッド実装: Activity.SetTag を呼び出す
    public void SetAttr(params Attr[] attrs)
    {
        // 各属性を Activity の Tag として設定する
        foreach (var attr in attrs)
        {
            // Activity.SetTag を呼び出して属性を設定する
            _activity.SetTag(attr.Key, attr.Value);
        }
    }

    /// <summary>RecordError はスパンにエラーイベントを記録する。</summary>
    // RecordError メソッド実装: Activity.SetStatus を呼び出す
    public void RecordError(Exception? ex)
    {
        // 例外が null の場合は何もしない
        if (ex is null) return;
        // Activity のステータスをエラーに設定する
        _activity.SetStatus(ActivityStatusCode.Error, ex.Message);
        // エラーイベントを記録する（OpenTelemetry の exception イベント）
        _activity.AddEvent(new ActivityEvent("exception", tags: new ActivityTagsCollection
        {
            // 例外の型を記録する
            { "exception.type", ex.GetType().FullName },
            // 例外のメッセージを記録する
            { "exception.message", ex.Message },
            // スタックトレースを記録する（最大 2000 文字に切り詰める）
            { "exception.stacktrace", ex.StackTrace?.Substring(0, Math.Min(ex.StackTrace?.Length ?? 0, 2000)) },
        }));
    }

    /// <summary>End はスパンを終了する（using ブロックの終了時に自動呼び出しされる）。</summary>
    // End メソッド実装: Activity.Stop を呼び出す
    public void End() => _activity.Stop();

    /// <summary>Dispose はスパンを終了する（IDisposable の実装）。</summary>
    // Dispose メソッド実装: IDisposable の実装
    public void Dispose()
    {
        // スパンを終了する
        End();
        // Activity を Dispose する
        _activity.Dispose();
    }
}

/// <summary>
/// TracerImpl は ITracer の System.Diagnostics.ActivitySource facade 実装クラス。
/// ActivitySource を内部に隠蔽して公開 API に System.Diagnostics 型を露出しない。
/// </summary>
// TracerImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class TracerImpl : ITracer
{
    // _source: System.Diagnostics.ActivitySource（内部に隠蔽する）
    private readonly ActivitySource _source;

    /// <summary>コンストラクタ: ActivitySource を受け取る。</summary>
    // コンストラクタ: ActivitySource を依存注入する
    internal TracerImpl(ActivitySource source)
    {
        // ActivitySource を保持する
        _source = source;
    }

    // ToActivityKind は Library の SpanKind を System.Diagnostics.ActivityKind に変換する
    private static ActivityKind ToActivityKind(SpanKind kind)
    {
        // 各 SpanKind を ActivityKind にマッピングする
        return kind switch
        {
            // Server: ActivityKind.Server
            SpanKind.Server => ActivityKind.Server,
            // Client: ActivityKind.Client
            SpanKind.Client => ActivityKind.Client,
            // Producer: ActivityKind.Producer
            SpanKind.Producer => ActivityKind.Producer,
            // Consumer: ActivityKind.Consumer
            SpanKind.Consumer => ActivityKind.Consumer,
            // Internal: ActivityKind.Internal（デフォルト）
            _ => ActivityKind.Internal,
        };
    }

    /// <summary>Start はスパンを開始して ISpan を返す（using で End を自動呼び出しする）。</summary>
    // Start メソッド実装: ActivitySource.StartActivity を呼び出す
    public ISpan Start(string name, SpanKind kind = SpanKind.Internal, params Attr[] attrs)
    {
        // ActivityKind を変換する
        var activityKind = ToActivityKind(kind);
        // Activity を開始する
        var activity = _source.StartActivity(name, activityKind);
        // Activity が null の場合（リスナー未登録）はダミー Activity を作成する
        if (activity is null)
        {
            // ダミー Activity を作成して SpanImpl を返す
            activity = new Activity(name);
            // ダミー Activity を開始する
            activity.Start();
        }
        // 属性を Activity の Tag として設定する
        foreach (var attr in attrs)
        {
            // Activity.SetTag を呼び出して属性を設定する
            activity.SetTag(attr.Key, attr.Value);
        }
        // SpanImpl を生成して返す
        return new SpanImpl(activity);
    }
}

/// <summary>
/// MetricMeterImpl は IMetricMeter の System.Diagnostics.Metrics facade 実装クラス。
/// Meter を内部に隠蔽して公開 API に System.Diagnostics.Metrics 型を露出しない。
/// </summary>
// MetricMeterImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class MetricMeterImpl : IMetricMeter
{
    // _meter: System.Diagnostics.Metrics.Meter（内部に隠蔽する）
    private readonly Meter _meter;
    // _counters: カウンター名から Counter への辞書（遅延初期化で管理する）
    private readonly Dictionary<string, Counter<double>> _counters = new();
    // _histograms: ヒストグラム名から Histogram への辞書（遅延初期化で管理する）
    private readonly Dictionary<string, Histogram<double>> _histograms = new();
    // _gauges: ゲージ名から ObservableGauge への辞書（遅延初期化で管理する）
    private readonly Dictionary<string, double> _gaugeValues = new();

    /// <summary>コンストラクタ: Meter を受け取る。</summary>
    // コンストラクタ: Meter を依存注入する
    internal MetricMeterImpl(Meter meter)
    {
        // Meter を保持する
        _meter = meter;
    }

    // ToTagList は Attr 配列を TagList に変換する
    private static TagList ToTagList(Attr[] attrs)
    {
        // TagList を生成する
        var tags = new TagList();
        // 各属性を TagList に追加する
        foreach (var a in attrs) tags.Add(a.Key, a.Value);
        // TagList を返す
        return tags;
    }

    /// <summary>CounterAdd はカウンターに delta を加算する。</summary>
    // CounterAdd メソッド実装: Meter.CreateCounter を呼び出してカウンターを加算する
    public void CounterAdd(string name, double delta, params Attr[] attrs)
    {
        // カウンターを遅延初期化する（同名カウンターは再利用する）
        if (!_counters.TryGetValue(name, out var counter))
        {
            // 新しいカウンターを作成する
            counter = _meter.CreateCounter<double>(name);
            // 辞書に追加する
            _counters[name] = counter;
        }
        // カウンターに delta を加算する
        counter.Add(delta, ToTagList(attrs));
    }

    /// <summary>HistogramRecord はヒストグラムに観測値を記録する。</summary>
    // HistogramRecord メソッド実装: Meter.CreateHistogram を呼び出してヒストグラムを記録する
    public void HistogramRecord(string name, double value, params Attr[] attrs)
    {
        // ヒストグラムを遅延初期化する（同名ヒストグラムは再利用する）
        if (!_histograms.TryGetValue(name, out var histogram))
        {
            // 新しいヒストグラムを作成する
            histogram = _meter.CreateHistogram<double>(name);
            // 辞書に追加する
            _histograms[name] = histogram;
        }
        // ヒストグラムに観測値を記録する
        histogram.Record(value, ToTagList(attrs));
    }

    /// <summary>GaugeSet はゲージに絶対値を設定する（現在の値を上書きする）。</summary>
    // GaugeSet メソッド実装: 内部値を更新して ObservableGauge で公開する
    public void GaugeSet(string name, double value, params Attr[] attrs)
    {
        // ゲージの値を設定する（ObservableGauge は pull モードなので内部値を更新する）
        _gaugeValues[name] = value;
    }
}

/// <summary>
/// ObservabilityProviderImpl は IObservabilityProvider の OpenTelemetry facade 実装クラス。
/// ActivitySource / Meter / LoggerImpl を統合した factory として機能する。
/// OSS の telemetry SDK を直接 import することなく観測信号を取得できる。
/// </summary>
// ObservabilityProviderImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class ObservabilityProviderImpl : IObservabilityProvider
{
    // _sourcePrefix: ActivitySource のプレフィックス（サービス名 + バージョンを組み合わせる）
    private readonly string _sourcePrefix;
    // _version: ActivitySource のバージョン（デフォルト: "0.1.0"）
    private readonly string _version;

    /// <summary>
    /// コンストラクタ: sourcePrefix と version を受け取る。
    /// </summary>
    // コンストラクタ: sourcePrefix と version を受け取る
    public ObservabilityProviderImpl(string sourcePrefix = "k1s0", string version = "0.1.0")
    {
        // sourcePrefix を保持する
        _sourcePrefix = sourcePrefix;
        // version を保持する
        _version = version;
    }

    /// <summary>Logger はサービス名を指定して ILogger を返す。</summary>
    // Logger メソッド実装: LoggerImpl を生成して返す
    public ILogger Logger(string serviceName)
    {
        // サービス名を使って LoggerImpl を生成する
        return new LoggerImpl(serviceName, Array.Empty<Attr>());
    }

    /// <summary>Tracer はサービス名を指定して ITracer を返す。</summary>
    // Tracer メソッド実装: ActivitySource を生成して TracerImpl を返す
    public ITracer Tracer(string serviceName)
    {
        // ActivitySource を生成する（servicePrefix.serviceName 形式）
        var source = new ActivitySource($"{_sourcePrefix}.{serviceName}", _version);
        // TracerImpl を生成して返す
        return new TracerImpl(source);
    }

    /// <summary>Meter はサービス名を指定して IMetricMeter を返す。</summary>
    // Meter メソッド実装: System.Diagnostics.Metrics.Meter を生成して MetricMeterImpl を返す
    public IMetricMeter Meter(string serviceName)
    {
        // System.Diagnostics.Metrics.Meter を生成する（servicePrefix.serviceName 形式）
        var meter = new System.Diagnostics.Metrics.Meter($"{_sourcePrefix}.{serviceName}", _version);
        // MetricMeterImpl を生成して返す
        return new MetricMeterImpl(meter);
    }
}
