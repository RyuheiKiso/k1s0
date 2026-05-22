// @conformanceAssert(["bulk_upload_at_least_once_idempotent",
//   "backpressure_applied_correctly", "half_close_terminates_cleanly",
//   "large_msg_fragmented_correctly"])
// Spec 04: SCADA テレメトリ収集 (v1_bulk_upload) — 01_Bidi 適合仕様 §158 ship blocker 4/9
// 工場 SCADA システムからのテレメトリ bulk upload に対する assertion を検証する

// @playwright/test から test をインポートする
import { test } from "@playwright/test";
// support/containers からスタック起動ヘルパーをインポートする
import type { ManufacturingStack } from "../support/containers";
import { startManufacturingStack } from "../support/containers";
// slo_assertions から bulk_upload assertion をインポートする
import { assertBulkUploadIdempotent } from "../support/assertions/slo_assertions";
// bidi_assertions から v1_bulk_upload 対応 assertion をインポートする
import {
  assertBackpressureApplied,
  assertHalfCloseTerminatesCleanly,
  assertLargeMsgFragmented,
} from "../support/assertions/bidi_assertions";

// テスト全体で共有するスタックインスタンスを保持する変数
let stack: ManufacturingStack;

// 全テスト開始前に bulk_upload 向けスタックを起動する
test.beforeAll(async () => {
  // Kafka と PostgreSQL を有効化したスタックを起動する（テレメトリは大量データ）
  stack = await startManufacturingStack({
    adapters: ["grpc_native", "messaging_bridge"],
    requireKafka: true,
    enableRls: true,
  });
});

// 全テスト終了後にスタックを停止する
test.afterAll(async () => {
  // コンテナリソースを解放する
  await stack.stop();
});

// テストの説明ブロック: SCADA テレメトリ収集 v1_bulk_upload
test.describe("Spec 04: SCADA テレメトリ収集 (v1_bulk_upload)", () => {
  // テスト 1: at-least-once 冪等性確認
  test("bulk_upload_at_least_once_idempotent — SCADA テレメトリが重複送信されても冪等に受信される", async ({ page }) => {
    // gateway モックを設定する
    await page.route(`${stack.gatewayUrl}/health`, async route => {
      // health エンドポイントに OK を返す
      await route.fulfill({ status: 200, body: "ok" });
    });
    // health エンドポイントに移動してページコンテキストを準備する
    await page.goto(stack.gatewayUrl + "/health");
    // bulk_upload_at_least_once_idempotent assertion を実行する
    await assertBulkUploadIdempotent(page);
  });

  // テスト 2: Kafka quota throttle 下での backpressure 確認
  test("backpressure_applied_correctly — Kafka quota throttle 時にバックプレッシャーが適用される", async ({ page }) => {
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

  // テスト 3: bulk upload 完了後の half-close クリーン終了確認
  test("half_close_terminates_cleanly — SCADA バッチ転送完了後にコネクションがクリーンに終了する", async ({ page }) => {
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

  // テスト 4: 大きなテレメトリペイロードのフラグメンテーション確認
  test("large_msg_fragmented_correctly — SCADA の大きなテレメトリペイロードが正しくフラグメント化される", async ({ page }) => {
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
});
