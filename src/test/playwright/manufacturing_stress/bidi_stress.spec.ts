// @conformanceAssert(["bidi_seq_monotonic_per_session", "bidi_resume_seq_continuous",
//   "backpressure_applied_correctly", "half_close_terminates_cleanly",
//   "proxy_buffer_handled", "hlc_resume_token_valid",
//   "large_msg_fragmented_correctly", "keepalive_after_idle"])
// bidi_stress.spec.ts — 01_Bidi 適合仕様 §120-158 fault chaos / stress verification
// tier3 v1_fault_chaos: 接続切断・再接続・大メッセージ断片化・keepalive の bidi streaming 不変条件を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "./support/containers";
import { startManufacturingStack } from "./support/containers";
// bidi_assertions から全 8 assertion をインポートする
import {
  assertBidiSeqMonotonic,
  assertBidiResumeSeqContinuous,
  assertBackpressureApplied,
  assertHalfCloseTerminatesCleanly,
  assertProxyBufferHandled,
  assertHlcResumeTokenValid,
  assertLargeMsgFragmented,
  assertKeepaliveAfterIdle,
} from "./support/assertions/bidi_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に bidi stress 向けスタックを起動する
test.beforeAll(async () => {
  // gRPC bidi streaming アダプターを有効化したスタックを起動する
  stack = await startManufacturingStack({
    adapters: ["grpc_native", "http2_bridge"],
    requireKafka: false,
    enableRls: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: Bidi stress — fault chaos シナリオ全 8 種
test.describe("bidi_stress — 01_Bidi 適合仕様 fault chaos 全 8 conformance_assert", () => {
  // テスト 1: セッション内シーケンス単調増加確認
  test("bidi_seq_monotonic_per_session — 並列 1000 ops でシーケンス番号が単調増加する", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動してページコンテキストを初期化する
    await page.goto(stack.gatewayUrl + "/health");
    // bidi シーケンス単調増加 assertion を実行する
    await assertBidiSeqMonotonic(page);
  });

  // テスト 2: 再接続後のシーケンス継続性確認
  test("bidi_resume_seq_continuous — 接続切断後の再接続でシーケンスが連続する", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // bidi 再接続後のシーケンス継続性 assertion を実行する
    await assertBidiResumeSeqContinuous(page);
  });

  // テスト 3: 高負荷時の backpressure 適用確認
  test("backpressure_applied_correctly — ストリーム高負荷時に backpressure が適用される", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // backpressure 適用 assertion を実行する
    await assertBackpressureApplied(page);
  });

  // テスト 4: half-close による正常終了確認
  test("half_close_terminates_cleanly — half-close でストリームが正常終了する", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // half-close 正常終了 assertion を実行する
    await assertHalfCloseTerminatesCleanly(page);
  });

  // テスト 5: プロキシバッファリング処理確認
  test("proxy_buffer_handled — プロキシ経由でバッファリングが正しく処理される", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // プロキシバッファ処理 assertion を実行する
    await assertProxyBufferHandled(page);
  });

  // テスト 6: HLC resume token の有効性確認
  test("hlc_resume_token_valid — 再接続時の HLC resume token が有効である", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // HLC resume token 有効性 assertion を実行する
    await assertHlcResumeTokenValid(page);
  });

  // テスト 7: 大メッセージの断片化処理確認
  test("large_msg_fragmented_correctly — 大メッセージが正しく断片化・再組立てされる", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // 大メッセージ断片化 assertion を実行する
    await assertLargeMsgFragmented(page);
  });

  // テスト 8: アイドル後の keepalive 確認
  test("keepalive_after_idle — アイドル状態からの keepalive でストリームが維持される", async ({ page }) => {
    // gateway health エンドポイントへのルートを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health チェックに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // gateway の health エンドポイントに移動する
    await page.goto(stack.gatewayUrl + "/health");
    // keepalive after idle assertion を実行する
    await assertKeepaliveAfterIdle(page);
  });
});
