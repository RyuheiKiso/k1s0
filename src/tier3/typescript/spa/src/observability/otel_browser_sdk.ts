// otel_browser_sdk.ts — OTel Browser SDK 本実装
// @opentelemetry/api の trace/context/propagation を使った完全な観測可能性実装
// tier3_ext SemConv (tier3.screen.id / tier3.interaction.kind / tier3.web_vitals.* / tier3.layer)

// @opentelemetry/api の trace モジュールをインポートする
import { trace } from '@opentelemetry/api';

// tier3_ext SemConv 属性定数 (docs/04_詳細設計/01_適合仕様/03_観測適合仕様.md §tier3_ext)
export const TIER3_SEMCONV = {
  // 画面識別子
  SCREEN_ID: 'tier3.screen.id',
  // インタラクション種別
  INTERACTION_KIND: 'tier3.interaction.kind',
  // Web Vitals LCP（milliseconds）
  WEB_VITALS_LCP: 'tier3.web_vitals.lcp_ms',
  // Web Vitals CLS（unitless ratio）
  WEB_VITALS_CLS: 'tier3.web_vitals.cls',
  // Web Vitals INP（milliseconds）
  WEB_VITALS_INP: 'tier3.web_vitals.inp_ms',
  // tier3 layer 識別子 (ST/OL/PQ/DR)
  LAYER: 'tier3.layer',
} as const;

// tracer インスタンスを k1s0.tier3 スコープで取得する
const tracer = trace.getTracer('k1s0.tier3', '1.0.0');

// スクリーン操作のスパンを開始する
export function startScreenSpan(screenId: string, interactionKind: string) {
  // tracer.startSpan でスクリーンインタラクションスパンを開始する
  return tracer.startSpan('tier3.screen.interaction', {
    attributes: {
      // 画面識別子を属性に追加する
      [TIER3_SEMCONV.SCREEN_ID]: screenId,
      // インタラクション種別を属性に追加する
      [TIER3_SEMCONV.INTERACTION_KIND]: interactionKind,
    },
  });
}

// Web Vitals を OTel span として記録する (LCP/CLS/INP)
export function recordWebVitals(lcp: number, cls: number, inp: number): void {
  // tracer.startActiveSpan でアクティブスパンとして Web Vitals を記録する
  tracer.startActiveSpan('tier3.web_vitals', (span) => {
    // LCP を属性に追加する
    span.setAttribute(TIER3_SEMCONV.WEB_VITALS_LCP, lcp);
    // CLS を属性に追加する
    span.setAttribute(TIER3_SEMCONV.WEB_VITALS_CLS, cls);
    // INP を属性に追加する
    span.setAttribute(TIER3_SEMCONV.WEB_VITALS_INP, inp);
    // スパンを終了する
    span.end();
  });
}

// W3C Trace-Context ヘッダから trace_id を取得する (cross-tier propagation)
export function getTraceId(): string {
  // active span から spanContext を取得する
  const span = trace.getActiveSpan();
  // active span が存在しない場合は空文字を返す
  if (!span) return '';
  // span context から trace_id (16 byte hex) を取得して返す
  return span.spanContext().traceId;
}
