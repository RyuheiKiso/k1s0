// k1s0 tier3 OpenTelemetry Browser SDK 初期化（設計方針 18: OTel Browser SDK）
// Basic span / resource attributes / W3C Trace-Context propagation を設定する
// 新しい npm パッケージ追加禁止のため、fetch に monkey-patch するシンプルな実装を使用する
// 本格的な @opentelemetry/sdk-web は本番実装時に追加する

// トレースプロバイダー設定
export interface OtelBrowserConfig {
  // サービス名（resource attribute として設定する）
  readonly serviceName: string;
  // サービスバージョン（resource attribute として設定する）
  readonly serviceVersion: string;
  // OTel Collector エンドポイント URL（BFF 経由で送信する）
  readonly collectorUrl: string;
  // テナント slug（resource attribute として設定する）
  readonly tenantSlug: string;
  // 環境名（production / staging / development）
  readonly environment: string;
  // サンプリングレート（0.0 ～ 1.0）
  readonly samplingRate: number;
}

// span の型（W3C Trace-Context 準拠）
export interface SpanContext {
  // trace ID（128 bit hex string）
  readonly traceId: string;
  // span ID（64 bit hex string）
  readonly spanId: string;
  // parent span ID（string = 親スパン付き / undefined = ルートスパン）
  // exactOptionalPropertyTypes: true のため undefined を明示的に許可する
  readonly parentSpanId: string | undefined;
  // W3C Trace-Context traceparent ヘッダー値
  readonly traceparent: string;
  // span 名
  readonly spanName: string;
  // 開始時刻（Date.now() ミリ秒）
  readonly startTimeMs: number;
}

// trace ID を生成する（16 バイト hex string = 128 bit）
export function generateTraceId(): string {
  // crypto.getRandomValues を使って 16 バイトのランダムバイト列を生成する
  const bytes = new Uint8Array(16);
  // crypto.getRandomValues でランダムバイト列を埋める
  crypto.getRandomValues(bytes);
  // hex string に変換して返す
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

// span ID を生成する（8 バイト hex string = 64 bit）
export function generateSpanId(): string {
  // crypto.getRandomValues を使って 8 バイトのランダムバイト列を生成する
  const bytes = new Uint8Array(8);
  // crypto.getRandomValues でランダムバイト列を埋める
  crypto.getRandomValues(bytes);
  // hex string に変換して返す
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

// W3C traceparent ヘッダー値を生成する（version=00 固定）
// format: 00-{traceId}-{spanId}-01（sampled フラグ）
export function buildTraceparent(traceId: string, spanId: string): string {
  // traceparent ヘッダーを生成する（W3C Trace-Context spec 準拠）
  return `00-${traceId}-${spanId}-01`;
}

// ルートスパンを開始する
export function startRootSpan(spanName: string): SpanContext {
  // trace ID を生成する
  const traceId = generateTraceId();
  // span ID を生成する
  const spanId = generateSpanId();
  // traceparent ヘッダーを生成する
  const traceparent = buildTraceparent(traceId, spanId);
  // スパンコンテキストを返す
  return {
    // trace ID を設定する
    traceId,
    // span ID を設定する
    spanId,
    // parent span ID は undefined（ルートスパン）
    parentSpanId: undefined,
    // traceparent ヘッダーを設定する
    traceparent,
    // span 名を設定する
    spanName,
    // 開始時刻を設定する（HLC の代わりに Date.now() を span 時刻として使用する）
    startTimeMs: Date.now(),
  };
}

// 子スパンを開始する（親スパンのコンテキストを継承する）
export function startChildSpan(parentSpan: SpanContext, spanName: string): SpanContext {
  // 子スパンの span ID を生成する
  const spanId = generateSpanId();
  // traceparent ヘッダーを親の trace ID + 新しい span ID で生成する
  const traceparent = buildTraceparent(parentSpan.traceId, spanId);
  // 子スパンコンテキストを返す
  return {
    // 親の trace ID を継承する
    traceId: parentSpan.traceId,
    // 新しい span ID を設定する
    spanId,
    // 親の span ID を設定する
    parentSpanId: parentSpan.spanId,
    // traceparent ヘッダーを設定する
    traceparent,
    // span 名を設定する
    spanName,
    // 開始時刻を設定する
    startTimeMs: Date.now(),
  };
}

// OTel Browser SDK の初期化状態を保持するシングルトン
let _currentConfig: OtelBrowserConfig | null = null;
// 現在のルートスパン（ページロード時に生成する）
let _rootSpan: SpanContext | null = null;

// OTel Browser SDK を初期化する
// production/development 分岐禁止規約: 環境によって動作が変わるデバッグ出力は削除する
export function initOtelBrowser(config: OtelBrowserConfig): SpanContext {
  // 設定を保持する
  _currentConfig = config;
  // ページロード用のルートスパンを生成する
  _rootSpan = startRootSpan("pageload");
  // ルートスパンを返す（console.debug による環境別出力は禁止規約に従い削除済み）
  return _rootSpan;
}

// 現在の OTel 設定を取得する（未初期化の場合は null）
export function getCurrentOtelConfig(): OtelBrowserConfig | null {
  // 現在の設定を返す
  return _currentConfig;
}

// 現在のルートスパンを取得する（未初期化の場合は null）
export function getRootSpan(): SpanContext | null {
  // 現在のルートスパンを返す
  return _rootSpan;
}

// traceparent ヘッダーを現在のルートスパンから生成する
// fetch リクエストの headers に追加して W3C Trace-Context を伝播する
export function getTraceparentHeader(): string | null {
  // ルートスパンが存在しない場合は null を返す
  if (_rootSpan === null) return null;
  // traceparent ヘッダーを返す
  return _rootSpan.traceparent;
}

// span を終了して OTel Collector にエクスポートする（非同期）
// 実際の collector 送信は fetch を使って行う
export function endSpan(
  span: SpanContext,
  attributes?: Record<string, string | number | boolean>,
): void {
  // 設定が初期化されていない場合は何もしない
  if (_currentConfig === null) return;
  // サンプリングレートに応じてエクスポートをスキップする
  if (Math.random() > _currentConfig.samplingRate) return;
  // span 終了時刻を計算する
  const endTimeMs = Date.now();
  // collector へのエクスポートは fire-and-forget で実行する（エラーは無視する）
  // production/development 分岐禁止規約: console.debug による環境別出力は削除済み
  void exportSpanToCollector(_currentConfig, span, endTimeMs, attributes);
}

// OTel Collector に span をエクスポートする（OTLP HTTP/JSON 形式）
async function exportSpanToCollector(
  config: OtelBrowserConfig,
  span: SpanContext,
  endTimeMs: number,
  attributes?: Record<string, string | number | boolean>,
): Promise<void> {
  // OTLP HTTP/JSON 形式の span データを構築する
  const spanData = {
    // リソース attributes
    resource: {
      // service.name
      "service.name": config.serviceName,
      // service.version
      "service.version": config.serviceVersion,
      // tenant.slug
      "tenant.slug": config.tenantSlug,
      // deployment.environment
      "deployment.environment": config.environment,
    },
    // スパンデータ
    span: {
      // trace ID
      traceId: span.traceId,
      // span ID
      spanId: span.spanId,
      // parent span ID（undefined の場合は省略）
      ...(span.parentSpanId !== undefined ? { parentSpanId: span.parentSpanId } : {}),
      // span 名
      name: span.spanName,
      // 開始時刻（ミリ秒）
      startTimeMs: span.startTimeMs,
      // 終了時刻（ミリ秒）
      endTimeMs,
      // 持続時間（ミリ秒）
      durationMs: endTimeMs - span.startTimeMs,
      // 追加 attributes
      attributes: attributes ?? {},
    },
  };
  // fetch で collector にエクスポートする（エラーは呼び出し元が無視する）
  await fetch(config.collectorUrl, {
    // POST でエクスポートする
    method: "POST",
    // OTLP JSON 形式で送信する
    headers: { "Content-Type": "application/json" },
    // span データを JSON シリアライズして送信する
    body: JSON.stringify(spanData),
    // credentials を omit にして cross-origin cookie を送信しない
    credentials: "omit",
  });
}
