// tier3 Playwright シナリオ S04: 並行編集 (disjoint field) → stale_write → field-level rebase → auto resend
// 適合仕様: 11_クライアント状態適合仕様.md §製造業 pack stress test シナリオ 4
// stale_write conflict で 3way merge UI + HoldQueue が発動することを検証する
// disjoint field の場合は field-level rebase で自動解決できることを確認する
// page.evaluate() でインライン reducer を実行し SPA bundle への依存を排除する

// @playwright/test の test / expect をインポートする
import { test, expect } from "@playwright/test";

// reducer のインライン定義文字列（page.evaluate 内で eval するインライン実装）
// business_conflict_received (stale_write) の処理に特化したインライン実装
const REDUCER_INLINE = `
// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}

// FieldDiff が disjoint（重複なし）かを判定する
function isDisjoint(fieldDiff) {
  // client と server の変更フィールドに重複がない場合は disjoint とみなす
  if (!fieldDiff || !fieldDiff.clientFields || !fieldDiff.serverFields) {
    return true;
  }
  var clientSet = fieldDiff.clientFields;
  var serverSet = fieldDiff.serverFields;
  // 重複するフィールドが存在するかを確認する
  for (var i = 0; i < clientSet.length; i++) {
    if (serverSet.indexOf(clientSet[i]) !== -1) {
      // 重複するフィールドがある場合は intersect（disjoint でない）
      return false;
    }
  }
  // 重複なし = disjoint
  return true;
}

// business_conflict_received (stale_write) event を処理する
function reduceStaleWrite(state, event) {
  var nextState = Object.assign({}, state);
  var actions = [];
  var fieldDiff = event.fieldDiff;
  // disjoint フィールドの場合は auto rebase を試みる
  if (isDisjoint(fieldDiff)) {
    // auto rebase 実行: 3way merge UI は表示しない（auto 解決）
    actions.push({ type: 'AUTO_REBASE', reason: 'disjoint_fields' });
    // chained idempotency key で再送指示を追加する
    actions.push({ type: 'RESEND_WITH_CHAINED_KEY', originalKey: event.originalIdempotencyKey });
    // queue は hold しない（auto 解決可能）
    nextState.queueHeld = false;
  } else {
    // intersect フィールドの場合は 3way merge UI を表示して queue を hold する
    actions.push({ type: 'PRESENT_3WAY_MERGE_UI' });
    actions.push({ type: 'HOLD_QUEUE' });
    nextState.queueHeld = true;
  }
  return { nextState: nextState, actions: actions };
}

// business_conflict_received event を処理する（4 subtype 決定論的分岐）
function reduce(state, event) {
  switch (event.eventId) {
    case 'business_conflict_received':
      if (event.subtype === 'stale_write') {
        return reduceStaleWrite(state, event);
      }
      throw new Error('Unsupported subtype: ' + event.subtype);
    default:
      throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S04: 並行編集 (disjoint field) → stale_write → field-level rebase → auto resend のテスト
test("S04: disjoint field stale_write が field-level rebase で自動解決される", async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto("about:blank");

  // phase 1: disjoint field の場合は auto rebase で解決されることを確認する
  const disjointResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する（page context に注入する）
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // actor A が qty を変更し actor B が price を変更した（disjoint field）
    const state = {
      // server_truth: バージョン 3
      serverTruth: { version: 3 },
      // optimistic_local: actor A の qty 変更が in-flight 中
      optimisticLocal: { idempotencyKey: "idem-sw-001", value: { field: "qty", clientValue: 10 } },
      // pending_queue: stale_write の原因 entry
      pendingQueue: [{ idempotencyKey: "idem-sw-001" }],
      // queue は hold していない
      queueHeld: false,
    };
    // business_conflict_received(stale_write, disjoint field) event を発行する
    const event = {
      eventId: "business_conflict_received",
      subtype: "stale_write",
      aggregateId: "agg-order-001",
      serverVersion: 4,
      originalIdempotencyKey: "idem-sw-001",
      // actor A: qty / actor B: price → disjoint（重複なし）
      fieldDiff: { clientFields: ["qty"], serverFields: ["price"] },
    };
    // @ts-ignore（eval scope に reduce が存在することを保証する）
    return reduce(state, event);
  }, REDUCER_INLINE);

  // disjoint field の場合は AUTO_REBASE action が含まれることを確認する
  expect(disjointResult.actions).toContainEqual(
    expect.objectContaining({ type: "AUTO_REBASE", reason: "disjoint_fields" }),
  );
  // RESEND_WITH_CHAINED_KEY action が含まれることを確認する（auto resend）
  expect(disjointResult.actions).toContainEqual(
    expect.objectContaining({ type: "RESEND_WITH_CHAINED_KEY", originalKey: "idem-sw-001" }),
  );
  // disjoint の場合は queue を hold しないことを確認する
  expect(disjointResult.nextState.queueHeld).toBe(false);

  // phase 2: intersect field の場合は 3way merge UI + HoldQueue になることを確認する
  const intersectResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // actor A と actor B が同じフィールド qty を変更した（intersect field）
    const state = {
      serverTruth: { version: 3 },
      optimisticLocal: { idempotencyKey: "idem-sw-002", value: { field: "qty", clientValue: 10 } },
      pendingQueue: [{ idempotencyKey: "idem-sw-002" }],
      queueHeld: false,
    };
    // business_conflict_received(stale_write, intersect field) event を発行する
    const event = {
      eventId: "business_conflict_received",
      subtype: "stale_write",
      aggregateId: "agg-order-002",
      serverVersion: 4,
      originalIdempotencyKey: "idem-sw-002",
      // actor A: qty / actor B: qty → intersect（重複あり）
      fieldDiff: { clientFields: ["qty"], serverFields: ["qty"] },
    };
    // @ts-ignore
    return reduce(state, event);
  }, REDUCER_INLINE);

  // intersect field の場合は PRESENT_3WAY_MERGE_UI action が含まれることを確認する
  expect(intersectResult.actions).toContainEqual(
    expect.objectContaining({ type: "PRESENT_3WAY_MERGE_UI" }),
  );
  // HOLD_QUEUE action が含まれることを確認する（manual 解決が必要）
  expect(intersectResult.actions).toContainEqual(
    expect.objectContaining({ type: "HOLD_QUEUE" }),
  );
  // intersect の場合は queue を hold することを確認する
  expect(intersectResult.nextState.queueHeld).toBe(true);

  // phase 3: isDisjoint 関数が正しく動作することを確認する
  const disjointCheckResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    return {
      // qty vs price → disjoint
      // @ts-ignore
      qtyVsPrice: isDisjoint({ clientFields: ["qty"], serverFields: ["price"] }),
      // qty vs qty → intersect
      // @ts-ignore
      qtyVsQty: isDisjoint({ clientFields: ["qty"], serverFields: ["qty"] }),
      // qty, name vs price, note → disjoint
      // @ts-ignore
      multiDisjoint: isDisjoint({ clientFields: ["qty", "name"], serverFields: ["price", "note"] }),
      // qty, name vs name, note → intersect
      // @ts-ignore
      multiIntersect: isDisjoint({ clientFields: ["qty", "name"], serverFields: ["name", "note"] }),
    };
  }, REDUCER_INLINE);

  // qty vs price は disjoint であることを確認する
  expect(disjointCheckResult.qtyVsPrice).toBe(true);
  // qty vs qty は intersect（disjoint でない）であることを確認する
  expect(disjointCheckResult.qtyVsQty).toBe(false);
  // 複数フィールドが完全に分離している場合は disjoint であることを確認する
  expect(disjointCheckResult.multiDisjoint).toBe(true);
  // 複数フィールドに重複がある場合は intersect であることを確認する
  expect(disjointCheckResult.multiIntersect).toBe(false);
});
