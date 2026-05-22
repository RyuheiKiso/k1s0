// @conformanceAssert(["rd_seq_continuous_across_resume", "proxy_buffer_inject_handled",
//   "keepalive_maintained_after_idle", "resume_token_hlc_valid",
//   "alert_replay_after_disconnect", "alert_200ms_lag_within"])
// Spec 08: 警報配信 (v1_alert) — 01_Bidi 適合仕様 §158 ship blocker 8/9
// 製造設備の異常検知アラートをリアルタイムかつ SLO 200ms 以内で配信する assertion を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// slo_assertions から v1_alert assertion をインポートする
import {
  assertAlertReplayAfterDisconnect,
  assertAlert200msLag,
} from "../support/assertions/slo_assertions";
// bidi_assertions から v1_alert 対応 assertion をインポートする
import {
  assertBidiResumeSeqContinuous,
  assertProxyBufferHandled,
  assertKeepaliveAfterIdle,
  assertHlcResumeTokenValid,
} from "../support/assertions/bidi_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に警報配信向けスタックを起動する
test.beforeAll(async () => {
  // Prometheus + AlertManager を有効化したスタックを起動する
  stack = await startManufacturingStack({
    adapters: ["sse_paired", "webhook"],
    requirePrometheus: true,
    requireKeycloak: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: 警報配信 v1_alert
test.describe("Spec 08: 警報配信 (v1_alert)", () => {
  // テスト 1: 切断後シーケンス連続性確認 (警報は欠損 0 が必須)
  test("rd_seq_continuous_across_resume — ネットワーク切断後にアラートが連続して届く", async ({ page }) => {
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

  // テスト 2: プロキシバッファリング処理確認 (警報は遅延に敏感)
  test("proxy_buffer_inject_handled — プロキシ経由でも警報が遅延なく届く", async ({ page }) => {
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

  // テスト 3: アイドル後 keepalive 維持確認 (警報は 24x7 監視が前提)
  test("keepalive_maintained_after_idle — 深夜のアイドル期間も警報接続が維持される", async ({ page }) => {
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

  // テスト 4: HLC resume_token 確認 (警報は TLS disconnect 後も再送が必要)
  test("resume_token_hlc_valid — 警報の resume_token が有効な HLC 形式を持つ", async ({ page }) => {
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

  // テスト 5: 切断後アラート replay 確認
  test("alert_replay_after_disconnect — 切断中に発生した未受信アラートが再接続後に replay される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // alert_replay_after_disconnect assertion を実行する
    await assertAlertReplayAfterDisconnect(page);
  });

  // テスト 6: 200ms 配信遅延 SLO 確認
  test("alert_200ms_lag_within — 緊急アラートの配信遅延が 200ms 以内に収まる", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // alert_200ms_lag_within assertion を実行する
    await assertAlert200msLag(page);
  });
});
