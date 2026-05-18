// IObservability.cs — k1s0 tier1 Library C# 実装: Logging / Tracing / Metrics の L3 interface
// 01_オブザーバビリティ適合仕様.md §観測信号の 3 型（ログ・トレース・メトリクス）に準拠する。
// OSS 型（Microsoft.Extensions.Logging 等）を公開シグネチャに一切露出しない L3 抽象 interface を宣言する。
// 全シグナルは OpenTelemetry OTLP 経由で collector に送信される（transport 抽象化）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyList / IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.Threading.Tasks: ValueTask / Task に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// Attr は構造化ログ / スパン / メトリクスに付与するキー・バリュー属性を宣言する型。
/// OSS の attribute 型を露出しない独自語彙とする。
/// </summary>
// Attr レコード定義: キー・バリュー属性
public readonly record struct Attr(
    // Key: 属性名（小文字 snake_case を推奨する）
    string Key,
    // Value: 属性値（string 型のみ受け付ける: 型安全性を優先する）
    string Value
);

/// <summary>
/// Severity は構造化ログの重要度レベルを宣言する enum。
/// OpenTelemetry Log Data Model §SeverityNumber に準拠した語彙とする。
/// </summary>
// Severity 列挙型定義
public enum Severity
{
    /// <summary>Debug: 詳細デバッグ情報（開発環境での詳細追跡用）</summary>
    Debug,
    /// <summary>Info: 通常の動作情報（運用ダッシュボードに表示するレベル）</summary>
    Info,
    /// <summary>Warn: 警告（処理は継続するが注意が必要な状態）</summary>
    Warn,
    /// <summary>Error: エラー（処理が失敗した状態、alert 対象）</summary>
    Error,
    /// <summary>Fatal: 致命的エラー（アプリケーション停止が必要な状態）</summary>
    Fatal,
}

/// <summary>
/// ILogger は構造化ログ出力の L3 抽象 interface を宣言する。
/// OpenTelemetry OTLP Logs Signal に準拠したシグネチャとする。
/// OSS 型（Microsoft.Extensions.Logging / Serilog 等）を一切含まない。
/// </summary>
// ILogger インターフェース定義
public interface ILogger
{
    /// <summary>
    /// Debug は Severity.Debug レベルのログを出力する（詳細デバッグ用）。
    /// ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
    /// </summary>
    // Debug メソッド: デバッグログを出力する
    void Debug(string msg, params Attr[] attrs);

    /// <summary>
    /// Info は Severity.Info レベルのログを出力する（通常動作記録用）。
    /// </summary>
    // Info メソッド: 情報ログを出力する
    void Info(string msg, params Attr[] attrs);

    /// <summary>
    /// Warn は Severity.Warn レベルのログを出力する（警告記録用）。
    /// </summary>
    // Warn メソッド: 警告ログを出力する
    void Warn(string msg, params Attr[] attrs);

    /// <summary>
    /// Error は Severity.Error レベルのログを出力する（エラー記録用）。
    /// ex は null 可（エラーがない場合でも呼び出せるようにする）。
    /// </summary>
    // Error メソッド: エラーログを出力する
    void Error(string msg, Exception? ex, params Attr[] attrs);

    /// <summary>
    /// Fatal は Severity.Fatal レベルのログを出力する（致命的エラー用）。
    /// 実装側は log 出力後に Environment.Exit(1) 等を呼び出すことを保証する。
    /// </summary>
    // Fatal メソッド: 致命的エラーログを出力する
    void Fatal(string msg, Exception? ex, params Attr[] attrs);

    /// <summary>
    /// With は追加属性を持つ派生 ILogger を返す（子 logger を生成する）。
    /// 全てのログ出力に attrs が自動付与される（tenant_id 等の常駐属性に使用する）。
    /// </summary>
    // With メソッド: 追加属性を持つ派生 ILogger を返す
    ILogger With(params Attr[] attrs);
}

/// <summary>
/// SpanKind は OpenTelemetry SpanKind を宣言する enum。
/// Server / Client / Producer / Consumer / Internal の 5 種を定義する。
/// </summary>
// SpanKind 列挙型定義
public enum SpanKind
{
    /// <summary>Server: 着信 RPC / HTTP リクエストを受け取る側</summary>
    Server,
    /// <summary>Client: 発信 RPC / HTTP リクエストを送る側</summary>
    Client,
    /// <summary>Producer: Messaging キューへの publish 側</summary>
    Producer,
    /// <summary>Consumer: Messaging キューからの consume 側</summary>
    Consumer,
    /// <summary>Internal: プロセス内部の操作（外部通信なし）</summary>
    Internal,
}

/// <summary>
/// ISpan は OpenTelemetry SpanContext の L3 抽象 interface を宣言する。
/// IDisposable を実装して using で End を自動呼び出しできるようにする。
/// OSS の trace.Span 型を露出しない独自語彙とする。
/// </summary>
// ISpan インターフェース定義
public interface ISpan : IDisposable
{
    /// <summary>TraceId は分散トレースの root trace 識別子（16 bytes hex 文字列）を返す。</summary>
    // TraceId プロパティ
    string TraceId { get; }

    /// <summary>SpanId は現在のスパン識別子（8 bytes hex 文字列）を返す。</summary>
    // SpanId プロパティ
    string SpanId { get; }

    /// <summary>
    /// SetAttr はスパンにキー・バリュー属性を追加する。
    /// 実装側は OpenTelemetry attribute に変換して付与する。
    /// </summary>
    // SetAttr メソッド: スパンに属性を追加する
    void SetAttr(params Attr[] attrs);

    /// <summary>
    /// RecordError はスパンにエラーイベントを記録する。
    /// ex が null の場合は何もしない。
    /// </summary>
    // RecordError メソッド: スパンにエラーを記録する
    void RecordError(Exception? ex);

    /// <summary>
    /// End はスパンを終了する（using ブロックの終了時に自動呼び出しされる）。
    /// </summary>
    // End メソッド: スパンを終了する
    void End();
}

/// <summary>
/// ITracer は分散トレーシングの L3 抽象 interface を宣言する。
/// OpenTelemetry OTLP Traces Signal に準拠したシグネチャとする。
/// OSS の trace.Tracer 型を引数・戻り値に一切含まない。
/// </summary>
// ITracer インターフェース定義
public interface ITracer
{
    /// <summary>
    /// Start はスパンを開始して ISpan を返す（using で End を自動呼び出しする）。
    /// name はスパン名（"Service.Method" 等の短い識別子を推奨する）。
    /// kind は SpanKind（省略時は SpanKind.Internal とする）。
    /// </summary>
    // Start メソッド: スパンを開始する
    ISpan Start(string name, SpanKind kind = SpanKind.Internal, params Attr[] attrs);
}

/// <summary>
/// IMetricMeter は L3 メトリクス記録 interface を宣言する。
/// OpenTelemetry Metrics SDK の MeterProvider / Meter を隠蔽する。
/// OSS 型を公開 API に一切露出しない。
/// </summary>
// IMetricMeter インターフェース定義
public interface IMetricMeter
{
    /// <summary>
    /// CounterAdd はカウンターに delta を加算する（delta は正の値のみ許容する）。
    /// name は OTLP メトリクス名（"http.server.request.count" 等のドット記法）。
    /// </summary>
    // CounterAdd メソッド: カウンターを加算する
    void CounterAdd(string name, double delta, params Attr[] attrs);

    /// <summary>
    /// HistogramRecord はヒストグラムに観測値を記録する。
    /// name は OTLP メトリクス名、value は観測値。
    /// </summary>
    // HistogramRecord メソッド: ヒストグラムに値を記録する
    void HistogramRecord(string name, double value, params Attr[] attrs);

    /// <summary>
    /// GaugeSet はゲージに絶対値を設定する（現在の値を上書きする）。
    /// name は OTLP メトリクス名、value は現在値（負の値も許容する）。
    /// </summary>
    // GaugeSet メソッド: ゲージに値を設定する
    void GaugeSet(string name, double value, params Attr[] attrs);
}

/// <summary>
/// IObservabilityProvider は ILogger / ITracer / IMetricMeter を統合した L3 factory interface を宣言する。
/// 呼び出し元は Provider から各 signal instrument を取得して使用する。
/// OSS の telemetry SDK を直接 import することなく観測信号を取得できる。
/// </summary>
// IObservabilityProvider インターフェース定義
public interface IObservabilityProvider
{
    /// <summary>
    /// Logger はサービス名を指定して ILogger を返す。
    /// serviceName は OTLP resource.ServiceName 属性に対応する（"tier1.key_svc" 等）。
    /// </summary>
    // Logger メソッド: ILogger を取得する
    ILogger Logger(string serviceName);

    /// <summary>
    /// Tracer はサービス名を指定して ITracer を返す。
    /// </summary>
    // Tracer メソッド: ITracer を取得する
    ITracer Tracer(string serviceName);

    /// <summary>
    /// Meter はサービス名を指定して IMetricMeter を返す。
    /// </summary>
    // Meter メソッド: IMetricMeter を取得する
    IMetricMeter Meter(string serviceName);
}
