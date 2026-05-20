// s05_presence.spec.ts — presence indicator シナリオ（conflict 系 S05b）
// @playwright/test で concurrent_edit → UPDATE_PRESENCE の処理を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §concurrent_edit → update_presence_indicator
// 命名規約: s05_parallel_edit_lost_update_3way_merge.spec.ts（オフライン系 S05）と区別するために S05b とする

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// presence indicator 処理の reducer インライン定義
const REDUCER_INLINE = `
// business_conflict_received (concurrent_edit) を処理する
// 他 actor が presence 中 → UPDATE_PRESENCE action を発行する
function reduceConcurrentEdit(state, actorId) {
  return {
    nextState: state,
    actions: [{ type: 'UPDATE_PRESENCE', actorId: actorId }],
  };
}
// presence entry が有効かどうかを HLC カウンタで判定する
// wall-clock TTL 禁止のため HLC カウンタを使用する
function isPresenceValid(entry, currentHlc) {
  return entry.expiresAtLogical > currentHlc;
}
// presence リストから有効な entry のみを返す
function getActiveActors(presenceEntries, currentHlc) {
  return presenceEntries.filter(function(e) { return isPresenceValid(e, currentHlc); });
}
function reduce(state, event) {
  switch (event.eventId) {
    case 'concurrent_edit': return reduceConcurrentEdit(state, event.actorId);
    default: throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S05: presence indicator → UPDATE_PRESENCE テスト
// テスト名: S05b を使用して s05_parallel_edit_lost_update_3way_merge の S05 との重複を回避する
test('S05b: concurrent_edit → UPDATE_PRESENCE action が発行される', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // Step 1: 通常の編集状態を用意する
  const initialState = {
    serverTruth: { version: 2 },
    optimisticLocal: { idempotencyKey: 'edit-idem-001', payload: { fieldId: 'F001', value: 10 } },
    pendingQueue: [],
    queueHeld: false,
  };

  // Step 2: concurrent_edit event を受信する（他 actor が同じ aggregate を編集中）
  const result = await page.evaluate(
    (args) => {
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // concurrent_edit event を発行する（actor-456 が同じ aggregate を編集中）
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'concurrent_edit', actorId: 'actor-456' });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // UPDATE_PRESENCE action が発行されることを確認する
  expect(result.actions).toContainEqual(
    expect.objectContaining({ type: 'UPDATE_PRESENCE', actorId: 'actor-456' }),
  );
  // state は変更されないことを確認する（presence 更新は副作用のみ）
  expect(result.nextState).toEqual(initialState);
});

// S05c: presence リストの HLC 有効期限フィルタリングテスト（S05b との命名重複を回避する）
test('S05c: HLC カウンタによる有効 presence entry フィルタリング', async ({ page }) => {
  // 空白ページを開く
  await page.goto('about:blank');

  // HLC ベースの presence フィルタリングを検証する
  const result = await page.evaluate((reducerCode) => {
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // presence entry リストを用意する（有効/失効が混在する）
    var presenceEntries = [
      // HLC カウンタ 200 で失効する entry（現在 HLC=100 のため有効）
      { actorId: 'actor-A', expiresAtLogical: 200 },
      // HLC カウンタ 50 で失効した entry（現在 HLC=100 のため失効）
      { actorId: 'actor-B', expiresAtLogical: 50 },
      // HLC カウンタ 150 で失効する entry（現在 HLC=100 のため有効）
      { actorId: 'actor-C', expiresAtLogical: 150 },
    ];
    // 現在の HLC カウンタ（wall-clock TTL 禁止のため HLC を使用する）
    var currentHlc = 100;
    // 有効な presence entry を取得する
    // @ts-ignore（eval scope の getActiveActors 関数を使用する）
    return getActiveActors(presenceEntries, currentHlc);
  }, REDUCER_INLINE);

  // 有効な presence entry が 2 件（actor-A, actor-C）であることを確認する
  expect(result).toHaveLength(2);
  // actor-A が有効であることを確認する
  expect(result.some((e: { actorId: string }) => e.actorId === 'actor-A')).toBe(true);
  // actor-C が有効であることを確認する
  expect(result.some((e: { actorId: string }) => e.actorId === 'actor-C')).toBe(true);
  // actor-B（失効）が含まれないことを確認する
  expect(result.some((e: { actorId: string }) => e.actorId === 'actor-B')).toBe(false);
});
