// s04_chained_resend.spec.ts — auto_resend_with_chained_key シナリオ
// @playwright/test で stale_write (disjoint) → chain idempotency key → auto resend を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §stale_write + FieldDiff.IsDisjoint → auto resend

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// chained resend 処理の reducer インライン定義
const REDUCER_INLINE = `
// FieldDiff が disjoint かどうかを判定する
function isDisjoint(clientFields, serverFields) {
  for (var i = 0; i < serverFields.length; i++) {
    if (clientFields.indexOf(serverFields[i]) !== -1) return false;
  }
  return true;
}
// ChainIdempotencyKey: base key から chain された新しい key を生成する
function chainIdempotencyKey(baseKey, next) {
  var randomSuffix = Math.random().toString(36).slice(2, 18);
  var basePrefix = baseKey.length > 12 ? baseKey.slice(0, 12) : baseKey;
  var nextPrefix = next.length > 8 ? next.slice(0, 8) : next;
  return basePrefix + '_' + nextPrefix + '_' + randomSuffix;
}
// stale_write (disjoint) の reducer
function reduceStaleWriteDisjoint(state, fieldDiff) {
  var baseKey = state.optimisticLocal ? state.optimisticLocal.idempotencyKey : 'unknown';
  var newKey = chainIdempotencyKey(baseKey, 'rebase');
  return {
    nextState: Object.assign({}, state, { optimisticLocal: null }),
    actions: [
      { type: 'ROLLBACK_OPTIMISTIC' },
      { type: 'SEND_QUEUE_IN_ORDER' },
      { type: 'NOTIFY_SILENT_TOAST', message: 'rebase_clean: auto resend with key=' + newKey },
    ],
    chainedKey: newKey,
  };
}
// stale_write (intersecting) の reducer
function reduceStaleWriteIntersecting(state) {
  return {
    nextState: Object.assign({}, state, { queueHeld: true }),
    actions: [
      { type: 'PRESENT_3WAY_MERGE_UI' },
      { type: 'HOLD_QUEUE' },
    ],
  };
}
function reduce(state, event) {
  if (event.eventId === 'stale_write') {
    if (event.fieldDiff && isDisjoint(event.fieldDiff.clientFields, event.fieldDiff.serverFields)) {
      return reduceStaleWriteDisjoint(state, event.fieldDiff);
    }
    return reduceStaleWriteIntersecting(state);
  }
  throw new Error('Unknown event: ' + event.eventId);
}
`;

// S04: stale_write (disjoint) → ROLLBACK + chained key auto resend テスト
test('S04: stale_write disjoint → ROLLBACK_OPTIMISTIC + SEND_QUEUE_IN_ORDER', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: OL in-flight 中の状態を用意する（F001 を変更した mutation）
  const initialState = {
    serverTruth: { version: 3 },
    optimisticLocal: {
      idempotencyKey: 'stale-idem-001',
      payload: { fieldId: 'F001', value: 50 },
    },
    pendingQueue: [
      // PQ に同 mutation を保持する
      { idempotencyKey: 'stale-idem-001', payload: { fieldId: 'F001', value: 50 } },
    ],
    queueHeld: false,
  };

  // Step 2: stale_write event を発行する（server が F002 を変更した FieldDiff が disjoint）
  const result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // stale_write event を発行する（clientFields: [F001], serverFields: [F002] = disjoint）
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, {
        eventId: 'stale_write',
        fieldDiff: {
          clientFields: ['F001'],
          serverFields: ['F002'],
        },
      });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // OL が null になることを確認する（rollback が発生した）
  expect(result.nextState.optimisticLocal).toBeNull();
  // ROLLBACK_OPTIMISTIC action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'ROLLBACK_OPTIMISTIC' }),
  );
  // SEND_QUEUE_IN_ORDER action が発行されることを確認する（auto resend）
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'SEND_QUEUE_IN_ORDER' }),
  );
  // NOTIFY_SILENT_TOAST action が発行されることを確認する
  const toastAction = result.actions.find((a: { type: string }) => a.type === 'NOTIFY_SILENT_TOAST');
  expect(toastAction).toBeDefined();
  // chained key が toast メッセージに含まれることを確認する
  expect((toastAction as { type: string; message: string }).message).toContain('rebase_clean');
  // chainedKey が base prefix + rebase + random から生成されていることを確認する
  expect(result.chainedKey).toMatch(/^stale-idem-_rebase_/);
});

// S04b: stale_write (intersecting) → 3way merge UI + hold テスト
test('S04b: stale_write intersecting → PRESENT_3WAY_MERGE_UI + HOLD_QUEUE', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: OL in-flight 中の状態を用意する
  const initialState = {
    serverTruth: { version: 3 },
    optimisticLocal: { idempotencyKey: 'stale-idem-002', payload: { fieldId: 'F001', value: 50 } },
    pendingQueue: [],
    queueHeld: false,
  };

  // Step 2: stale_write (intersecting) event を発行する
  const result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // stale_write event を発行する（clientFields: [F001], serverFields: [F001] = intersecting）
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, {
        eventId: 'stale_write',
        fieldDiff: {
          clientFields: ['F001'],
          serverFields: ['F001'],
        },
      });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // PRESENT_3WAY_MERGE_UI action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'PRESENT_3WAY_MERGE_UI' }),
  );
  // HOLD_QUEUE action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'HOLD_QUEUE' }),
  );
  // queueHeld が true になることを確認する
  expect(result.nextState.queueHeld).toBe(true);
});
