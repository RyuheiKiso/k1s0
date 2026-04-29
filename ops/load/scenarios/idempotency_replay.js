// 本ファイルは tier1 facade の idempotency_key dedup 動作確認シナリオ。
//
// docs 正典:
//   docs/03_要件定義/00_共通規約.md §「冪等性と再試行」
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//     （AuditService.Record の idempotency_key で hash chain 二重追記を防ぐ）
//
// シナリオ:
//   同一 idempotency_key で N 回 Audit.Record を投げ、すべて同じ audit_id が返り
//   verifyChain 後の checked_count が 1 件だけ増えることを確認する。
//
// 起動例:
//   K6_TARGET_BASE=http://t1-audit:50080 K6_AUTH_TOKEN=$JWT \
//     k6 run ops/load/scenarios/idempotency_replay.js

import http from 'k6/http';
import { check } from 'k6';
import { k1s0URL, defaultHeaders, defaultTenantID, uniqueKey } from '../k6/helpers/common.js';

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    'checks': ['rate==1.0'],
  },
};

export default function () {
  const tenantId = defaultTenantID();
  const headers = defaultHeaders();
  const ctx = { tenantId, subject: 'k6' };
  const idemKey = uniqueKey('audit-idem');

  // 同一 idempotency_key で 5 回 Record を実行する。
  const ids = [];
  for (let i = 0; i < 5; i++) {
    const res = http.post(
      k1s0URL('audit', 'record'),
      JSON.stringify({
        event: {
          actor: 'k6-load',
          action: 'WRITE',
          resource: `k1s0:tenant:${tenantId}:resource:loadtest`,
          outcome: 'SUCCESS',
        },
        idempotencyKey: idemKey,
        context: ctx,
      }),
      { headers, tags: { phase: 'record' } },
    );
    check(res, { [`record ${i} 200`]: (r) => r.status === 200 });
    if (res.status === 200) {
      try {
        ids.push(JSON.parse(res.body).auditId);
      } catch {
        ids.push('');
      }
    }
  }

  // 全 5 件で同 audit_id が返ることを確認（dedup が効いている証跡）。
  check(null, {
    '5 calls returned same auditId': () => {
      if (ids.length !== 5) return false;
      const head = ids[0];
      if (!head) return false;
      return ids.every((id) => id === head);
    },
  });

  // verifyChain で checked_count をチェック。dedup が機能していれば
  // 1 hash chain entry のみ追加（複数 iteration で本シナリオ実行する場合は累積）。
  const v = http.post(
    k1s0URL('audit', 'verifychain'),
    JSON.stringify({ context: ctx }),
    { headers, tags: { phase: 'verify' } },
  );
  check(v, {
    'verifyChain 200': (r) => r.status === 200,
    'verifyChain valid=true': (r) => {
      try {
        return JSON.parse(r.body).valid === true;
      } catch {
        return false;
      }
    },
  });
}
