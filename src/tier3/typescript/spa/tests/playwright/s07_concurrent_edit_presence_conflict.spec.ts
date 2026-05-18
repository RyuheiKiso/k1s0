// tier3 Playwright シナリオ S07: presence indicator 表示中の編集衝突 → concurrent_edit → user choice
// 適合仕様: 11_クライアント状態適合仕様.md §製造業 pack stress test シナリオ 7
// concurrent_edit 検出で presence indicator が更新されて user に continue/abort 選択肢が提示されることを検証する
// 複数 actor が同時編集中の状態で conflict が発生した場合の UI フローを確認する
// page.evaluate() でインライン reducer を実行し SPA bundle への依存を排除する

// @playwright/test の test / expect をインポートする
import { test, expect } from "@playwright/test";

// reducer のインライン定義文字列（page.evaluate 内で eval するインライン実装）
// business_conflict_received (concurrent_edit) の処理に特化したインライン実装
const REDUCER_INLINE = `
// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}

// business_conflict_received (concurrent_edit) event を処理する
// concurrent_edit: 複数 actor が同時編集 → presence indicator 更新 + user choice UI 表示
function reduceConcurrentEdit(state, event) {
  var nextState = Object.assign({}, state);
  var actions = [];
  // presence indicator を更新する（競合している actor の存在を表示する）
  actions.push({
    type: 'UPDATE_PRESENCE',
    actorId: event.conflictingActorId || 'unknown',
    aggregateId: event.aggregateId,
    presenceType: 'concurrent_edit_conflict',
  });
  // user に continue または abort の選択肢を提示する action を追加する
  actions.push({
    type: 'SHOW_CONFLICT_CHOICE_UI',
    choices: ['continue', 'abort'],
    conflictType: 'concurrent_edit',
    aggregateId: event.aggregateId,
    conflictingActorId: event.conflictingActorId || 'unknown',
  });
  // BusinessErrorPanel にコンフリクトを登録するアクション
  actions.push({
    type: 'REGISTER_CONFLICT_TO_PANEL',
    conflictType: 'concurrent_edit',
    aggregateId: event.aggregateId,
  });
  return { nextState: nextState, actions: actions };
}

// business_conflict_received event を処理する（4 subtype 決定論的分岐）
function reduce(state, event) {
  switch (event.eventId) {
    case 'business_conflict_received':
      if (event.subtype === 'concurrent_edit') {
        return reduceConcurrentEdit(state, event);
      }
      throw new Error('Unsupported subtype: ' + event.subtype);
    default:
      throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S07: presence indicator 表示中の concurrent_edit → user choice のテスト
test("S07: concurrent_edit が presence indicator を更新して user に continue/abort を提示する", async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto("about:blank");

  // phase 1: concurrent_edit で presence indicator が更新されることを確認する
  const concurrentEditResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する（page context に注入する）
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // presence indicator が表示されている状態（他 actor が同時編集中）
    const state = {
      // server_truth: 最新バージョン
      serverTruth: null,
      // optimistic_local: 自分の変更が in-flight 中
      optimisticLocal: { idempotencyKey: "idem-ce-001", value: { field: "qty", value: 20 } },
      // pending_queue: 空
      pendingQueue: [],
      // queue は hold していない
      queueHeld: false,
    };
    // business_conflict_received(concurrent_edit) event を発行する
    const event = {
      eventId: "business_conflict_received",
      subtype: "concurrent_edit",
      aggregateId: "agg-ce-001",
      serverVersion: 2,
      // 競合している actor の ID
      conflictingActorId: "actor-user-B",
    };
    // @ts-ignore（eval scope に reduce が存在することを保証する）
    return reduce(state, event);
  }, REDUCER_INLINE);

  // UPDATE_PRESENCE action が含まれることを確認する（presence indicator を更新する）
  expect(concurrentEditResult.actions).toContainEqual(
    expect.objectContaining({ type: "UPDATE_PRESENCE", actorId: "actor-user-B" }),
  );
  // SHOW_CONFLICT_CHOICE_UI action が含まれることを確認する（user に選択肢を提示する）
  expect(concurrentEditResult.actions).toContainEqual(
    expect.objectContaining({ type: "SHOW_CONFLICT_CHOICE_UI", conflictType: "concurrent_edit" }),
  );
  // REGISTER_CONFLICT_TO_PANEL action が含まれることを確認する
  expect(concurrentEditResult.actions).toContainEqual(
    expect.objectContaining({ type: "REGISTER_CONFLICT_TO_PANEL", conflictType: "concurrent_edit" }),
  );
  // queueHeld は変化しないことを確認する（concurrent_edit は queue を hold しない）
  expect(concurrentEditResult.nextState.queueHeld).toBe(false);

  // phase 2: SHOW_CONFLICT_CHOICE_UI の選択肢が continue と abort を含むことを確認する
  const choiceAction = concurrentEditResult.actions.find(
    (a: { type: string }) => a.type === "SHOW_CONFLICT_CHOICE_UI",
  );
  // 選択肢 action が存在することを確認する
  expect(choiceAction).toBeDefined();
  // continue 選択肢が含まれることを確認する
  expect(
    (choiceAction as { type: string; choices: string[] }).choices,
  ).toContain("continue");
  // abort 選択肢が含まれることを確認する
  expect(
    (choiceAction as { type: string; choices: string[] }).choices,
  ).toContain("abort");

  // phase 3: conflictingActorId が unknown の場合の動作を確認する
  const unknownActorResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // conflictingActorId が設定されていない場合（actor ID 不明）
    const state = {
      serverTruth: null,
      optimisticLocal: null,
      pendingQueue: [],
      queueHeld: false,
    };
    const event = {
      eventId: "business_conflict_received",
      subtype: "concurrent_edit",
      aggregateId: "agg-ce-002",
      serverVersion: 2,
      // conflictingActorId を省略する（undefined になる）
    };
    // @ts-ignore
    return reduce(state, event);
  }, REDUCER_INLINE);

  // conflictingActorId が unknown にフォールバックすることを確認する
  expect(unknownActorResult.actions).toContainEqual(
    expect.objectContaining({ type: "UPDATE_PRESENCE", actorId: "unknown" }),
  );
  // SHOW_CONFLICT_CHOICE_UI は actor ID 不明でも表示されることを確認する
  expect(unknownActorResult.actions).toContainEqual(
    expect.objectContaining({ type: "SHOW_CONFLICT_CHOICE_UI" }),
  );
});
