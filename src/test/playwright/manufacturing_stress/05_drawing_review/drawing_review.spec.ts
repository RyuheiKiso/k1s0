// @conformanceAssert(["pl_seq_monotonic_per_session", "rd_seq_continuous_across_resume",
//   "backpressure_applied_correctly", "half_close_terminates_cleanly",
//   "proxy_buffer_inject_handled", "resume_token_hlc_valid",
//   "large_msg_fragmented_correctly", "keepalive_maintained_after_idle",
//   "crdt_merge_no_data_loss", "presence_conflict_resolved"])
// Spec 05: 図面 collaborative review (v1_interactive) — 01_Bidi 適合仕様 §158 ship blocker 5/9
// 複数エンジニアが同時に図面を編集するリアルタイム協調作業の assertion を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// bidi_assertions から v1_interactive 全 8 assertion をインポートする
import {
  assertBidiSeqMonotonic,
  assertBidiResumeSeqContinuous,
  assertBackpressureApplied,
  assertHalfCloseTerminatesCleanly,
  assertProxyBufferHandled,
  assertHlcResumeTokenValid,
  assertLargeMsgFragmented,
  assertKeepaliveAfterIdle,
} from "../support/assertions/bidi_assertions";
// crdt_assertions から CRDT / presence 固有 assertion をインポートする
import {
  assertCrdtMergeNoDataLoss,
  assertPresenceConflictResolved,
} from "../support/assertions/crdt_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に collaborative review 向けスタックを起動する
test.beforeAll(async () => {
  // Valkey (CRDT state) と Keycloak を有効化したスタックを起動する
  stack = await startManufacturingStack({
    adapters: ["grpc_native", "connect_bidi", "web_transport"],
    requireKeycloak: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: 図面 collaborative review v1_interactive
test.describe("Spec 05: 図面 collaborative review (v1_interactive)", () => {
  // テスト 1: v1_interactive 基本 assertion (設備リモートと同クラス)
  test("pl_seq_monotonic_per_session — 図面編集操作のシーケンスが単調増加する", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // pl_seq_monotonic_per_session assertion を実行する
    await assertBidiSeqMonotonic(page);
  });

  // テスト 2: 切断後の図面編集再接続
  test("rd_seq_continuous_across_resume — ネットワーク切断後に図面編集が再開できる", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // rd_seq_continuous_across_resume assertion を実行する
    await assertBidiResumeSeqContinuous(page);
  });

  // テスト 3: 大量編集時の backpressure 確認
  test("backpressure_applied_correctly — 大量の図面編集操作時に backpressure が適用される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // backpressure_applied_correctly assertion を実行する
    await assertBackpressureApplied(page);
  });

  // テスト 4: レビュー完了後の half-close 確認
  test("half_close_terminates_cleanly — レビューセッション終了後にコネクションがクリーンに終了する", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // half_close_terminates_cleanly assertion を実行する
    await assertHalfCloseTerminatesCleanly(page);
  });

  // テスト 5: プロキシバッファリング処理確認
  test("proxy_buffer_inject_handled — プロキシ経由でもリアルタイム編集が正常動作する", async ({ page }) => {
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

  // テスト 6: HLC resume_token 確認
  test("resume_token_hlc_valid — 図面編集の resume_token が HLC 形式を持つ", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // resume_token_hlc_valid assertion を実行する
    await assertHlcResumeTokenValid(page);
  });

  // テスト 7: 大きな図面ファイルのフラグメンテーション確認
  test("large_msg_fragmented_correctly — 大きな図面ファイルが正しくフラグメント化される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // large_msg_fragmented_correctly assertion を実行する
    await assertLargeMsgFragmented(page);
  });

  // テスト 8: アイドル後 keepalive 維持確認
  test("keepalive_maintained_after_idle — 図面レビューのアイドル期間も接続が維持される", async ({ page }) => {
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

  // テスト 9: CRDT merge データロスなし確認
  test("crdt_merge_no_data_loss — 2 エンジニアの並列編集が全てマージされる", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // crdt_merge_no_data_loss assertion を実行する
    await assertCrdtMergeNoDataLoss(page);
  });

  // テスト 10: presence conflict 解決確認
  test("presence_conflict_resolved — 同一部品への同時編集 conflict が正しく解決される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // presence_conflict_resolved assertion を実行する
    await assertPresenceConflictResolved(page);
  });
});
