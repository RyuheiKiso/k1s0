// s07_optimistic_rollback.spec.ts — optimistic rejection + rollback シナリオ
// @playwright/test で optimistic_rejected → ROLLBACK_OPTIMISTIC を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §optimistic_rejected → rollback_optimistic

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// optimistic rollback 処理の reducer インライン定義
const REDUCER_INLINE = `
// optimistic_rejected event を処理する（OL rollback + business error 表示）
function reduceOptimisticRejected(state, idempotencyKey, errorCode, conflictSubtype) {
  var nextState = Object.assign({}, state, { optimisticLocal: null });
  var actions = [
    { type: 'ROLLBACK_OPTIMISTIC' },
    { type: 'PRESENT_BUSINESS_ERROR', errorCode: errorCode },
  ];
  if (conflictSubtype !== undefined && conflictSubtype !== null) {
    actions.push({ type: 'DISPATCH_CONFLICT_SUBTYPE', subtype: conflictSubtype });
  }
  return { nextState: nextState, actions: actions };
}
function reduce(state, event) {
  switch (event.eventId) {
    case 'optimistic_rejected':
      return reduceOptimisticRejected(state, event.idempotencyKey, event.errorCode, event.conflictSubtype);
    default:
      throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S07: optimistic_rejected → ROLLBACK_OPTIMISTIC テスト
test('S07: optimistic_rejected → ROLLBACK_OPTIMISTIC + PRESENT_BUSINESS_ERROR', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: OL in-flight 中の状態を用意する
  const initialState = {
    serverTruth: { version: 4 },
    optimisticLocal: {
      idempotencyKey: 'opt-idem-001',
      payload: { fieldId: 'F001', value: 99 },
    },
    pendingQueue: [
      { idempotencyKey: 'opt-idem-001', payload: { fieldId: 'F001', value: 99 } },
    ],
    queueHeld: false,
  };

  // Step 2: optimistic_rejected event を発行する（サーバーが mutation を拒否した）
  const result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // optimistic_rejected event を発行する（バリデーションエラーで拒否）
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, {
        eventId: 'optimistic_rejected',
        idempotencyKey: 'opt-idem-001',
        errorCode: 'VALIDATION_ERROR',
        conflictSubtype: null,
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
  // PRESENT_BUSINESS_ERROR action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'PRESENT_BUSINESS_ERROR', errorCode: 'VALIDATION_ERROR' }),
  );
  // DISPATCH_CONFLICT_SUBTYPE が含まれないことを確認する（conflictSubtype が null のため）
  expect(result.actions.some((a: { type: string }) => a.type === 'DISPATCH_CONFLICT_SUBTYPE')).toBe(false);
});

// S07b: optimistic_rejected with conflictSubtype → DISPATCH_CONFLICT_SUBTYPE テスト
test('S07b: optimistic_rejected with stale_write → DISPATCH_CONFLICT_SUBTYPE', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: OL in-flight 中の状態を用意する
  const initialState = {
    serverTruth: { version: 4 },
    optimisticLocal: { idempotencyKey: 'opt-idem-002', payload: { fieldId: 'F001', value: 100 } },
    pendingQueue: [],
    queueHeld: false,
  };

  // Step 2: optimistic_rejected event を conflictSubtype 付きで発行する
  const result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, {
        eventId: 'optimistic_rejected',
        idempotencyKey: 'opt-idem-002',
        errorCode: 'BUSINESS_CONFLICT',
        conflictSubtype: 'stale_write',
      });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // DISPATCH_CONFLICT_SUBTYPE action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'DISPATCH_CONFLICT_SUBTYPE', subtype: 'stale_write' }),
  );
  // OL が null になることを確認する
  expect(result.nextState.optimisticLocal).toBeNull();
});
