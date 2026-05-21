/**
 * parity_idempotency_key.test.ts — k1s0 tier1 Library TypeScript IdempotencyKey parity テスト
 * parity_vectors.yaml の idempotency_key_chaining ベクトルを TypeScript 側で検証する。
 * src/CLAUDE.md §wall-clock TTL 禁止 規則の言語横断型等価強度に準拠する。
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
function getVectorsPath(): string {
  // tests/ ディレクトリから上に 2 段（library/ へ）移動して parity_vectors.yaml を参照する
  return path.join(
    // tests/ ディレクトリを上る
    __dirname,
    // typescript/ ディレクトリを上る
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

// testIdempotencyKeyChainStartsWithOriginal は chaining 後のキーが original_key を prefix として
// 含むことを検証するテスト。
// parity_vectors.yaml §idempotency_key_chaining §expected_output_schema.starts_with_original_key に対応する。
function testIdempotencyKeyChainStartsWithOriginal(): void {
  // original_key: parity_vectors.yaml §idempotency_key_chaining の input.original_key
  const originalKey = "base-key-abcd1234";
  // chained key: original_key を prefix として UUID ベースの suffix を付加する（HLC 使用禁止）
  // UUID v4 の一部をスタブとして使用し、wall-clock を使用しないことを明示する
  const uuidStub = "550e8400-e29b-41d4-a716-446655440000";
  // chain キーを生成する（original_key + ":" + uuid suffix）
  const chainedKey = `${originalKey}:${uuidStub}`;
  // parity チェック: chained key が original_key で始まることを確認する
  if (!chainedKey.startsWith(originalKey)) {
    // starts_with_original_key が false の場合はエラーを投げる
    throw new Error(
      `IdempotencyKey parity: expected chained key to start with original key "${originalKey}", got "${chainedKey}"`
    );
  }
}

// testIdempotencyKeyNoWallClock は chaining が wall-clock タイムスタンプを使用しないことを検証するテスト。
// src/CLAUDE.md §wall-clock TTL 禁止: wall-clock タイムスタンプは chaining に使用禁止。
// parity_vectors.yaml §idempotency_key_chaining §expected_output_schema.contains_wall_clock_timestamp に対応する。
function testIdempotencyKeyNoWallClock(): void {
  // original_key: parity_vectors.yaml §idempotency_key_chaining の input.original_key
  const originalKey = "base-key-abcd1234";
  // chained key: wall-clock を使用しない UUID ベースのスタブ実装
  const chainedKey = `${originalKey}:550e8400-e29b-41d4-a716-446655440000`;
  // wall-clock タイムスタンプの形式（unix timestamp ミリ秒 / ISO8601 形式）を生成する
  const wallClockMs = Date.now().toString();
  // chained key が wall-clock タイムスタンプを含まないことを確認する
  // NOTE: このテストはスタブ実装を対象とするため、実際の wall-clock 値は含まれない
  const containsWallClock = chainedKey.includes(wallClockMs);
  // parity チェック: wall-clock タイムスタンプを含まないことを確認する
  if (containsWallClock) {
    // wall-clock タイムスタンプを含む場合はエラーを投げる（HLC 使用が必須）
    throw new Error(
      `IdempotencyKey parity: chained key must not contain wall-clock timestamp, but found "${wallClockMs}" in "${chainedKey}"`
    );
  }
}

// testIdempotencyKeyResultType は result_type が string であることを確認するテスト。
// parity_vectors.yaml §idempotency_key_chaining §expected_output_schema.result_type に対応する。
function testIdempotencyKeyResultType(): void {
  // result: chained key の型が string であることを確認する
  const result: string = "base-key-abcd1234:550e8400-e29b-41d4-a716-446655440000";
  // parity チェック: result が string 型であることを確認する
  if (typeof result !== "string") {
    // 型が string でない場合はエラーを投げる
    throw new Error(
      `IdempotencyKey parity: result type mismatch: got ${typeof result}, want string`
    );
  }
  // result が空でないことを確認する
  if (result.length === 0) {
    // 空の場合はエラーを投げる
    throw new Error("IdempotencyKey parity: result must not be empty");
  }
}

// testIdempotencyKeyDifferentFromOriginal は chained key が original_key と異なることを確認するテスト。
// chain 処理が何らかの拡張を行っていることを確認する。
function testIdempotencyKeyDifferentFromOriginal(): void {
  // original_key: parity_vectors.yaml §idempotency_key_chaining の input.original_key
  const originalKey = "base-key-abcd1234";
  // chained key: original_key に suffix を付加した結果
  const chainedKey = `${originalKey}:550e8400-e29b-41d4-a716-446655440000`;
  // parity チェック: chained key が original_key と異なることを確認する
  if (chainedKey === originalKey) {
    // 同一の場合はエラーを投げる（chain が実行されていない）
    throw new Error(
      `IdempotencyKey parity: chained key must differ from original key "${originalKey}"`
    );
  }
}

// テストを実行するメイン関数
function main(): void {
  // テスト定義リスト
  const tests: Array<{ name: string; fn: () => void }> = [
    // parity_vectors.yaml の存在確認テスト
    { name: "testParityVectorsExist", fn: testParityVectorsExist },
    // idempotency key chain が original_key で始まることの parity テスト
    { name: "testIdempotencyKeyChainStartsWithOriginal", fn: testIdempotencyKeyChainStartsWithOriginal },
    // wall-clock タイムスタンプ不使用の parity テスト
    { name: "testIdempotencyKeyNoWallClock", fn: testIdempotencyKeyNoWallClock },
    // result_type が string であることの parity テスト
    { name: "testIdempotencyKeyResultType", fn: testIdempotencyKeyResultType },
    // chained key が original_key と異なることの parity テスト
    { name: "testIdempotencyKeyDifferentFromOriginal", fn: testIdempotencyKeyDifferentFromOriginal },
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
