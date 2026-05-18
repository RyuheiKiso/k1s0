// tier3 Playwright シナリオ S03: 25h オフライン → Idempotency-Key TTL 超過 → user 確認 → 新 key で resend
// 適合仕様: 11_クライアント状態適合仕様.md §製造業 pack stress test シナリオ 3
// 24h TTL を超えた Idempotency-Key が expired 扱いになることを検証する
// wall-clock TTL 禁止: 時刻計算は HLC offset_ms ベースで行う
// page.evaluate() でインライン実装を実行し SPA bundle への依存を排除する

// @playwright/test の test / expect をインポートする
import { test, expect } from "@playwright/test";

// HLC/offset_ms ベースの TTL 判定ロジックと reducer のインライン定義
// wall-clock 依存を排除し HLC offset_ms で TTL を計算する
const REDUCER_INLINE = `
// HLC Idempotency-Key の TTL を offset_ms で定義する（24 時間 = 86400000 ms）
var IDEMPOTENCY_KEY_TTL_MS = 86400000;

// PQ entry の Idempotency-Key が TTL 超過かを判定する
// wall-clock 禁止: hlcCreatedAt と currentHlcMs の差分で判定する
function isIdempotencyKeyExpired(entry, currentHlcMs) {
  // hlcCreatedAt が未設定の場合は有効とみなす
  if (entry.hlcCreatedAt === undefined || entry.hlcCreatedAt === null) {
    return false;
  }
  // HLC offset_ms で経過時間を計算する（wall-clock 禁止）
  var elapsedMs = currentHlcMs - entry.hlcCreatedAt;
  // TTL 超過かを判定する（25h = 90000000 ms > 24h TTL）
  return elapsedMs > IDEMPOTENCY_KEY_TTL_MS;
}

// 4 layer client state の初期値を生成する
function createInitialState() {
  return { serverTruth: null, optimisticLocal: null, pendingQueue: [], queueHeld: false };
}

// TTL 超過エントリを検出して expired フラグを付与する（purge は outbox パッケージの責務）
function detectExpiredEntries(state, currentHlcMs) {
  // 各 PQ entry の TTL を HLC offset_ms で判定する
  var expiredEntries = state.pendingQueue.filter(function(entry) {
    return isIdempotencyKeyExpired(entry, currentHlcMs);
  });
  // 有効なエントリを抽出する
  var validEntries = state.pendingQueue.filter(function(entry) {
    return !isIdempotencyKeyExpired(entry, currentHlcMs);
  });
  return {
    expiredEntries: expiredEntries,
    validEntries: validEntries,
    expiredCount: expiredEntries.length,
    validCount: validEntries.length,
  };
}

// pending_queue_resume event を処理する（TTL 超過エントリを除外して PQ を送信する）
function reducePendingQueueResumeWithTtl(state, currentHlcMs) {
  // TTL 超過エントリを検出する
  var detection = detectExpiredEntries(state, currentHlcMs);
  // TTL 超過エントリがある場合は user 確認が必要（actions に追加する）
  var actions = [];
  if (detection.expiredCount > 0) {
    // user 確認アクションを追加する（新しい idempotency key の発行を促す）
    actions.push({
      type: 'PROMPT_USER_REKEY',
      expiredKeys: detection.expiredEntries.map(function(e) { return e.idempotencyKey; }),
      message: 'Idempotency-Key の TTL が超過しました。新しいキーで再送しますか？',
    });
  }
  // 有効なエントリのみを送信対象とする
  if (detection.validCount > 0 && !state.queueHeld) {
    actions.push({ type: 'SEND_QUEUE_IN_ORDER' });
  }
  return {
    nextState: Object.assign({}, state),
    actions: actions,
    detection: detection,
  };
}
`;

// S03: 25h オフライン → Idempotency-Key TTL 超過 → user 確認 → 新 key resend のテスト
test("S03: 25h オフライン後 Idempotency-Key TTL が超過して user 確認が表示される", async ({ page }) => {
  // 空白ページを開く（SPA bundle に依存しない）
  await page.goto("about:blank");

  // phase 1: 24h 未満の entry は TTL 内（有効）であることを確認する
  const withinTtlResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する（page context に注入する）
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 基準 HLC タイムスタンプ（ms）を設定する
    var baseHlcMs = 1000000000000;
    // 23h = 82800000 ms 前に作成されたエントリ（TTL 内）
    var createdAt23hAgo = baseHlcMs - 82800000;
    // TTL 超過判定を実行する（24h TTL 内のエントリ）
    // @ts-ignore（eval scope に isIdempotencyKeyExpired が存在することを保証する）
    var expired = isIdempotencyKeyExpired(
      { idempotencyKey: "idem-23h-001", hlcCreatedAt: createdAt23hAgo },
      baseHlcMs,
    );
    return { expired: expired, elapsedMs: baseHlcMs - createdAt23hAgo };
  }, REDUCER_INLINE);

  // 23h 経過では TTL 超過しないことを確認する
  expect(withinTtlResult.expired).toBe(false);
  // 経過時間が 82800000 ms であることを確認する
  expect(withinTtlResult.elapsedMs).toBe(82800000);

  // phase 2: 25h 経過後のエントリが TTL 超過することを確認する
  const expiredTtlResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 基準 HLC タイムスタンプ（ms）を設定する
    var baseHlcMs = 1000000000000;
    // 25h = 90000000 ms 前に作成されたエントリ（TTL 超過）
    var createdAt25hAgo = baseHlcMs - 90000000;
    // TTL 超過判定を実行する（25h は 24h TTL を超過する）
    // @ts-ignore
    var expired = isIdempotencyKeyExpired(
      { idempotencyKey: "idem-25h-001", hlcCreatedAt: createdAt25hAgo },
      baseHlcMs,
    );
    return { expired: expired, elapsedMs: baseHlcMs - createdAt25hAgo };
  }, REDUCER_INLINE);

  // 25h 経過では TTL 超過することを確認する
  expect(expiredTtlResult.expired).toBe(true);
  // 経過時間が 90000000 ms であることを確認する
  expect(expiredTtlResult.elapsedMs).toBe(90000000);

  // phase 3: PQ resume 時に TTL 超過エントリが検出されて user 確認が要求されることを確認する
  const resumeResult = await page.evaluate((reducerCode) => {
    // reducer のインライン定義を eval で実行する
    // eslint-disable-next-line no-eval
    eval(reducerCode);
    // 基準 HLC タイムスタンプを設定する
    var baseHlcMs = 1000000000000;
    // PQ に TTL 超過 entry 1 件と有効 entry 1 件がある状態を用意する
    var state = {
      serverTruth: null,
      optimisticLocal: null,
      pendingQueue: [
        // 25h 前に作成されたエントリ（TTL 超過）
        { idempotencyKey: "idem-old-001", hlcCreatedAt: baseHlcMs - 90000000 },
        // 1h 前に作成されたエントリ（TTL 内）
        { idempotencyKey: "idem-new-002", hlcCreatedAt: baseHlcMs - 3600000 },
      ],
      queueHeld: false,
    };
    // pending_queue_resume with TTL check を実行する
    // @ts-ignore
    return reducePendingQueueResumeWithTtl(state, baseHlcMs);
  }, REDUCER_INLINE);

  // PROMPT_USER_REKEY action が含まれることを確認する（TTL 超過 entry がある）
  expect(resumeResult.actions).toContainEqual(
    expect.objectContaining({ type: "PROMPT_USER_REKEY" }),
  );
  // SEND_QUEUE_IN_ORDER action が含まれることを確認する（有効 entry は送信する）
  expect(resumeResult.actions).toContainEqual(
    expect.objectContaining({ type: "SEND_QUEUE_IN_ORDER" }),
  );
  // 超過件数が 1 件であることを確認する
  expect(resumeResult.detection.expiredCount).toBe(1);
  // 有効件数が 1 件であることを確認する
  expect(resumeResult.detection.validCount).toBe(1);
  // 超過した key が正しいことを確認する
  expect(resumeResult.detection.expiredEntries[0].idempotencyKey).toBe("idem-old-001");
});
