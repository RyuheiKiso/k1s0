// 本ファイルは tier1 State API の baseline 性能シナリオ。
//
// docs 正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/02_State_API.md
//   docs/03_要件定義/30_非機能要件/B_性能.md（NFR-B-PERF-001: tier1 内部 API は
//     p99 latency < 500ms）
//
// シナリオ:
//   1 VU が Save → Get → Delete の round-trip を 60 秒間ループする。
//   p95 / p99 を threshold で監視し、tier1 SLO（NFR-A-SLA-001）違反時は fail。
//
// 起動例:
//   K6_TARGET_BASE=http://t1-state:50080 K6_AUTH_TOKEN=$JWT \
//     k6 run ops/load/scenarios/state_baseline.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import encoding from 'k6/encoding';
import { k1s0URL, defaultHeaders, defaultTenantID, uniqueKey } from '../k6/helpers/common.js';

// k6 オプション: 1 VU で 60 秒、p95/p99 threshold を docs SLO に揃える。
export const options = {
  vus: 1,
  duration: '60s',
  thresholds: {
    // NFR-B-PERF-001: p99 < 500ms。p95 はその半分 250ms に設定。
    'http_req_duration{step:save}': ['p(95)<250', 'p(99)<500'],
    'http_req_duration{step:get}':  ['p(95)<150', 'p(99)<300'],
    // 失敗率 1% 以下（NFR-A-SLA-001 の error budget 準拠）。
    'http_req_failed': ['rate<0.01'],
  },
};

// ペイロードを base64 化する helper（protojson の bytes 仕様）。
function b64(s) {
  return encoding.b64encode(s);
}

export default function () {
  const tenantId = defaultTenantID();
  const key = uniqueKey('state-baseline');
  const headers = defaultHeaders();
  const ctx = { tenantId, subject: 'k6' };

  // --- Save（特権 RPC、auth header 必須） ---
  const saveRes = http.post(
    k1s0URL('state', 'set'),
    JSON.stringify({
      store: 'kvstore',
      key,
      data: b64('hello world'),
      ttlSec: 60,
      context: ctx,
    }),
    { headers, tags: { step: 'save' } },
  );
  check(saveRes, {
    'save 200': (r) => r.status === 200,
    'save returns newEtag': (r) => {
      try {
        return Boolean(JSON.parse(r.body).newEtag);
      } catch {
        return false;
      }
    },
  });

  // --- Get ---
  const getRes = http.post(
    k1s0URL('state', 'get'),
    JSON.stringify({ store: 'kvstore', key, context: ctx }),
    { headers, tags: { step: 'get' } },
  );
  check(getRes, {
    'get 200': (r) => r.status === 200,
    'get found': (r) => {
      try {
        return JSON.parse(r.body).notFound !== true;
      } catch {
        return false;
      }
    },
  });

  // --- Delete ---
  http.post(
    k1s0URL('state', 'delete'),
    JSON.stringify({ store: 'kvstore', key, context: ctx }),
    { headers, tags: { step: 'delete' } },
  );

  // 1 iteration / sec のペースを保つ（baseline はバースト目的ではない）。
  sleep(1);
}
