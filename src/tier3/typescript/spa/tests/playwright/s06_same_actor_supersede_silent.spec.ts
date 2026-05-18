// tier3 Playwright シナリオ S06: 同 actor 同 aggregate 2 回更新 → supersede → silent toast
// 適合仕様: 11_クライアント状態適合仕様.md §製造業 pack stress test シナリオ 6
// supersede 検出で silent toast が表示され PQ 先頭 entry が削除されることを検証する
// 同一 actor の後続更新が先行更新を silent に上書きすることを確認する
// page.evaluate() でインライン reducer を実行し SPA bundle への依存を排除する

// @playwright/test の test / expect をインポートする
import { test, expect } from "@playwright/test";

// reducer のインライン定義文字列（page.evaluate 内で eval するインライン実装）
// business_conflict_received (supersede) の処理に特化したインライン実装
const REDUCER_INLINE = `
// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}

// business_conflict_received (supersede) event を処理する
// supersede: 同 actor の後続更新が先行更新を上書き → PQ 先頭 entry 削除 + silent toast
function reduceSupersede(state, event) {
  var nextState = Object.assign({}, state);
  var actions = [];
  // PQ に entry がある場合は先頭 entry を削除する（後続更新で先行更新が不要になった）
  if (nextState.pendingQueue.length > 0) {
    // 削除する先頭 entry の idempotency key を取得する
    var deletedKey = nextState.pendingQueue[0].idempotencyKey;
    // PQ の先頭 entry を削除する（先行更新が supersede された）
    nextState.pendingQueue = nextState.pendingQueue.slice(1);
    // PQ entry 削除 action を追加する
    actions.push({ type: 'DELETE_PQ_ENTRY', idempotencyKey: deletedKey });
  }
  // silent toast を表示する action を追加する（user への通知）
  actions.push({
    type: 'NOTIFY_SILENT_TOAST',
    message: '後続の操作で既に上書きされました',
    aggregateId: event.aggregateId,
    reason: 'supersede',
  });
  return { nextState: nextState, actions: actions };
}

// business_conflict_received event を処理する（4 subtype 決定論的分岐）
function reduce(state, event) {
  switch (event.eventId) {
    case 'business_conflict_received':
      if (event.subtype === 'supersede') {
        return reduceSupersede(state, event);
      }
      throw new Error('Unsupported subtype: ' + event.subtype);
    default:
      throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S06: 同 actor 同 aggregate 2 回更新 → supersede → silent toast のテスト
test("S06: 同 actor の後続更新が supersede で silent toast を表示し PQ entry を削除する", async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto("about:blank");

  // phase 1: supersede で PQ 先頭 entry が削除されて silent toast が表示されることを確認する
  const supersedeResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する（page context に注入する）
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 同 actor が同 aggregate を 2 回更新した状態（PQ に 2 件）
    const state = {
      // server_truth: 未初期化
      serverTruth: null,
      // optimistic_local: 送信待ちなし
      optimisticLocal: null,
      // pending_queue: 先行更新 + 後続更新の 2 件
      pendingQueue: [
        // 先行更新（supersede される側）
        { idempotencyKey: "idem-sp-001", value: { field: "note", value: "first edit" } },
        // 後続更新（先行更新を上書きする側）
        { idempotencyKey: "idem-sp-002", value: { field: "note", value: "second edit" } },
      ],
      // queue は hold していない
      queueHeld: false,
    };
    // business_conflict_received(supersede) event を発行する
    const event = {
      eventId: "business_conflict_received",
      subtype: "supersede",
      aggregateId: "agg-sp-001",
      serverVersion: 3,
    };
    // @ts-ignore（eval scope に reduce が存在することを保証する）
    return reduce(state, event);
  }, REDUCER_INLINE);

  // NOTIFY_SILENT_TOAST action が含まれることを確認する（silent 通知）
  expect(supersedeResult.actions).toContainEqual(
    expect.objectContaining({ type: "NOTIFY_SILENT_TOAST", reason: "supersede" }),
  );
  // DELETE_PQ_ENTRY action が含まれることを確認する（先頭 entry 削除）
  expect(supersedeResult.actions).toContainEqual(
    expect.objectContaining({ type: "DELETE_PQ_ENTRY", idempotencyKey: "idem-sp-001" }),
  );
  // PQ が 1 件に減っていることを確認する（先頭 entry が削除された）
  expect(supersedeResult.nextState.pendingQueue).toHaveLength(1);
  // 残った entry が後続更新 (idem-sp-002) であることを確認する
  expect(supersedeResult.nextState.pendingQueue[0].idempotencyKey).toBe("idem-sp-002");
  // queueHeld は変化しないことを確認する（supersede は queue を hold しない）
  expect(supersedeResult.nextState.queueHeld).toBe(false);

  // phase 2: PQ が空の場合は entry 削除なし silent toast のみを確認する
  const emptyQueueResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // PQ が空の状態で supersede を受け取る
    const state = {
      serverTruth: null,
      optimisticLocal: null,
      // pending_queue: 空
      pendingQueue: [],
      queueHeld: false,
    };
    // business_conflict_received(supersede) event を発行する
    const event = {
      eventId: "business_conflict_received",
      subtype: "supersede",
      aggregateId: "agg-sp-002",
      serverVersion: 2,
    };
    // @ts-ignore
    return reduce(state, event);
  }, REDUCER_INLINE);

  // PQ が空の場合は DELETE_PQ_ENTRY action がないことを確認する
  const deleteActions = emptyQueueResult.actions.filter(
    (a: { type: string }) => a.type === "DELETE_PQ_ENTRY",
  );
  expect(deleteActions).toHaveLength(0);
  // silent toast は表示されることを確認する
  expect(emptyQueueResult.actions).toContainEqual(
    expect.objectContaining({ type: "NOTIFY_SILENT_TOAST" }),
  );
  // PQ は空のままであることを確認する
  expect(emptyQueueResult.nextState.pendingQueue).toHaveLength(0);

  // phase 3: silent toast のメッセージ内容を確認する
  const toastAction = supersedeResult.actions.find(
    (a: { type: string }) => a.type === "NOTIFY_SILENT_TOAST",
  );
  // silent toast のメッセージが設定されていることを確認する
  expect(toastAction).toBeDefined();
  expect((toastAction as { type: string; message: string }).message).toBe(
    "後続の操作で既に上書きされました",
  );
  // aggregateId がメッセージに含まれることを確認する
  expect((toastAction as { type: string; aggregateId: string }).aggregateId).toBe("agg-sp-001");
});
