// 本ファイルは tier1 facade の RateLimit interceptor 動作確認シナリオ。
//
// docs 正典:
//   docs/03_要件定義/00_共通規約.md §「レート制限」
//   ops/runbooks/daily/error-code-alert-policy.md（K1s0Tier1RateLimitExceeded
//     アラートのトリガー条件と一致させる）
//
// シナリオ:
//   1 テナント / 1 VU から burst で連投し、TIER1_RATELIMIT_RPS / BURST 設定値を
//   超えた所で 429 が返ることを確認する。dev 環境では既定 RPS=100 / BURST=200 だが、
//   本シナリオでは Pod 起動時に env で RPS=2 / BURST=3 を上書きしている前提。
//
// 起動例:
//   # 別 shell で
//   TIER1_AUTH_MODE=off TIER1_RATELIMIT_RPS=2 TIER1_RATELIMIT_BURST=3 \
//     TIER1_HTTP_LISTEN_ADDR=:50080 ./t1-pii &
//
//   # k6 を流す
//   K6_TARGET_BASE=http://localhost:50080 \
//     k6 run ops/load/scenarios/rate_limit.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { k1s0URL, defaultHeaders } from '../k6/helpers/common.js';

export const options = {
  // 1 VU で 12 req を burst 投げて、最初の 3 件が 200、残りが 429 となることを確認する。
  vus: 1,
  iterations: 1,
  thresholds: {
    // 連投シナリオは 200 と 429 が混在するため http_req_failed は使わず checks を見る。
    'checks{phase:burst}': ['rate>=0.99'],
  },
};

export default function () {
  const headers = defaultHeaders();
  let success = 0;
  let throttled = 0;
  // burst 12 件を間隔ゼロで投げる。
  for (let i = 0; i < 12; i++) {
    const res = http.post(
      k1s0URL('pii', 'classify'),
      JSON.stringify({ text: 'load-test' }),
      { headers, tags: { phase: 'burst' } },
    );
    if (res.status === 200) success++;
    else if (res.status === 429) throttled++;
  }
  // RPS=2 / BURST=3 で 1 burst 即時送信した場合、3 件成功 → 9 件 throttle が期待。
  check(null, {
    'burst yields some successes': () => success >= 1,
    'burst yields some 429': () => throttled >= 1,
    'burst respects bucket size (≤ BURST+1 successes)': () => success <= 4,
  });
  // 1 秒待つと RPS=2 補充されるため、ペースを落として再取得が成功する。
  sleep(1.5);
  const recovery = http.post(
    k1s0URL('pii', 'classify'),
    JSON.stringify({ text: 'recovery' }),
    { headers, tags: { phase: 'recovery' } },
  );
  check(recovery, {
    'recovery 200 after refill': (r) => r.status === 200,
  });
}
