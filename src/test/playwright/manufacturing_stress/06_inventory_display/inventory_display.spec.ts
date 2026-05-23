// @conformanceAssert(["live_snapshot_latest_wins_under_loss",
//   "live_snapshot_500ms_lag_within", "live_snapshot_first_tile_within_500ms",
//   "proxy_buffer_inject_handled"])
// Spec 06: 在庫最新値表示 (v1_live_snapshot) — 01_Bidi 適合仕様 §158 ship blocker 6/9
// 倉庫・棚の在庫数をリアルタイム表示する live snapshot クラスの assertion を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// slo_assertions から v1_live_snapshot assertion をインポートする
import {
  assertLiveSnapshotLatestWins,
  assertLiveSnapshot500msLag,
  assertFirstTileWithin500ms,
} from "../support/assertions/slo_assertions";
// bidi_assertions からプロキシ assertion をインポートする
import { assertProxyBufferHandled } from "../support/assertions/bidi_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に在庫表示向けスタックを起動する
test.beforeAll(async () => {
  // PostgreSQL RLS を有効化して在庫データを取得するスタックを起動する
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

// テストの説明ブロック: 在庫最新値表示 v1_live_snapshot
test.describe("Spec 06: 在庫最新値表示 (v1_live_snapshot)", () => {
  // テスト 1: 連続更新時の最新値優先確認
  test("live_snapshot_latest_wins_under_loss — 在庫数の高頻度更新で最新値が優先される", async ({ page }) => {
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

  // テスト 2: 在庫更新遅延 500ms 上限確認
  test("live_snapshot_500ms_lag_within — 在庫数更新の遅延が 500ms 以内に収まる", async ({ page }) => {
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

  // テスト 3: 初期在庫表示タイミング確認
  test("live_snapshot_first_tile_within_500ms — 在庫ページロード後 500ms 以内に最初の数値が表示される", async ({ page }) => {
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

  // テスト 4: プロキシ経由の在庫更新処理確認
  test("proxy_buffer_inject_handled — プロキシ経由でも在庫最新値が正確に表示される", async ({ page }) => {
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
});
