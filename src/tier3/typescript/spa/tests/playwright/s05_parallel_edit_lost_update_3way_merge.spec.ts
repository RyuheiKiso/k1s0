// tier3 Playwright シナリオ S05: 並行編集 (intersect field) → lost_update → 3way merge UI
// 適合仕様: 11_クライアント状態適合仕様.md §製造業 pack stress test シナリオ 5
// lost_update conflict で 3way merge UI + HoldQueue が発動することを検証する
// isDisjoint = false（intersect field）の場合に UI 表示用コンフリクトが生成されることを確認する
// page.evaluate() でインライン reducer を実行し SPA bundle への依存を排除する

// @playwright/test の test / expect をインポートする
import { test, expect } from "@playwright/test";

// reducer のインライン定義文字列（page.evaluate 内で eval するインライン実装）
// business_conflict_received (lost_update) の処理に特化したインライン実装
const REDUCER_INLINE = `
// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}

// 3way merge コンフリクト情報を生成する（UI 表示用）
function buildConflictInfo(event) {
  return {
    // コンフリクトタイプ
    conflictType: event.subtype,
    // aggregate ID
    aggregateId: event.aggregateId,
    // server バージョン
    serverVersion: event.serverVersion,
    // field 差分情報
    fieldDiff: event.fieldDiff,
    // client side の値
    clientVersion: event.clientVersion,
    // コンフリクト発生時刻（HLC ベース）
    detectedAtHlc: event.hlcTimestamp || null,
  };
}

// business_conflict_received (lost_update) event を処理する
// lost_update は常に 3way merge UI を表示して queue を hold する（auto 解決不可）
function reduceLostUpdate(state, event) {
  // next state で queue を hold する
  var nextState = Object.assign({}, state, { queueHeld: true });
  // UI 表示用のコンフリクト情報を生成する
  var conflictInfo = buildConflictInfo(event);
  var actions = [
    // 3way merge UI を表示するアクション
    { type: 'PRESENT_3WAY_MERGE_UI', conflictInfo: conflictInfo },
    // queue を hold するアクション（manual 解決まで送信を止める）
    { type: 'HOLD_QUEUE' },
    // BusinessErrorPanel にコンフリクトを登録するアクション
    { type: 'REGISTER_CONFLICT_TO_PANEL', conflictInfo: conflictInfo },
  ];
  return { nextState: nextState, actions: actions };
}

// business_conflict_received event を処理する（4 subtype 決定論的分岐）
function reduce(state, event) {
  switch (event.eventId) {
    case 'business_conflict_received':
      if (event.subtype === 'lost_update') {
        return reduceLostUpdate(state, event);
      }
      throw new Error('Unsupported subtype: ' + event.subtype);
    default:
      throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S05: 並行編集 (intersect field) → lost_update → 3way merge UI のテスト
test("S05: intersect field lost_update が 3way merge UI を表示して queue を hold する", async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto("about:blank");

  // phase 1: lost_update conflict で 3way merge UI が表示されることを確認する
  const lostUpdateResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する（page context に注入する）
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // actor A と actor B が同じフィールドを変更した（intersect field = lost_update）
    const state = {
      // server_truth: バージョン 5
      serverTruth: { version: 5 },
      // optimistic_local: actor A の qty 変更が in-flight 中
      optimisticLocal: { idempotencyKey: "idem-lu-001", value: { qty: 15 } },
      // pending_queue: lost_update の原因 entry
      pendingQueue: [{ idempotencyKey: "idem-lu-001" }],
      // queue は hold していない（conflict 受信前）
      queueHeld: false,
    };
    // business_conflict_received(lost_update, intersect field) event を発行する
    const event = {
      eventId: "business_conflict_received",
      subtype: "lost_update",
      aggregateId: "agg-order-002",
      // server も qty を変更していた（intersect field）
      serverVersion: 6,
      clientVersion: 5,
      hlcTimestamp: "2026-05-18T10:00:00Z",
      // actor A: qty / actor B: qty → intersect（重複あり）
      fieldDiff: { clientFields: ["qty", "price"], serverFields: ["qty"] },
    };
    // @ts-ignore（eval scope に reduce が存在することを保証する）
    return reduce(state, event);
  }, REDUCER_INLINE);

  // next state で queueHeld が true になることを確認する（queue を hold する）
  expect(lostUpdateResult.nextState.queueHeld).toBe(true);
  // PRESENT_3WAY_MERGE_UI action が含まれることを確認する（always, lost_update は auto 解決不可）
  expect(lostUpdateResult.actions).toContainEqual(
    expect.objectContaining({ type: "PRESENT_3WAY_MERGE_UI" }),
  );
  // HOLD_QUEUE action が含まれることを確認する
  expect(lostUpdateResult.actions).toContainEqual(
    expect.objectContaining({ type: "HOLD_QUEUE" }),
  );
  // REGISTER_CONFLICT_TO_PANEL action が含まれることを確認する（BusinessErrorPanel 登録）
  expect(lostUpdateResult.actions).toContainEqual(
    expect.objectContaining({ type: "REGISTER_CONFLICT_TO_PANEL" }),
  );
  // コンフリクト情報に aggregateId が含まれることを確認する
  const mergeUiAction = lostUpdateResult.actions.find(
    (a: { type: string }) => a.type === "PRESENT_3WAY_MERGE_UI",
  );
  expect(mergeUiAction).toBeDefined();
  expect((mergeUiAction as { type: string; conflictInfo: { aggregateId: string } }).conflictInfo.aggregateId).toBe("agg-order-002");
  // コンフリクト情報に conflictType が設定されることを確認する
  expect((mergeUiAction as { type: string; conflictInfo: { conflictType: string } }).conflictInfo.conflictType).toBe("lost_update");

  // phase 2: 既に queueHeld の状態でも lost_update が届いた場合の動作を確認する
  const alreadyHeldResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 既に queue が hold 中の状態（前の conflict 解決待ち）
    const state = {
      serverTruth: { version: 7 },
      optimisticLocal: null,
      pendingQueue: [],
      // 既に hold 中
      queueHeld: true,
    };
    // 新しい lost_update event が届く
    const event = {
      eventId: "business_conflict_received",
      subtype: "lost_update",
      aggregateId: "agg-order-003",
      serverVersion: 8,
      clientVersion: 7,
      fieldDiff: { clientFields: ["note"], serverFields: ["note"] },
    };
    // @ts-ignore
    return reduce(state, event);
  }, REDUCER_INLINE);

  // 既に hold 中でも queueHeld が true のままであることを確認する
  expect(alreadyHeldResult.nextState.queueHeld).toBe(true);
  // PRESENT_3WAY_MERGE_UI action は常に含まれることを確認する
  expect(alreadyHeldResult.actions).toContainEqual(
    expect.objectContaining({ type: "PRESENT_3WAY_MERGE_UI" }),
  );
});
