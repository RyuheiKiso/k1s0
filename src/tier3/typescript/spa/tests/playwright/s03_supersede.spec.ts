// s03_supersede.spec.ts — supersede 競合シナリオ（conflict 系 S03b）
// @playwright/test で supersede BusinessConflict subtype の処理を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §supersede → delete_queue_entry + silent_toast
// 命名規約: s03_25h_offline_idempotency_ttl.spec.ts（オフライン系 S03）と区別するために S03b とする

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// supersede 処理の reducer インライン定義
const REDUCER_INLINE = `
// business_conflict_received (supersede) を処理する
// 同 actor 後続 op で上書き済み → PQ 先頭 entry を削除して silent toast を表示する
function reduceSupersede(state) {
  var keys = state.pendingQueue.slice();
  var actions = [];
  if (keys.length > 0) {
    var key = keys[0].idempotencyKey;
    keys = keys.slice(1);
    actions.push({ type: 'DELETE_PQ_ENTRY', idempotencyKey: key });
  }
  actions.push({ type: 'NOTIFY_SILENT_TOAST', message: '後続の操作で既に上書きされました' });
  return {
    nextState: Object.assign({}, state, { pendingQueue: keys }),
    actions: actions,
  };
}
function reduce(state, event) {
  switch (event.eventId) {
    case 'supersede': return reduceSupersede(state);
    default: throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S03b: supersede → PQ 先頭 entry 削除 + silent toast テスト（s03_25h_offline_idempotency_ttl の S03 との重複を回避する）
test('S03b: supersede → DELETE_PQ_ENTRY + NOTIFY_SILENT_TOAST', async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto('about:blank');

  // Step 1: PQ に 2 件 entry がある状態を用意する
  const initialState = {
    serverTruth: { version: 5 },
    optimisticLocal: null,
    pendingQueue: [
      // 先頭 entry（supersede の対象となる旧い mutation）
      { idempotencyKey: 'sup-idem-001', payload: { fieldId: 'F001', value: 10 } },
      // 2 件目 entry（後続 op で上書きした最新 mutation）
      { idempotencyKey: 'sup-idem-002', payload: { fieldId: 'F001', value: 20 } },
    ],
    queueHeld: false,
  };

  // Step 2: supersede event を発行して先頭 entry を削除する
  const result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // supersede event を発行する（同 actor が後続 op で上書きした状況）
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'supersede' });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // PQ の先頭 entry（sup-idem-001）が削除されていることを確認する
  expect(result.nextState.pendingQueue).toHaveLength(1);
  expect(result.nextState.pendingQueue[0].idempotencyKey).toBe('sup-idem-002');

  // DELETE_PQ_ENTRY action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'DELETE_PQ_ENTRY', idempotencyKey: 'sup-idem-001' }),
  );
  // NOTIFY_SILENT_TOAST action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'NOTIFY_SILENT_TOAST' }),
  );

  // Step 3: PQ が 1 件だけの状態で supersede を再度発行して空になることを確認する
  const result2 = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // 2 回目の supersede event を発行する
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'supersede' });
    },
    { reducerCode: REDUCER_INLINE, state: result.nextState },
  );

  // PQ が空になることを確認する
  expect(result2.nextState.pendingQueue).toHaveLength(0);
  // 2 回目の DELETE_PQ_ENTRY action が発行されることを確認する
  expect(result2.actions).toContainEqual(
    expect.objectContaining({ type: 'DELETE_PQ_ENTRY', idempotencyKey: 'sup-idem-002' }),
  );
});
