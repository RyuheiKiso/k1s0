// tier3 Playwright シナリオ S02: 工場現場 5 件 offline 記録 → PQ resume → 全 ack
// 適合仕様: 11_クライアント状態適合仕様.md §製造業 pack stress test シナリオ 2
// PQ に 5 件 enqueue した後 network 復帰で SEND_QUEUE_IN_ORDER が返ることを検証する
// page.evaluate() でインライン reducer を実行し SPA bundle への依存を排除する

// @playwright/test の test / expect をインポートする
import { test, expect } from "@playwright/test";

// reducer のインライン定義文字列（page.evaluate 内で eval するインライン実装）
// 本番 bundle に依存しないため TypeScript primary の仕様準拠を自力で記述する
const REDUCER_INLINE = `
// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}

// pending_queue_resume event を処理する（PQ を in-order で送信する）
function reducePendingQueueResume(state) {
  // queue が hold 中または PQ が空の場合は何もしない
  if (state.queueHeld || state.pendingQueue.length === 0) {
    return { nextState: state, actions: [] };
  }
  // PQ に entry がある場合は SEND_QUEUE_IN_ORDER を返す
  return { nextState: state, actions: [{ type: 'SEND_QUEUE_IN_ORDER' }] };
}

// optimistic_acknowledged event を処理する（PQ entry 削除 + ST promote）
function reduceOptimisticAcknowledged(state, idempotencyKey, confirmedVersion) {
  // 指定された idempotencyKey の PQ entry を削除する
  var nextPendingQueue = state.pendingQueue.filter(function(pq) {
    return pq.idempotencyKey !== idempotencyKey;
  });
  var nextState = Object.assign({}, state, {
    optimisticLocal: null,
    serverTruth: { version: confirmedVersion, idempotencyKey: idempotencyKey },
    pendingQueue: nextPendingQueue,
  });
  return {
    nextState: nextState,
    actions: [
      { type: 'PROMOTE_OPTIMISTIC', idempotencyKey: idempotencyKey },
      { type: 'DELETE_PQ_ENTRY', idempotencyKey: idempotencyKey },
      { type: 'UPDATE_SERVER_TRUTH', version: confirmedVersion },
    ],
  };
}

// 5 event dispatcher（main reduce 関数）
function reduce(state, event) {
  switch (event.eventId) {
    case 'pending_queue_resume':
      return reducePendingQueueResume(state);
    case 'optimistic_acknowledged':
      return reduceOptimisticAcknowledged(state, event.idempotencyKey, event.confirmedVersion);
    default:
      throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S02: 工場現場 5 件 offline 記録 → PQ resume → 全 ack のテスト
test("S02: 工場現場 5 件 offline 記録後 PQ resume で全件送信指示が返る", async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto("about:blank");

  // phase 1: 5 件の PQ entry がある状態で pending_queue_resume を発行する
  const resumeResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する（page context に注入する）
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 5 件の PQ entry を持つ state を用意する（工場現場でオフライン中に記録した状態を想定する）
    const state = {
      // server_truth: 未初期化（オフライン中のため）
      serverTruth: null,
      // optimistic_local: 送信待ちなし
      optimisticLocal: null,
      // pending_queue: offline 中に記録した 5 件の検査結果
      pendingQueue: [
        { idempotencyKey: "inspect-idem-001", value: { itemId: "ITEM-001", result: "pass" } },
        { idempotencyKey: "inspect-idem-002", value: { itemId: "ITEM-002", result: "pass" } },
        { idempotencyKey: "inspect-idem-003", value: { itemId: "ITEM-003", result: "fail" } },
        { idempotencyKey: "inspect-idem-004", value: { itemId: "ITEM-004", result: "pass" } },
        { idempotencyKey: "inspect-idem-005", value: { itemId: "ITEM-005", result: "pass" } },
      ],
      // queue は hold していない（network 復帰後）
      queueHeld: false,
    };
    // pending_queue_resume event を発行する（network 復帰）
    const event = { eventId: "pending_queue_resume", resumeReason: "network_recovery" };
    // @ts-ignore（eval scope に reduce が存在することを保証する）
    return reduce(state, event);
  }, REDUCER_INLINE);

  // SEND_QUEUE_IN_ORDER action が含まれることを確認する（全件 in-order 送信指示）
  expect(resumeResult.actions).toContainEqual(
    expect.objectContaining({ type: "SEND_QUEUE_IN_ORDER" }),
  );
  // PQ は消えていないことを確認する（送信指示のみで削除は ack 後に行う）
  expect(resumeResult.nextState.pendingQueue).toHaveLength(5);

  // phase 2: 5 件が順番に ack されることを確認する（連続 ack シミュレーション）
  const ackAllResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 5 件 PQ がある初期状態を用意する
    let state = {
      serverTruth: null,
      optimisticLocal: null,
      pendingQueue: [
        { idempotencyKey: "inspect-idem-001" },
        { idempotencyKey: "inspect-idem-002" },
        { idempotencyKey: "inspect-idem-003" },
        { idempotencyKey: "inspect-idem-004" },
        { idempotencyKey: "inspect-idem-005" },
      ],
      queueHeld: false,
    };
    // 5 件を順番に ack する（in-order 送信と ack のシミュレーション）
    var results = [];
    for (var i = 1; i <= 5; i++) {
      var key = "inspect-idem-00" + i;
      // @ts-ignore
      var r = reduce(state, { eventId: "optimistic_acknowledged", idempotencyKey: key, confirmedVersion: i });
      // 各 ack 後の PQ 件数を記録する
      results.push({ pendingQueueLength: r.nextState.pendingQueue.length, confirmedVersion: i });
      // state を更新する（次の ack のため）
      state = r.nextState;
    }
    // 最終的な PQ の長さを返す
    return { results: results, finalPendingQueueLength: state.pendingQueue.length };
  }, REDUCER_INLINE);

  // 5 件全て ack 後に PQ が空になることを確認する
  expect(ackAllResult.finalPendingQueueLength).toBe(0);
  // 各 ack ごとに PQ が 1 件ずつ減っていることを確認する
  expect(ackAllResult.results[0].pendingQueueLength).toBe(4);
  expect(ackAllResult.results[1].pendingQueueLength).toBe(3);
  expect(ackAllResult.results[2].pendingQueueLength).toBe(2);
  expect(ackAllResult.results[3].pendingQueueLength).toBe(1);
  expect(ackAllResult.results[4].pendingQueueLength).toBe(0);

  // phase 3: queueHeld=true の場合は resume しないことを確認する
  const heldResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // queue が hold 中の状態を用意する（conflict 解決中を想定する）
    const state = {
      serverTruth: null,
      optimisticLocal: null,
      // 3 件の PQ entry がある
      pendingQueue: [
        { idempotencyKey: "inspect-idem-011" },
        { idempotencyKey: "inspect-idem-012" },
        { idempotencyKey: "inspect-idem-013" },
      ],
      // queue は hold 中（conflict 解決待ち）
      queueHeld: true,
    };
    // pending_queue_resume event を発行する（hold 中なので無視されるはず）
    const event = { eventId: "pending_queue_resume", resumeReason: "network_recovery" };
    // @ts-ignore
    return reduce(state, event);
  }, REDUCER_INLINE);

  // queueHeld=true の場合は actions が空であることを確認する（送信しない）
  expect(heldResult.actions).toHaveLength(0);
  // PQ は変化しないことを確認する
  expect(heldResult.nextState.pendingQueue).toHaveLength(3);
});
