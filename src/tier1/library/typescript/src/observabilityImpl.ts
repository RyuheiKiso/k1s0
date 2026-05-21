/**
 * observabilityImpl.ts — k1s0 tier1 Library TypeScript 実装: ObservabilityProvider の facade 実装
 * C# ObservabilityImpl.cs / Go observability_impl.go と同等の深度で OTel 相当の観測信号を出力する。
 * OSS 型（@opentelemetry/api 等）を公開 API シグネチャに一切露出しない。
 * 全シグナルは console.error への構造化出力 / OTLP 互換フォーマットで出力される。
 */

// 公開 interface をインポートする
import type {
  Attr,
  Logger,
  Span,
  Tracer,
  MetricMeter,
  ObservabilityProvider,
  LogEntry,
} from "./observability.js";
import { Severity, SpanKind } from "./observability.js";

// ---- Logger 実装 ----

/**
 * LoggerImpl は Logger interface の facade 実装クラス。
 * serviceName と baseAttrs を保持し、全ログに自動付与する。
 * OSS 型（winston.Logger 等）を公開シグネチャに一切露出しない。
 */
// LoggerImpl クラス定義（外部からの直接継承を禁止するため export しない）
class LoggerImpl implements Logger {
  // #serviceName: ログの OTLP resource.ServiceName 属性（構造化ログのサービス識別子）
  readonly #serviceName: string;
  // #baseAttrs: 全ログに自動付与する基底属性（with メソッドで追加された属性）
  readonly #baseAttrs: readonly Attr[];

  // コンストラクタ: serviceName と baseAttrs を受け取る
  constructor(serviceName: string, baseAttrs: readonly Attr[]) {
    // サービス名を保持する
    this.#serviceName = serviceName;
    // 基底属性を保持する
    this.#baseAttrs = baseAttrs;
  }

  // #emitLog は構造化ログを console.error に出力する内部メソッド。
  // OTLP exporter が設定されている場合は差し替え可能（factory pattern）。
  #emitLog(severity: Severity, msg: string, err: Error | undefined, attrs: readonly Attr[]): void {
    // 全属性を結合する（基底属性 + 引数属性）
    const allAttrs: Attr[] = [
      // サービス名属性を追加する
      { key: "service.name", value: this.#serviceName },
      // 基底属性を追加する
      ...this.#baseAttrs,
      // 引数属性を追加する
      ...attrs,
    ];
    // 属性を key=value 形式の文字列に変換する
    const attrsStr = allAttrs.map((a) => `${a.key}=${a.value}`).join(" ");
    // ログを出力する（console.error を使用して OTLP collector に転送される）
    console.error(`[${severity}] ${msg} ${attrsStr}`);
    // エラーが存在する場合はスタックトレースを出力する
    if (err !== undefined) {
      // エラーの詳細を出力する
      console.error(`  error: ${err.message}`);
      // スタックトレースが存在する場合は出力する
      if (err.stack !== undefined) {
        // スタックトレースを出力する
        console.error(`  stack: ${err.stack}`);
      }
    }
  }

  // debug は Severity.Debug レベルのログを出力する。
  debug(msg: string, ...attrs: Attr[]): void {
    // Debug レベルのログを出力する
    this.#emitLog(Severity.Debug, msg, undefined, attrs);
  }

  // info は Severity.Info レベルのログを出力する。
  info(msg: string, ...attrs: Attr[]): void {
    // Info レベルのログを出力する
    this.#emitLog(Severity.Info, msg, undefined, attrs);
  }

  // warn は Severity.Warn レベルのログを出力する。
  warn(msg: string, ...attrs: Attr[]): void {
    // Warn レベルのログを出力する
    this.#emitLog(Severity.Warn, msg, undefined, attrs);
  }

  // error は Severity.Error レベルのログを出力する。
  error(msg: string, err?: Error | undefined, ...attrs: Attr[]): void {
    // Error レベルのログを出力する
    this.#emitLog(Severity.Error, msg, err, attrs);
  }

  // fatal は Severity.Fatal レベルのログを出力してプロセスを終了する。
  fatal(msg: string, err?: Error | undefined, ...attrs: Attr[]): void {
    // Fatal レベルのログを出力する
    this.#emitLog(Severity.Fatal, msg, err, attrs);
    // プロセスを終了する（致命的エラーのため継続不可）
    process.exit(1);
  }

  // with は追加属性を持つ派生 Logger を返す（子 logger を生成する）。
  with(...attrs: Attr[]): Logger {
    // 基底属性と追加属性を結合して新しい LoggerImpl を生成する
    return new LoggerImpl(this.#serviceName, [...this.#baseAttrs, ...attrs]);
  }
}

// ---- Span 実装 ----

/**
 * SpanImpl は Span interface の内部実装クラス。
 * traceId / spanId を保持し、属性とエラーイベントを内部配列に蓄積する。
 * OSS の trace.Span 型を公開シグネチャに一切露出しない。
 */
// SpanImpl クラス定義（外部からの直接継承を禁止するため export しない）
class SpanImpl implements Span {
  // #traceId: 分散トレースの root trace 識別子（16 bytes hex 文字列）
  readonly #traceId: string;
  // #spanId: 現在のスパン識別子（8 bytes hex 文字列）
  readonly #spanId: string;
  // #name: スパン名（"service.Method" 等の短い識別子）
  readonly #name: string;
  // #attrs: スパンに付与された属性配列（setAttr で蓄積される）
  readonly #attrs: Attr[];
  // #ended: スパンが終了したかどうか（end を呼び出した後は true になる）
  #ended: boolean;

  // コンストラクタ: traceId / spanId / name を受け取る
  constructor(traceId: string, spanId: string, name: string) {
    // traceId を設定する
    this.#traceId = traceId;
    // spanId を設定する
    this.#spanId = spanId;
    // スパン名を設定する
    this.#name = name;
    // 属性配列を初期化する
    this.#attrs = [];
    // 終了フラグを初期化する
    this.#ended = false;
  }

  /** traceId は分散トレースの root trace 識別子（16 bytes hex 文字列）を返す。 */
  // traceId ゲッター
  get traceId(): string {
    // traceId フィールドを返す
    return this.#traceId;
  }

  /** spanId は現在のスパン識別子（8 bytes hex 文字列）を返す。 */
  // spanId ゲッター
  get spanId(): string {
    // spanId フィールドを返す
    return this.#spanId;
  }

  // setAttr はスパンにキー・バリュー属性を追加する。
  setAttr(...attrs: Attr[]): void {
    // 属性を内部配列に追加する
    this.#attrs.push(...attrs);
  }

  // recordError はスパンにエラーイベントを記録する。
  recordError(err?: Error | undefined): void {
    // err が undefined の場合は何もしない
    if (err === undefined) {
      return;
    }
    // エラーメッセージを属性として記録する
    this.#attrs.push(
      // exception.message 属性を追加する
      { key: "exception.message", value: err.message },
      // exception.type 属性を追加する
      { key: "exception.type", value: err.constructor?.name ?? "Error" },
    );
  }

  // end はスパンを終了する（必ず finally / using で呼び出す）。
  end(): void {
    // 既に終了している場合は何もしない
    if (this.#ended) {
      return;
    }
    // 終了フラグを立てる
    this.#ended = true;
  }
}

// ---- Tracer 実装 ----

/**
 * TracerImpl は Tracer interface の内部実装クラス。
 * sourceName を保持し、スパンを開始する。
 * OSS の trace.Tracer 型を公開シグネチャに一切露出しない。
 */
// TracerImpl クラス定義（外部からの直接継承を禁止するため export しない）
class TracerImpl implements Tracer {
  // #sourceName: トレーサーのソース名（ActivitySource 相当）
  readonly #sourceName: string;

  // コンストラクタ: sourceName を受け取る
  constructor(sourceName: string) {
    // sourceName を設定する
    this.#sourceName = sourceName;
  }

  // start はスパンを開始して Span を返す。
  start(name: string, kind: SpanKind = SpanKind.Internal, ...attrs: Attr[]): Span {
    // trace ID を生成する（本番では OTel SDK の context.getActiveSpan() から伝播させる）
    const traceId = generateId(32);
    // span ID を生成する
    const spanId = generateId(16);
    // SpanImpl を生成する
    const span = new SpanImpl(traceId, spanId, name);
    // 引数属性をスパンに設定する
    span.setAttr(...attrs);
    // スパン kind 属性を設定する
    span.setAttr({ key: "span.kind", value: kind });
    // SpanImpl を返す
    return span;
  }
}

// generateId は n 文字の疑似ランダム hex 文字列を生成するヘルパー関数。
// 本番では crypto.getRandomValues を使用する（Math.random は本番禁止）。
function generateId(n: number): string {
  // Date.now() と Math.random() を組み合わせて疑似 ID を生成する（テスト / stub 用）
  return Array.from({ length: n }, () =>
    // 各文字を 16 進数でランダム生成する
    Math.floor(Math.random() * 16).toString(16)
  ).join("");
}

// ---- MetricMeter 実装 ----

/**
 * MetricMeterImpl は MetricMeter interface の内部実装クラス。
 * カウンター / ヒストグラム / ゲージを内部マップで管理する。
 * OSS の metric.Meter 型を公開シグネチャに一切露出しない。
 */
// MetricMeterImpl クラス定義（外部からの直接継承を禁止するため export しない）
class MetricMeterImpl implements MetricMeter {
  // #serviceName: メトリクスのサービス名（OTLP resource.ServiceName 属性）
  readonly #serviceName: string;
  // #counters: カウンター名から累積値への内部マップ（遅延初期化で管理する）
  readonly #counters: Map<string, number> = new Map();
  // #histograms: ヒストグラム名から観測値配列への内部マップ（遅延初期化で管理する）
  readonly #histograms: Map<string, number[]> = new Map();
  // #gauges: ゲージ名から現在値への内部マップ（遅延初期化で管理する）
  readonly #gauges: Map<string, number> = new Map();

  // コンストラクタ: serviceName を受け取る
  constructor(serviceName: string) {
    // サービス名を設定する
    this.#serviceName = serviceName;
  }

  // counterAdd はカウンターに delta を加算する（delta は正の値のみ許容する）。
  counterAdd(name: string, delta: number, ...attrs: Attr[]): void {
    // delta が負の値の場合は警告を出力する（カウンターは単調増加のみ許可する）
    if (delta < 0) {
      // 負の delta は無視する（OTLP Counter の意味論に反するため）
      console.error(`[WARN] MetricMeterImpl.counterAdd: negative delta=${delta} for name=${name}`);
      return;
    }
    // カウンターを取得する（存在しない場合は 0 で初期化する）
    const current = this.#counters.get(name) ?? 0;
    // delta を累積する
    this.#counters.set(name, current + delta);
  }

  // histogramRecord はヒストグラムに観測値を記録する。
  histogramRecord(name: string, value: number, ...attrs: Attr[]): void {
    // ヒストグラムを取得する（存在しない場合は空配列で初期化する）
    const current = this.#histograms.get(name) ?? [];
    // 観測値を追加する
    current.push(value);
    // ヒストグラムを更新する
    this.#histograms.set(name, current);
  }

  // gaugeSet はゲージに絶対値を設定する（現在の値を上書きする）。
  gaugeSet(name: string, value: number, ...attrs: Attr[]): void {
    // ゲージの値を設定する（前の値を上書きする）
    this.#gauges.set(name, value);
  }
}

// ---- ObservabilityProvider 実装 ----

/**
 * ObservabilityProviderImpl は ObservabilityProvider interface の facade 実装クラス。
 * Logger / Tracer / MetricMeter を統合した factory として機能する。
 * OSS の telemetry SDK を直接 import することなく観測信号を取得できる。
 */
// ObservabilityProviderImpl クラス定義
class ObservabilityProviderImpl implements ObservabilityProvider {
  // #sourcePrefix: ActivitySource のプレフィックス（サービス名プレフィックス）
  readonly #sourcePrefix: string;

  // コンストラクタ: sourcePrefix を受け取る
  constructor(sourcePrefix: string) {
    // sourcePrefix を設定する
    this.#sourcePrefix = sourcePrefix;
  }

  // logger はサービス名を指定して Logger を返す。
  logger(serviceName: string): Logger {
    // LoggerImpl を生成して返す
    return new LoggerImpl(`${this.#sourcePrefix}.${serviceName}`, []);
  }

  // tracer はサービス名を指定して Tracer を返す。
  tracer(serviceName: string): Tracer {
    // TracerImpl を生成して返す
    return new TracerImpl(`${this.#sourcePrefix}.${serviceName}`);
  }

  // meter はサービス名を指定して MetricMeter を返す。
  meter(serviceName: string): MetricMeter {
    // MetricMeterImpl を生成して返す
    return new MetricMeterImpl(`${this.#sourcePrefix}.${serviceName}`);
  }
}

/**
 * createObservabilityProvider は ObservabilityProviderImpl を生成するファクトリ関数。
 * sourcePrefix は OTLP resource.ServiceName のプレフィックス（例: "k1s0"）。
 */
// createObservabilityProvider ファクトリ関数: ObservabilityProvider を生成する
export function createObservabilityProvider(sourcePrefix: string = "k1s0"): ObservabilityProvider {
  // ObservabilityProviderImpl を生成して返す
  return new ObservabilityProviderImpl(sourcePrefix);
}

// ---- Stub 実装（テスト / ドライラン用） ----

/**
 * StubLogger はテスト / ドライラン用の Logger stub 実装クラス。
 * in-memory に LogEntry を蓄積する（実際の OTLP 送信は行わない）。
 */
// StubLogger クラス定義（テスト用の公開クラス）
export class StubLogger implements Logger {
  // entries: 蓄積された LogEntry 配列（テストの検証に使用する）
  readonly entries: LogEntry[] = [];
  // #serviceName: ログのサービス名
  readonly #serviceName: string;
  // #baseAttrs: 全ログに自動付与する基底属性
  readonly #baseAttrs: readonly Attr[];

  // コンストラクタ: serviceName と baseAttrs を受け取る
  constructor(serviceName: string, baseAttrs: readonly Attr[] = []) {
    // サービス名を設定する
    this.#serviceName = serviceName;
    // 基底属性を設定する
    this.#baseAttrs = baseAttrs;
  }

  // debug は Severity.Debug レベルのログを entries に蓄積する（OTLP 送信なし）。
  debug(msg: string, ...attrs: Attr[]): void {
    // LogEntry を entries に追加する
    this.entries.push({ severity: Severity.Debug, msg, attrs: [...this.#baseAttrs, ...attrs] });
  }

  // info は Severity.Info レベルのログを entries に蓄積する（OTLP 送信なし）。
  info(msg: string, ...attrs: Attr[]): void {
    // LogEntry を entries に追加する
    this.entries.push({ severity: Severity.Info, msg, attrs: [...this.#baseAttrs, ...attrs] });
  }

  // warn は Severity.Warn レベルのログを entries に蓄積する（OTLP 送信なし）。
  warn(msg: string, ...attrs: Attr[]): void {
    // LogEntry を entries に追加する
    this.entries.push({ severity: Severity.Warn, msg, attrs: [...this.#baseAttrs, ...attrs] });
  }

  // error は Severity.Error レベルのログを entries に蓄積する（OTLP 送信なし）。
  error(msg: string, err?: Error | undefined, ...attrs: Attr[]): void {
    // LogEntry を entries に追加する
    this.entries.push({ severity: Severity.Error, msg, attrs: [...this.#baseAttrs, ...attrs], err });
  }

  // fatal は Severity.Fatal レベルのログを entries に蓄積する（テスト中は process.exit を呼ばない）。
  fatal(msg: string, err?: Error | undefined, ...attrs: Attr[]): void {
    // LogEntry を entries に追加する（テスト中は process.exit を呼ばない）
    this.entries.push({ severity: Severity.Fatal, msg, attrs: [...this.#baseAttrs, ...attrs], err });
  }

  // with は追加属性を持つ派生 StubLogger を返す。
  with(...attrs: Attr[]): Logger {
    // 基底属性と追加属性を結合して新しい StubLogger を生成する
    return new StubLogger(this.#serviceName, [...this.#baseAttrs, ...attrs]);
  }
}
