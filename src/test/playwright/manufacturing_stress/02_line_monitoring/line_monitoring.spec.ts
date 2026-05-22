// @conformanceAssert(["live_snapshot_latest_wins_under_loss",
//   "live_snapshot_500ms_lag_within", "live_snapshot_first_tile_within_500ms",
//   "proxy_buffer_inject_handled", "keepalive_maintained_after_idle"])
// Spec 02: ライン稼働監視 (v1_live_snapshot) — 01_Bidi 適合仕様 §158 ship blocker 2/9
// 生産ラインのリアルタイム稼働率をタイル表示する live snapshot クラスの assertion を検証する

// @playwright/test から test と expect をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// slo_assertions から v1_live_snapshot assertion 関数をインポートする
import {
  assertLiveSnapshotLatestWins,
  assertLiveSnapshot500msLag,
  assertFirstTileWithin500ms,
} from "../support/assertions/slo_assertions";
// bidi_assertions からプロキシ / keepalive assertion をインポートする
import {
  assertProxyBufferHandled,
  assertKeepaliveAfterIdle,
} from "../support/assertions/bidi_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に live snapshot 向けスタックを起動する
test.beforeAll(async () => {
  // PostgreSQL と接続して RLS で稼働データを取得するスタックを起動する
  stack = await startManufacturingStack({
    adapters: ["sse_paired", "long_poll"],
    enableRls: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: ライン稼働監視 v1_live_snapshot
test.describe("Spec 02: ライン稼働監視 (v1_live_snapshot)", () => {
  // テスト 1: パケットロス下での最新値優先確認
  test("live_snapshot_latest_wins_under_loss — パケットロス下でも最新の稼働データが表示される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // live_snapshot_latest_wins_under_loss assertion を実行する
    await assertLiveSnapshotLatestWins(page);
  });

  // テスト 2: 500ms 遅延上限確認
  test("live_snapshot_500ms_lag_within — 稼働データの更新遅延が 500ms 以内に収まる", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // live_snapshot_500ms_lag_within assertion を実行する
    await assertLiveSnapshot500msLag(page);
  });

  // テスト 3: ページロード後 500ms 以内の最初タイル表示確認
  test("live_snapshot_first_tile_within_500ms — ページロード後 500ms 以内に稼働タイルが表示される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // live_snapshot_first_tile_within_500ms assertion を実行する
    await assertFirstTileWithin500ms(page);
  });

  // テスト 4: プロキシバッファリング処理確認 (live snapshot 向け)
  test("proxy_buffer_inject_handled — プロキシ経由でのバッファリング遅延を処理する", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // proxy_buffer_inject_handled assertion を実行する
    await assertProxyBufferHandled(page);
  });

  // テスト 5: アイドル後 keepalive 維持確認 (ライン常時監視向け)
  test("keepalive_maintained_after_idle — 監視ページがアイドル状態でも接続が維持される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // keepalive_maintained_after_idle assertion を実行する
    await assertKeepaliveAfterIdle(page);
  });
});
