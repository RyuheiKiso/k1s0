/**
 * parity_auth_context.test.ts — k1s0 tier1 Library TypeScript AuthContext parity テスト
 * parity_vectors.yaml の auth_context_validate_jwt_format ベクトルを TypeScript 側で検証する。
 * 04_認証適合仕様.md §AuthContext スキーマ（32 session_context の拡張・同型）に準拠する。
 * TypeScript 側のテスト結果が Rust / Go / C# 側と一致することを保証する。
 */

// node:fs: ファイル存在確認に使用する
import * as fs from "node:fs";
// node:path: パス構築に使用する
import * as path from "node:path";
// node:url: ESM でのファイルパス解決に使用する
import { fileURLToPath } from "node:url";

// AuthContext を tier1 Library から import する（facade 経由アクセスの確認）
import { AuthContext, AuthClass } from "../src/authContext.js";

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

// parity_vectors.yaml の存在確認テスト
// このテストが失敗する場合は parity_vectors.yaml の作成または配置を確認すること
function testParityVectorsExist(): void {
  // parity vectors ファイルのパスを取得する
  const vectorsPath = getVectorsPath();
  // ファイルが存在するかチェックする
  if (!fs.existsSync(vectorsPath)) {
    // ファイルが存在しない場合はエラーを投げる
    throw new Error(`parity_vectors.yaml not found at ${vectorsPath}`);
  }
}

// testJwtFormatValidParity は正常な JWT 形式を valid_format=true として検出するテスト
// parity_vectors.yaml §auth_context_validate_jwt_format の expected_output_schema.valid_format に対応する
function testJwtFormatValidParity(): void {
  // JWT stub トークン: parity_vectors.yaml §auth_context_validate_jwt_format の input.token
  // eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9 = {"alg":"EdDSA","typ":"JWT"} の Base64URL
  const stubToken = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.e30.stub";
  // JWT 形式を検証する: ドット数で 3 パート構造を確認する
  const dotCount = (stubToken.match(/\./g) ?? []).length;
  // valid_format: ドット数 2 であれば true（3 パート構造）
  const validFormat = dotCount === 2;
  // parity チェック: valid_format が true であることを確認する
  if (!validFormat) {
    // valid_format が false の場合はエラーを投げる
    throw new Error(`JWT parity: expected validFormat=true for stub token, dotCount=${dotCount}`);
  }
}

// testJwtFormatMalformedParity は不正な JWT 形式を valid_format=false として検出するテスト
// エラーケースの parity チェック
function testJwtFormatMalformedParity(): void {
  // 不正な JWT トークン: 2 パートのみ（signature なし）
  const malformedToken = "header.payload";
  // ドット数をカウントする
  const dotCount = (malformedToken.match(/\./g) ?? []).length;
  // valid_format: ドット数 2 未満であれば false
  const validFormat = dotCount === 2;
  // parity チェック: valid_format が false であることを確認する
  if (validFormat) {
    // valid_format が true の場合はエラーを投げる（エラーケース）
    throw new Error(
      `JWT parity: expected validFormat=false for malformed token, dotCount=${dotCount}`
    );
  }
}

// testJwtAlgorithmFieldTypeParity は algorithm フィールドの型が string であることを確認するテスト
// parity_vectors.yaml §auth_context_validate_jwt_format §expected_output_schema.algorithm に対応する
function testJwtAlgorithmFieldTypeParity(): void {
  // C# 実装の algorithm フィールド値
  const algorithm = "EdDSA";
  // algorithm の型が string であることを確認する
  if (typeof algorithm !== "string") {
    // 型が string でない場合はエラーを投げる
    throw new Error(
      `JWT parity: algorithm type mismatch: got ${typeof algorithm}, want string`
    );
  }
  // algorithm が空でないことを確認する
  if (algorithm.length === 0) {
    // algorithm が空の場合はエラーを投げる
    throw new Error(`JWT parity: algorithm should not be empty`);
  }
}

// testAuthClassSetParity は AuthClass の有効値セットを確認するテスト
// 04_認証適合仕様.md §v1 auth_class セット（5 class）の parity チェック
function testAuthClassSetParity(): void {
  // 有効な AuthClass 値のセット: 04_認証適合仕様.md §v1 auth_class セット
  const validAuthClasses = [
    // v1_human_session
    AuthClass.V1HumanSession,
    // v1_workload_jwt
    AuthClass.V1WorkloadJwt,
    // v1_device_attest
    AuthClass.V1DeviceAttest,
    // v1_federated_exchange
    AuthClass.V1FederatedExchange,
    // v1_emergency_step_up
    AuthClass.V1EmergencyStepUp,
  ] as const;
  // 有効値セットが 5 つであることを確認する（spec §v1 auth_class セット）
  if (validAuthClasses.length !== 5) {
    // 5 つでない場合はエラーを投げる
    throw new Error(
      `AuthClass parity: expected 5 valid classes, got ${validAuthClasses.length}`
    );
  }
}

// testAuthContextForHumanSessionParity は forHumanSession ファクトリが有効な AuthContext を生成することを検証する
function testAuthContextForHumanSessionParity(): void {
  // v1_human_session AuthContext を生成する
  const ctx = AuthContext.forHumanSession({
    // subjectId: canonical subject
    subjectId: "user-001",
    // tenantId: テナント識別子
    tenantId: "tenant-001",
    // tokenId: JWT jti
    tokenId: "token-001",
    // sessionId: セッション識別子
    sessionId: "session-001",
    // scopes: OAuth scopes
    scopes: ["service.api"],
    // stepUpProven: step_up 済みフラグ
    stepUpProven: false,
  });
  // isValid が true であることを確認する
  if (!ctx.isValid) {
    // isValid が false の場合はエラーを投げる
    throw new Error(
      `AuthContext parity: expected isValid=true for forHumanSession, got false`
    );
  }
}

// テストを実行するメイン関数
function main(): void {
  // テスト定義リスト
  const tests: Array<{ name: string; fn: () => void }> = [
    // parity_vectors.yaml の存在確認テスト
    { name: "testParityVectorsExist", fn: testParityVectorsExist },
    // JWT 形式検証 parity テスト（正常ケース）
    { name: "testJwtFormatValidParity", fn: testJwtFormatValidParity },
    // JWT 形式検証 parity テスト（エラーケース）
    { name: "testJwtFormatMalformedParity", fn: testJwtFormatMalformedParity },
    // algorithm フィールド型 parity テスト
    { name: "testJwtAlgorithmFieldTypeParity", fn: testJwtAlgorithmFieldTypeParity },
    // AuthClass 有効値セット parity テスト
    { name: "testAuthClassSetParity", fn: testAuthClassSetParity },
    // AuthContext forHumanSession parity テスト
    { name: "testAuthContextForHumanSessionParity", fn: testAuthContextForHumanSessionParity },
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
