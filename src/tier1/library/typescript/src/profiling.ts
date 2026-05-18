/**
 * profiling.ts — k1s0 tier1 Library TypeScript 実装: Profiling の L2* interface
 * 01_オブザーバビリティ適合仕様.md §プロファイル収集（backend 専用 L2*）に準拠する。
 * pprof / Pyroscope 等 OSS の API を Library 独自語彙に翻訳する L2* facade を宣言する。
 * 公開シグネチャに OSS 型を露出しない（pprof 等は一切含まない）。
 */

/**
 * ProfileType はプロファイルの種別を宣言する enum。
 * L2* 族内共通 API として pprof の profile type 名称に準拠した語彙とする。
 */
// ProfileType 列挙型定義
export const enum ProfileType {
  // Cpu: CPU 使用時間のサンプリングプロファイル
  Cpu = "cpu",
  // Heap: ヒープメモリ使用状況のプロファイル
  Heap = "heap",
  // EventLoop: イベントループ遅延プロファイル（Node.js 専用）
  EventLoop = "event_loop",
  // GcPressure: ガベージコレクション負荷プロファイル
  GcPressure = "gc_pressure",
  // Allocs: メモリアロケーションのプロファイル
  Allocs = "allocs",
}

/**
 * ProfileOptions は Profiler.start / Profiler.capture に渡すオプションを宣言する型。
 * L2* 族内共通 API として OSS 概念を Library 独自語彙に翻訳する。
 */
// ProfileOptions 型定義
export interface ProfileOptions {
  // sampleRateHz: サンプリングレート（Hz / samples per second）。0 = 実装固有のデフォルト値
  readonly sampleRateHz?: number | undefined;
  // maxDurationMs: プロファイル収集の最大継続時間（ミリ秒、0 = 無制限）
  readonly maxDurationMs?: number | undefined;
  // labels: プロファイルに付与するラベル（tenant_id / service 等）
  readonly labels?: Readonly<Record<string, string>> | undefined;
}

/**
 * ProfileSummary はプロファイル収集結果のサマリーを宣言する型。
 * OSS の Profile オブジェクトを直接露出せず Library 語彙で結果を表現する。
 */
// ProfileSummary 型定義
export interface ProfileSummary {
  // profileType: 収集したプロファイルの種別
  readonly profileType: ProfileType;
  // sampleCount: 収集したサンプル数
  readonly sampleCount: number;
  // durationMs: 収集にかかった時間（ミリ秒単位）
  readonly durationMs: number;
  // sizeBytes: プロファイルデータのバイト数
  readonly sizeBytes: number;
}

/**
 * Profiler は L2* プロファイル収集 interface を宣言する。
 * 族内共通 API として pprof / Pyroscope 等を Library 独自語彙に翻訳する。
 * OSS の profiler 型を引数・戻り値に一切含まない。
 */
// Profiler インターフェース定義
export interface Profiler {
  /**
   * start はプロファイルの連続収集を開始する。
   * profileType で収集するプロファイル種別を指定する。
   * opts で収集オプションを指定する（未指定 = デフォルト設定を使用する）。
   */
  // start メソッド: プロファイル収集を開始する
  start(profileType: ProfileType, opts?: ProfileOptions): Promise<void>;

  /**
   * stop はプロファイルの収集を停止する。
   */
  // stop メソッド: プロファイル収集を停止する
  stop(profileType: ProfileType): Promise<void>;

  /**
   * capture は単発のプロファイルを収集して Uint8Array で返す。
   * profileType で収集するプロファイル種別を指定する。
   * 戻り値は pprof 形式のバイナリデータ。
   */
  // capture メソッド: 単発プロファイルを収集する
  capture(profileType: ProfileType, opts?: ProfileOptions): Promise<[Uint8Array, ProfileSummary]>;

  /**
   * isRunning はプロファイル収集が実行中かどうかを返す。
   */
  // isRunning メソッド: 実行中かどうかを返す
  isRunning(profileType: ProfileType): boolean;
}

/**
 * ContinuousProfilerConfig は常時プロファイリング（Pyroscope 等）の設定を宣言する型。
 * L2* として Pyroscope Agent の設定語彙を Library 独自語彙に翻訳する。
 */
// ContinuousProfilerConfig 型定義
export interface ContinuousProfilerConfig {
  // endpointUrl: Pyroscope / OTLP profiles エンドポイント URL
  readonly endpointUrl: string;
  // applicationName: プロファイルを識別するアプリケーション名
  readonly applicationName: string;
  // sampleRateHz: CPU プロファイルのサンプリングレート（Hz 単位）
  readonly sampleRateHz: number;
  // enabledTypes: 収集するプロファイル種別のスライス
  readonly enabledTypes: readonly ProfileType[];
  // labels: 全プロファイルに付与する共通ラベル（tenant_id / service 等）
  readonly labels?: Readonly<Record<string, string>> | undefined;
}

/**
 * ContinuousProfiler はバックグラウンドで常時プロファイルを収集する L2* interface を宣言する。
 * Pyroscope Agent 等の常時プロファイリング OSS を Library 独自語彙で抽象化する。
 */
// ContinuousProfiler インターフェース定義
export interface ContinuousProfiler {
  /**
   * start は設定に従ってバックグラウンドのプロファイル収集を開始する。
   * 返された AbortController の signal でプロファイル収集を停止する。
   */
  // start メソッド: バックグラウンドプロファイル収集を開始する
  start(signal: AbortSignal): Promise<void>;

  /**
   * stop はバックグラウンドのプロファイル収集を停止する（グレースフルシャットダウン）。
   */
  // stop メソッド: バックグラウンドプロファイル収集を停止する
  stop(): Promise<void>;
}
