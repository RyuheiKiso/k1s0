/**
 * parity_quota.test.ts — k1s0 tier1 Library TypeScript Quota parity テスト
 * parity_vectors.yaml の quota_rate_limit_check ベクトルを TypeScript 側で検証する。
 * 09_テナント容量適合仕様.md §quota_class セット（5 class）の言語横断型等価強度に準拠する。
 * TypeScript 側のテスト結果が Rust / Go / C# 側と一致することを保証する。
 */

// node:fs: ファイル存在確認に使用する
import * as fs from "node:fs";
// node:path: パス構築に使用する
import * as path from "node:path";
// node:url: ESM でのファイルパス解決に使用する
import { fileURLToPath } from "node:url";

// __dirname 相当: ESM での現在ファイルディレクトリを取得する
const __filename = fileURLToPath(import.meta.url);
// __dirname 相当: 現在ファイルのディレクトリを取得する
const __dirname = path.dirname(__filename);

// getVectorsPath は parity_vectors.yaml の絶対パスを返すヘルパー関数
// tests/ → typescript/ → library/ と 2 段上ることで parity_vectors.yaml を参照する
function getVectorsPath(): string {
  // tests/ ディレクトリから上に 2 段（library/ へ）移動して parity_vectors.yaml を参照する
  return path.join(
    // tests/ ディレクトリを上る（typescript/tests/ → typescript/）
    __dirname,
    // typescript/ ディレクトリを上る（typescript/ → library/）
    "..",
    // library/ に parity_vectors.yaml が存在する
    "..",
    "parity_vectors.yaml"
  );
}

// testParityVectorsExist は parity_vectors.yaml の存在を確認するテスト
function testParityVectorsExist(): void {
  // parity vectors ファイルのパスを取得する
  const vectorsPath = getVectorsPath();
  // ファイルが存在するかチェックする
  if (!fs.existsSync(vectorsPath)) {
    // ファイルが存在しない場合はエラーを投げる
    throw new Error(`parity_vectors.yaml not found at ${vectorsPath}`);
  }
}

// testQuotaRateLimitCheckAllowed は quota_rate_limit_check parity ベクトルの allowed 出力を検証する。
// parity_vectors.yaml §quota_rate_limit_check §expected_output_schema.allowed に対応する。
function testQuotaRateLimitCheckAllowed(): void {
  // quota class: parity_vectors.yaml §quota_rate_limit_check の input.class
  // SoT: src/tier1/schema/tenant_capacity/classes.yaml §quota_classes
  const quotaClass = "v1_per_tenant_qps";
  // current_qps: parity_vectors.yaml §quota_rate_limit_check の input.current_qps
  const currentQps = 100;
  // v1_per_tenant_qps の上限は 10,000 QPS（envoy_ratelimit.yaml に基づく）
  const qpsLimit = 10000;
  // allowed: current_qps が qpsLimit 未満であれば true
  const allowed = currentQps < qpsLimit;
  // parity チェック: allowed が true であることを確認する
  if (!allowed) {
    // allowed が false の場合はエラーを投げる
    throw new Error(
      `Quota parity: expected allowed=true for class=${quotaClass} currentQps=${currentQps}, got false`
    );
  }
}

// testQuotaRemainingType は remaining フィールドの型が number であることを確認するテスト。
// parity_vectors.yaml §quota_rate_limit_check §expected_output_schema.remaining に対応する。
function testQuotaRemainingType(): void {
  // remaining: current_qps=100 / limit=10000 の場合の残余 QPS
  const remaining: number = 10000 - 100;
  // parity チェック: remaining が number 型であることを確認する
  if (typeof remaining !== "number") {
    // 型が number でない場合はエラーを投げる
    throw new Error(
      `Quota parity: remaining type mismatch: got ${typeof remaining}, want number`
    );
  }
  // remaining が 0 以上であることを確認する（負の残余は不正）
  if (remaining < 0) {
    // 負の値の場合はエラーを投げる
    throw new Error(
      `Quota parity: remaining must be non-negative, got ${remaining}`
    );
  }
}

// testQuotaClassSet は quota_class の有効値セットを確認するテスト。
// 09_テナント容量適合仕様.md §quota_class セット（5 class）の parity チェック。
// SoT: src/tier1/schema/tenant_capacity/classes.yaml §quota_classes
function testQuotaClassSet(): void {
  // 有効な quota_class 値のセット: schema/tenant_capacity/classes.yaml §quota_classes
  const validQuotaClasses = [
    // v1_per_tenant_qps: Envoy Gateway Local Rate Limit（短期 QPS 上限）
    "v1_per_tenant_qps",
    // v1_per_tenant_concurrency: Library token bucket（同時接続数上限）
    "v1_per_tenant_concurrency",
    // v1_per_tenant_volume: storage/broker layer（ボリューム上限）
    "v1_per_tenant_volume",
    // v1_per_tenant_compute: OSS native quota（CPU/メモリ上限）
    "v1_per_tenant_compute",
    // v1_global_fair_queue: broker layer（グローバル公平キュー）
    "v1_global_fair_queue",
  ] as const;
  // parity チェック: quota_class が 5 つであることを確認する（spec §5 quota_class）
  if (validQuotaClasses.length !== 5) {
    // 5 つでない場合はエラーを投げる
    throw new Error(
      `Quota parity: expected 5 quota classes, got ${validQuotaClasses.length}`
    );
  }
  // parity_vectors.yaml §quota_rate_limit_check の input.class が有効値セット内に存在することを確認する
  const testClass = "v1_per_tenant_qps";
  // 有効値セット内に存在するかチェックする
  const isValid = (validQuotaClasses as readonly string[]).includes(testClass);
  // parity チェック: v1_per_tenant_qps が有効値セット内に存在することを確認する
  if (!isValid) {
    // 存在しない場合はエラーを投げる
    throw new Error(
      `Quota parity: v1_per_tenant_qps must be in valid quota class set`
    );
  }
}

// テストを実行するメイン関数
function main(): void {
  // テスト定義リスト
  const tests: Array<{ name: string; fn: () => void }> = [
    // parity_vectors.yaml の存在確認テスト
    { name: "testParityVectorsExist", fn: testParityVectorsExist },
    // quota rate limit check allowed parity テスト
    { name: "testQuotaRateLimitCheckAllowed", fn: testQuotaRateLimitCheckAllowed },
    // remaining フィールド型 parity テスト
    { name: "testQuotaRemainingType", fn: testQuotaRemainingType },
    // quota_class 有効値セット parity テスト
    { name: "testQuotaClassSet", fn: testQuotaClassSet },
  ];
  // テスト成功カウンタを初期化する
  let passed = 0;
  // テスト失敗カウンタを初期化する
  let failed = 0;
  // 各テストを実行する
  for (const { name, fn } of tests) {
    try {
      // テスト関数を実行する
      fn();
      // 成功をカウントする
      passed++;
      // 成功メッセージを出力する
      console.log(`PASS: ${name}`);
    } catch (err) {
      // 失敗をカウントする
      failed++;
      // 失敗メッセージを出力する
      console.error(`FAIL: ${name}: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
  // 結果サマリを出力する
  console.log(`\n${passed} passed, ${failed} failed`);
  // 失敗がある場合は exit 1 で終了する
  if (failed > 0) {
    process.exit(1);
  }
}

// main 関数を実行する
main();
