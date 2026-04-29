// 本ファイルは k6 シナリオ共通の helper（環境変数 / auth header / endpoint 解決）。
//
// docs 正典:
//   docs/03_要件定義/30_非機能要件/B_性能.md（NFR-B-PERF-*）
//   docs/03_要件定義/30_非機能要件/A_可用性.md（NFR-A-SLA-*）
//   ops/runbooks/daily/error-code-alert-policy.md
//
// 環境変数:
//   K6_TARGET_BASE   ... 対象 BFF / tier1 facade の base URL（既定 http://localhost:50080）
//   K6_AUTH_TOKEN    ... `Authorization: Bearer <token>` の token（off mode なら "dev" でも可）
//   K6_TENANT_ID     ... 既定 tenant ID（実際の tenant は token claim から決まるが、
//                        off mode 用に request 識別子として残す）

// 環境変数取得 helper（k6 グローバル __ENV を経由）。未設定時は default を返す。
export function env(name, fallback) {
  const v = __ENV[name];
  if (v === undefined || v === '') {
    return fallback;
  }
  return v;
}

// 共通 endpoint 解決。
export function targetBase() {
  return env('K6_TARGET_BASE', 'http://localhost:50080');
}

// 共通 HTTP header（Authorization + Content-Type + traceparent placeholder）。
export function defaultHeaders() {
  return {
    'Content-Type': 'application/json; charset=utf-8',
    Authorization: `Bearer ${env('K6_AUTH_TOKEN', 'dev')}`,
  };
}

// 既定テナント ID（off mode で AuthClaims.TenantID が "demo-tenant" になる前提）。
export function defaultTenantID() {
  return env('K6_TENANT_ID', 'demo-tenant');
}

// k1s0 HTTP/JSON gateway の URL を組み立てる helper。
// docs §「HTTP/JSON 互換」: POST /k1s0/<api>/<rpc>。
export function k1s0URL(api, rpc) {
  return `${targetBase()}/k1s0/${api}/${rpc}`;
}

// テスト ID 用にユニーク文字列を返す（同 vu / iter で衝突回避）。
import exec from 'k6/execution';
export function uniqueKey(prefix) {
  return `${prefix}-${exec.vu.idInTest}-${exec.scenario.iterationInInstance}-${Date.now()}`;
}
