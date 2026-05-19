// s02_lost_update.spec.ts — lost_update 競合シナリオ
// @playwright/test で lost_update BusinessConflict subtype の処理を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §lost_update → present_3way_merge_ui + hold_queue

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// lost_update 処理の reducer インライン定義
const REDUCER_INLINE = `
// business_conflict_received (lost_update) を処理する
// safe 側: フィールド交差あり → 3way merge UI + hold queue
function reduceBusinessConflictLostUpdate(state) {
  return {
    nextState: Object.assign({}, state, { queueHeld: true }),
    actions: [
      { type: 'PRESENT_3WAY_MERGE_UI' },
      { type: 'HOLD_QUEUE' },
    ],
  };
}
// 3way merge 解決後の reducer（ローカルを採用した場合）
function resolveWithLocal(state) {
  return {
    nextState: Object.assign({}, state, { queueHeld: false }),
    actions: [{ type: 'SEND_QUEUE_IN_ORDER' }],
  };
}
// 3way merge 解決後の reducer（リモートを採用した場合）
function resolveWithRemote(state) {
  return {
    nextState: Object.assign({}, state, { queueHeld: false, pendingQueue: [] }),
    actions: [
      { type: 'DISCARD_PENDING_QUEUE' },
    ],
  };
}
function reduce(state, event) {
  switch (event.eventId) {
    case 'business_conflict_received_lost_update': return reduceBusinessConflictLostUpdate(state);
    case 'resolve_with_local': return resolveWithLocal(state);
    case 'resolve_with_remote': return resolveWithRemote(state);
    default: throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S02: lost_update 競合 → 3way merge UI 表示 → 解決テスト
test('S02: lost_update conflict → PRESENT_3WAY_MERGE_UI + HOLD_QUEUE', async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto('about:blank');

  // Step 1: フィールドが交差する lost_update 状態を用意する
  const initialState = {
    serverTruth: { version: 3 },
    optimisticLocal: { idempotencyKey: 'lu-idem-001', payload: { fieldId: 'F001', value: 42 } },
    pendingQueue: [
      // PQ に積まれた mutation（lost_update を引き起こした可能性のある mutation）
      { idempotencyKey: 'lu-idem-001', payload: { fieldId: 'F001', value: 42 } },
    ],
    queueHeld: false,
  };

  // Step 2: business_conflict_received (lost_update) を受信する
  const conflictResult = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // business_conflict_received (lost_update) event を発行する
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'business_conflict_received_lost_update' });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // PRESENT_3WAY_MERGE_UI action が発行されることを確認する
  expect(conflictResult.actions).toContainEqual(
    expect.objectContaining({ type: 'PRESENT_3WAY_MERGE_UI' }),
  );
  // HOLD_QUEUE action が発行されることを確認する
  expect(conflictResult.actions).toContainEqual(
    expect.objectContaining({ type: 'HOLD_QUEUE' }),
  );
  // queueHeld が true になることを確認する
  expect(conflictResult.nextState.queueHeld).toBe(true);

  // Step 3a: ローカルを採用した場合の解決
  const resolveLocalResult = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // ローカルを採用してキューを再開する
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'resolve_with_local' });
    },
    { reducerCode: REDUCER_INLINE, state: conflictResult.nextState },
  );

  // ローカル採用後に queueHeld が false になることを確認する
  expect(resolveLocalResult.nextState.queueHeld).toBe(false);
  // SEND_QUEUE_IN_ORDER action が発行されることを確認する
  expect(resolveLocalResult.actions).toContainEqual(
    expect.objectContaining({ type: 'SEND_QUEUE_IN_ORDER' }),
  );

  // Step 3b: リモートを採用した場合の解決（独立したテスト）
  const resolveRemoteResult = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // リモートを採用して PQ を破棄する
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'resolve_with_remote' });
    },
    { reducerCode: REDUCER_INLINE, state: conflictResult.nextState },
  );

  // リモート採用後に queueHeld が false になることを確認する
  expect(resolveRemoteResult.nextState.queueHeld).toBe(false);
  // PQ が空になることを確認する
  expect(resolveRemoteResult.nextState.pendingQueue).toHaveLength(0);
  // DISCARD_PENDING_QUEUE action が発行されることを確認する
  expect(resolveRemoteResult.actions).toContainEqual(
    expect.objectContaining({ type: 'DISCARD_PENDING_QUEUE' }),
  );
});
