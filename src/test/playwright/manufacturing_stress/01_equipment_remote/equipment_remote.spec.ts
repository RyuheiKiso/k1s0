// @conformanceAssert(["pl_seq_monotonic_per_session", "rd_seq_continuous_across_resume",
//   "backpressure_applied_correctly", "half_close_terminates_cleanly",
//   "proxy_buffer_inject_handled", "resume_token_hlc_valid",
//   "large_msg_fragmented_correctly", "keepalive_maintained_after_idle"])
// Spec 01: 設備リモート操作 (v1_interactive) — 01_Bidi 適合仕様 §158 ship blocker
// tier1 gateway に対して v1_interactive クラスの全 8 scenario assertion を検証する

// @playwright/test から test と expect をインポートする
import { test, expect } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// bidi_assertions から全 8 assertion 関数をインポートする
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

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前にスタックを起動する
test.beforeAll(async () => {
  // 製造業スタックを起動する（grpc_native + connect_bidi + web_transport adapter を使用）
  stack = await startManufacturingStack({
    adapters: ["grpc_native", "connect_bidi", "web_transport"],
    // Keycloak を有効化する（設備リモート操作は認証が必要）
    requireKeycloak: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: 設備リモート操作 v1_interactive
test.describe("Spec 01: 設備リモート操作 (v1_interactive)", () => {
  // テスト 1: セッション内シーケンス単調増加確認
  test("pl_seq_monotonic_per_session — 1000 並列オペレーション下でシーケンスが単調増加する", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // tier1 gateway に health チェックリクエストを送信する
    await page.goto(stack.gatewayUrl + "/health");
    // pl_seq_monotonic_per_session assertion を実行する
    await assertBidiSeqMonotonic(page);
  });

  // テスト 2: 切断後再接続シーケンス連続性確認
  test("rd_seq_continuous_across_resume — TLS 切断後の再接続でシーケンスが連続する", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // rd_seq_continuous_across_resume assertion を実行する
    await assertBidiResumeSeqContinuous(page);
  });

  // テスト 3: スロークライアント backpressure 確認
  test("backpressure_applied_correctly — スロークライアント時に backpressure が適用される", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // backpressure_applied_correctly assertion を実行する
    await assertBackpressureApplied(page);
  });

  // テスト 4: half-close クリーン終了確認
  test("half_close_terminates_cleanly — half-close 後にコネクションがクリーンに終了する", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // half_close_terminates_cleanly assertion を実行する
    await assertHalfCloseTerminatesCleanly(page);
  });

  // テスト 5: プロキシバッファリング注入処理確認
  test("proxy_buffer_inject_handled — プロキシバッファリング 500ms 注入を適切に処理する", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // proxy_buffer_inject_handled assertion を実行する
    await assertProxyBufferHandled(page);
  });

  // テスト 6: HLC resume_token 形式確認
  test("resume_token_hlc_valid — resume_token が有効な HLC 形式を持つ", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // resume_token_hlc_valid assertion を実行する
    await assertHlcResumeTokenValid(page);
  });

  // テスト 7: 大きなメッセージ (4MB) フラグメンテーション確認
  test("large_msg_fragmented_correctly — 4MB メッセージが正しくフラグメント化されて届く", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
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
  test("keepalive_maintained_after_idle — 5 分アイドル後も keepalive が維持される", async ({ page }) => {
    // tier1 gateway の health エンドポイントをモックする
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
