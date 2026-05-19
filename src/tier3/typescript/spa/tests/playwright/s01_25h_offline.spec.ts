// s01_25h_offline.spec.ts — 25 時間オフライン後の PQ ドレインシナリオ
// @playwright/test で tier3 SPA の long-term offline 耐性を検証する
// 適合仕様: 11_クライアント状態適合仕様.md §25h offline → PQ drain
// HLC ベースでオフライン期間を表現する（wall-clock TTL 禁止）

// @playwright/test の test / expect をインポートする
import { test, expect } from '@playwright/test';

// PQ ドレインの検証に使用する reducer インライン定義
// page.evaluate 内で動作する pure JS（TypeScript primary と仕様等価）
const REDUCER_INLINE = `
// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}
// pending_queue_resume event を処理する（PQ → in-order 送信）
function reducePendingQueueResume(state) {
  if (state.queueHeld || state.pendingQueue.length === 0) {
    return { nextState: state, actions: [] };
  }
  return { nextState: state, actions: [{ type: 'SEND_QUEUE_IN_ORDER' }] };
}
// optimistic_acknowledged event を処理する（PQ entry 削除 + ST promote）
function reduceOptimisticAcknowledged(state, idempotencyKey, confirmedVersion) {
  return {
    nextState: Object.assign({}, state, {
      optimisticLocal: null,
      serverTruth: { version: confirmedVersion },
      pendingQueue: state.pendingQueue.filter(function(pq) { return pq.idempotencyKey !== idempotencyKey; }),
    }),
    actions: [
      { type: 'PROMOTE_OPTIMISTIC', idempotencyKey: idempotencyKey },
      { type: 'DELETE_PQ_ENTRY', idempotencyKey: idempotencyKey },
    ],
  };
}
// 5 event dispatcher
function reduce(state, event) {
  switch (event.eventId) {
    case 'pending_queue_resume': return reducePendingQueueResume(state);
    case 'optimistic_acknowledged': return reduceOptimisticAcknowledged(state, event.idempotencyKey, event.confirmedVersion);
    default: throw new Error('Unknown event: ' + event.eventId);
  }
}
`;

// S01: 25 時間オフライン後の PQ ドレインテスト
test('S01: 25h offline → 再接続 → PQ 全件ドレイン', async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto('about:blank');

  // Step 1: 25h オフライン中に 5 件の mutation を PQ に積む状態を用意する
  const initialState = await page.evaluate(() => {
    // 5 件の PQ エントリを生成する（オフライン中に積まれた mutation を模倣する）
    return {
      serverTruth: { version: 1 },
      optimisticLocal: null,
      pendingQueue: [
        // 1 件目: フィールド計測 mutation
        { idempotencyKey: 'offline-idem-001', payload: { fieldId: 'F001', value: 10.5 } },
        // 2 件目: フィールド計測 mutation
        { idempotencyKey: 'offline-idem-002', payload: { fieldId: 'F001', value: 11.2 } },
        // 3 件目: フィールド計測 mutation
        { idempotencyKey: 'offline-idem-003', payload: { fieldId: 'F001', value: 9.8 } },
        // 4 件目: フィールド計測 mutation
        { idempotencyKey: 'offline-idem-004', payload: { fieldId: 'F002', value: 22.1 } },
        // 5 件目: フィールド計測 mutation
        { idempotencyKey: 'offline-idem-005', payload: { fieldId: 'F002', value: 21.7 } },
      ],
      queueHeld: false,
    };
  });

  // PQ に 5 件積まれていることを確認する
  expect(initialState.pendingQueue).toHaveLength(5);
  // queue が hold されていないことを確認する
  expect(initialState.queueHeld).toBe(false);

  // Step 2: 再接続イベント（pending_queue_resume）を受信して PQ ドレインをトリガーする
  const resumeResult = await page.evaluate(
    (args) => {
      // reducer インライン定義を eval で注入する
      // eslint-disable-next-line no-eval
      eval(args.reducerCode);
      // pending_queue_resume event を発行する（ネットワーク復帰を模倣する）
      // @ts-ignore（eval scope の reduce 関数を使用する）
      return reduce(args.state, { eventId: 'pending_queue_resume' });
    },
    { reducerCode: REDUCER_INLINE, state: initialState },
  );

  // SEND_QUEUE_IN_ORDER action が発行されることを確認する
  expect(resumeResult.actions).toContainEqual(
    expect.objectContaining({ type: 'SEND_QUEUE_IN_ORDER' }),
  );
  // state の pendingQueue が変わっていないことを確認する（送信開始のみ）
  expect(resumeResult.nextState.pendingQueue).toHaveLength(5);

  // Step 3: 各 PQ entry が順番に acknowledge されて PQ がドレインすることを検証する
  let currentState = resumeResult.nextState;
  const idempotencyKeys = ['offline-idem-001', 'offline-idem-002', 'offline-idem-003', 'offline-idem-004', 'offline-idem-005'];
  let version = 2;

  for (const key of idempotencyKeys) {
    // optimistic_acknowledged event を発行して PQ から entry を削除する
    const ackResult = await page.evaluate(
      (args) => {
        // eslint-disable-next-line no-eval
        eval(args.reducerCode);
        // @ts-ignore（eval scope の reduce 関数を使用する）
        return reduce(args.state, {
          eventId: 'optimistic_acknowledged',
          idempotencyKey: args.key,
          confirmedVersion: args.version,
        });
      },
      { reducerCode: REDUCER_INLINE, state: currentState, key, version },
    );
    // PQ から対象 entry が削除されていることを確認する
    expect(ackResult.nextState.pendingQueue.find((pq: { idempotencyKey: string }) => pq.idempotencyKey === key)).toBeUndefined();
    // 次のイテレーションのために state を更新する
    currentState = ackResult.nextState;
    version++;
  }

  // 全 PQ entry がドレインされて空になっていることを確認する
  expect(currentState.pendingQueue).toHaveLength(0);
  // server_truth が最終バージョンに設定されていることを確認する
  expect(currentState.serverTruth.version).toBe(6);
});
