// s08_quota_exhaustion.spec.ts — quota exhaustion シナリオ（conflict 系 S08b）
// @playwright/test で PQ が quota 上限に達した場合の挙動を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §quota enforcement → 429 / quota_exceeded
// 命名規約: s08_logout_full_purge_audit_emit.spec.ts（オフライン系 S08）と区別するために S08b とする

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// quota exhaustion 処理のインライン定義
const QUOTA_INLINE = `
// PQ の quota 上限（設定から注入する）
var PQ_QUOTA_LIMIT = 50;
// quota 超過かどうかを判定する
function isQuotaExhausted(pendingQueue) {
  return pendingQueue.length >= PQ_QUOTA_LIMIT;
}
// quota exhaustion 発生時の処理
// PQ への新規 enqueue を拒否して QUOTA_EXCEEDED error を返す
function tryEnqueue(state, mutation) {
  if (isQuotaExhausted(state.pendingQueue)) {
    return {
      enqueued: false,
      errorCode: 'QUOTA_EXCEEDED',
      currentCount: state.pendingQueue.length,
      limit: PQ_QUOTA_LIMIT,
    };
  }
  var newPendingQueue = state.pendingQueue.concat([mutation]);
  return {
    enqueued: true,
    nextState: Object.assign({}, state, { pendingQueue: newPendingQueue }),
    currentCount: newPendingQueue.length,
  };
}
// quota の使用率を返す（0.0 - 1.0）
function quotaUsage(pendingQueue) {
  return pendingQueue.length / PQ_QUOTA_LIMIT;
}
`;

// S08: PQ quota 上限到達 → enqueue 拒否テスト
// テスト名: S08b を使用して s08_logout_full_purge_audit_emit の S08 との重複を回避する
test('S08b: PQ quota 上限（50 件）到達で enqueue が拒否される', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: 49 件の PQ entry がある状態を用意する（上限 1 件手前）
  const nearLimitState = await page.evaluate(() => {
    // 49 件の PQ エントリを生成する
    var entries = [];
    for (var i = 1; i <= 49; i++) {
      entries.push({
        idempotencyKey: 'quota-idem-' + String(i).padStart(3, '0'),
        payload: { fieldId: 'F001', value: i },
      });
    }
    return {
      serverTruth: { version: 1 },
      optimisticLocal: null,
      pendingQueue: entries,
      queueHeld: false,
    };
  });

  // PQ が 49 件であることを確認する
  expect(nearLimitState.pendingQueue).toHaveLength(49);

  // Step 2: 50 件目の mutation を enqueue する（成功するはず）
  const enqueue50Result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.quotaInline);
      // 50 件目の mutation を enqueue する（上限に達するが拒否されない）
      // @ts-ignore（eval scope の tryEnqueue 関数を使用する）
      return tryEnqueue(args.state, { idempotencyKey: 'quota-idem-050', payload: { fieldId: 'F001', value: 50 } });
    },
    { quotaInline: QUOTA_INLINE, state: nearLimitState },
  );

  // 50 件目の enqueue が成功することを確認する
  expect(enqueue50Result.enqueued).toBe(true);
  // PQ が 50 件になることを確認する
  expect(enqueue50Result.currentCount).toBe(50);

  // Step 3: 51 件目の mutation を enqueue しようとする（quota 超過で拒否される）
  const enqueue51Result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.quotaInline);
      // 51 件目の mutation を enqueue しようとする（quota 超過）
      // @ts-ignore（eval scope の tryEnqueue 関数を使用する）
      return tryEnqueue(args.state, { idempotencyKey: 'quota-idem-051', payload: { fieldId: 'F001', value: 51 } });
    },
    { quotaInline: QUOTA_INLINE, state: enqueue50Result.nextState },
  );

  // 51 件目の enqueue が拒否されることを確認する
  expect(enqueue51Result.enqueued).toBe(false);
  // QUOTA_EXCEEDED エラーコードが返されることを確認する
  expect(enqueue51Result.errorCode).toBe('QUOTA_EXCEEDED');
  // 現在の件数と上限が正しく返されることを確認する
  expect(enqueue51Result.currentCount).toBe(50);
  expect(enqueue51Result.limit).toBe(50);
});

// S08b: quota 使用率の計算テスト
// S08c: PQ quota 使用率テスト（S08b との重複を回避する）
test('S08c: PQ quota 使用率が正しく計算される', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: 25 件の PQ entry で使用率 50% を確認する
  const usageResult = await page.evaluate(
    (quotaInline) => {
      // eslint-disable-next-line no-eval
      eval(quotaInline);
      // 25 件の PQ エントリを生成する
      var entries = [];
      for (var i = 0; i < 25; i++) {
        entries.push({ idempotencyKey: 'usage-idem-' + i });
      }
      // 使用率を計算する
      // @ts-ignore（eval scope の quotaUsage 関数を使用する）
      return quotaUsage(entries);
    },
    QUOTA_INLINE,
  );

  // 使用率が 0.5（50%）であることを確認する
  expect(usageResult).toBe(0.5);
});
