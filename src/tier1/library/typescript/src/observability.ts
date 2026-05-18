/**
 * observability.ts — k1s0 tier1 Library TypeScript 実装: Logging / Tracing / Metrics の L3 interface
 * 01_オブザーバビリティ適合仕様.md §観測信号の 3 型（ログ・トレース・メトリクス）に準拠する。
 * OSS 型（winston / otel SDK 等）を公開シグネチャに一切露出しない L3 抽象 interface を宣言する。
 * 全シグナルは OpenTelemetry OTLP 経由で collector に送信される（transport 抽象化）。
 */

/**
 * Attr は構造化ログ / スパン / メトリクスに付与するキー・バリュー属性を宣言する型。
 * OSS の attribute 型（otel attribute.KeyValue 等）を露出しない独自語彙とする。
 */
// Attr 型定義: キー・バリュー属性
export interface Attr {
  // key: 属性名（小文字 snake_case を推奨する）
  readonly key: string;
  // value: 属性値（string 型のみ受け付ける: 型安全性を優先する）
  readonly value: string;
}

/**
 * attr は Attr を生成するショートハンドヘルパー関数。
 * ログ呼び出し側のコードを簡潔にする（attr("tenant_id", id) 形式）。
 */
// attr ヘルパー関数: Attr を生成する
export function attr(key: string, value: string): Attr {
  // Attr オブジェクトを生成して返す
  return { key, value };
}

/**
 * Severity は構造化ログの重要度レベルを宣言する enum。
 * OpenTelemetry Log Data Model §SeverityNumber に準拠した語彙とする。
 */
// Severity 列挙型定義
export const enum Severity {
  // Debug: 詳細デバッグ情報（開発環境での詳細追跡用）
  Debug = "debug",
  // Info: 通常の動作情報（運用ダッシュボードに表示するレベル）
  Info = "info",
  // Warn: 警告（処理は継続するが注意が必要な状態）
  Warn = "warn",
  // Error: エラー（処理が失敗した状態、alert 対象）
  Error = "error",
  // Fatal: 致命的エラー（アプリケーション停止が必要な状態）
  Fatal = "fatal",
}

/**
 * Logger は構造化ログ出力の L3 抽象 interface を宣言する。
 * OpenTelemetry OTLP Logs Signal に準拠したシグネチャとする。
 * OSS 型（winston.Logger / pino.Logger 等）を引数・戻り値に一切含まない。
 */
// Logger インターフェース定義
export interface Logger {
  /**
   * debug は Severity.Debug レベルのログを出力する。
   * ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
   */
  // debug メソッド: デバッグログを出力する
  debug(msg: string, ...attrs: Attr[]): void;

  /**
   * info は Severity.Info レベルのログを出力する。
   * ctx からトレース context を自動伝播する（trace_id / span_id を付与する）。
   */
  // info メソッド: 情報ログを出力する
  info(msg: string, ...attrs: Attr[]): void;

  /**
   * warn は Severity.Warn レベルのログを出力する。
   */
  // warn メソッド: 警告ログを出力する
  warn(msg: string, ...attrs: Attr[]): void;

  /**
   * error は Severity.Error レベルのログを出力する。
   * err は undefined でも可（エラーがない場合でも呼び出せるようにする）。
   */
  // error メソッド: エラーログを出力する
  error(msg: string, err?: Error | undefined, ...attrs: Attr[]): void;

  /**
   * fatal は Severity.Fatal レベルのログを出力してプロセスを終了する。
   * 実装側は log 出力後に process.exit(1) 等を呼び出すことを保証する。
   */
  // fatal メソッド: 致命的エラーログを出力する
  fatal(msg: string, err?: Error | undefined, ...attrs: Attr[]): void;

  /**
   * with は追加属性を持つ派生 Logger を返す（子 logger を生成する）。
   * 全てのログ出力に attrs が自動付与される（tenant_id 等の常駐属性に使用する）。
   */
  // with メソッド: 追加属性を持つ派生 Logger を返す
  with(...attrs: Attr[]): Logger;
}

/**
 * SpanKind は OpenTelemetry SpanKind を宣言する enum。
 * Server / Client / Producer / Consumer / Internal の 5 種を定義する。
 */
// SpanKind 列挙型定義
export const enum SpanKind {
  // Server: 着信 RPC / HTTP リクエストを受け取る側
  Server = "server",
  // Client: 発信 RPC / HTTP リクエストを送る側
  Client = "client",
  // Producer: Messaging キューへの publish 側
  Producer = "producer",
  // Consumer: Messaging キューからの consume 側
  Consumer = "consumer",
  // Internal: プロセス内部の操作（外部通信なし）
  Internal = "internal",
}

/**
 * Span は OpenTelemetry SpanContext の L3 抽象 interface を宣言する。
 * OSS の trace.Span 型を露出しない独自語彙とする。
 */
// Span インターフェース定義
export interface Span {
  /** traceId は分散トレースの root trace 識別子（16 bytes hex 文字列）を返す。 */
  // traceId プロパティ
  readonly traceId: string;

  /** spanId は現在のスパン識別子（8 bytes hex 文字列）を返す。 */
  // spanId プロパティ
  readonly spanId: string;

  /**
   * setAttr はスパンにキー・バリュー属性を追加する。
   * 実装側は OpenTelemetry attribute に変換して付与する。
   */
  // setAttr メソッド: スパンに属性を追加する
  setAttr(...attrs: Attr[]): void;

  /**
   * recordError はスパンにエラーイベントを記録する。
   * err が undefined の場合は何もしない。
   */
  // recordError メソッド: スパンにエラーを記録する
  recordError(err?: Error | undefined): void;

  /**
   * end はスパンを終了する（必ず finally / using で呼び出す）。
   * 呼び出し後はスパンに対する操作は無効となる。
   */
  // end メソッド: スパンを終了する
  end(): void;
}

/**
 * SpanContext はスパンのコンテキスト情報を宣言する型。
 * 親スパンなどの情報を取り出す際に使用する。
 */
// SpanContext 型定義
export interface SpanContext {
  // traceId: トレース識別子
  readonly traceId: string;
  // spanId: スパン識別子
  readonly spanId: string;
}

/**
 * Tracer は分散トレーシングの L3 抽象 interface を宣言する。
 * OpenTelemetry OTLP Traces Signal に準拠したシグネチャとする。
 * OSS の trace.Tracer 型を引数・戻り値に一切含まない。
 */
// Tracer インターフェース定義
export interface Tracer {
  /**
   * start はスパンを開始して Span を返す。
   * name はスパン名（"service.Method" 等の短い識別子を推奨する）。
   * kind は SpanKind（省略時は SpanKind.Internal とする）。
   */
  // start メソッド: スパンを開始する
  start(name: string, kind?: SpanKind, ...attrs: Attr[]): Span;
}

/**
 * MetricMeter は L3 メトリクス記録 interface を宣言する。
 * OpenTelemetry Metrics SDK の MeterProvider / Meter を隠蔽する。
 * OSS 型（prometheus.Counter 等）を公開 API に一切露出しない。
 */
// MetricMeter インターフェース定義
export interface MetricMeter {
  /**
   * counterAdd はカウンターに delta を加算する（delta は正の値のみ許容する）。
   * name は OTLP メトリクス名（"http.server.request.count" 等のドット記法）。
   */
  // counterAdd メソッド: カウンターを加算する
  counterAdd(name: string, delta: number, ...attrs: Attr[]): void;

  /**
   * histogramRecord はヒストグラムに観測値を記録する。
   * name は OTLP メトリクス名、value は観測値。
   */
  // histogramRecord メソッド: ヒストグラムに値を記録する
  histogramRecord(name: string, value: number, ...attrs: Attr[]): void;

  /**
   * gaugeSet はゲージに絶対値を設定する（現在の値を上書きする）。
   * name は OTLP メトリクス名、value は現在値（負の値も許容する）。
   */
  // gaugeSet メソッド: ゲージに値を設定する
  gaugeSet(name: string, value: number, ...attrs: Attr[]): void;
}

/**
 * ObservabilityProvider は Logger / Tracer / MetricMeter を統合した L3 factory interface を宣言する。
 * 呼び出し元は Provider から各 signal instrument を取得して使用する。
 * OSS の telemetry SDK を直接 import することなく観測信号を取得できる。
 */
// ObservabilityProvider インターフェース定義
export interface ObservabilityProvider {
  /**
   * logger はサービス名を指定して Logger を返す。
   * serviceName は OTLP resource.ServiceName 属性に対応する。
   */
  // logger メソッド: Logger を取得する
  logger(serviceName: string): Logger;

  /**
   * tracer はサービス名を指定して Tracer を返す。
   */
  // tracer メソッド: Tracer を取得する
  tracer(serviceName: string): Tracer;

  /**
   * meter はサービス名を指定して MetricMeter を返す。
   */
  // meter メソッド: MetricMeter を取得する
  meter(serviceName: string): MetricMeter;
}

/**
 * LogEntry は Logger.info / Logger.error 等で記録するログエントリを宣言する型。
 * テスト / ドライラン用の stub Logger が in-memory にエントリを蓄積する際に使用する。
 */
// LogEntry 型定義
export interface LogEntry {
  // severity: ログの重要度レベル
  readonly severity: Severity;
  // msg: ログメッセージ
  readonly msg: string;
  // attrs: ログ属性スライス
  readonly attrs: readonly Attr[];
  // err: エラー（Severity.Error / Severity.Fatal の場合のみ設定される）
  readonly err?: Error | undefined;
}
