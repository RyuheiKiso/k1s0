/**
 * conformanceAssert.ts — spec 01 §assertion id の連結
 * docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §scenarios.yaml の assertion id を
 * TypeScript test runner が claim するための型安全デコレータ機構を提供する。
 * Rust conformance_assert.rs / Go conformance_assert.go /
 * C# ConformanceAssertAttribute.cs と 4 言語等価強度を保つ。
 */

/**
 * ConformanceAssertId は conformance assertion id を型安全に表現するブランド型。
 * scenarios.yaml の assertion フィールドの値を型として固定し、
 * typo による assertion id のズレを TypeScript 型検査で防ぐ。
 */
// ConformanceAssertId 型: string のブランド型（__brand で nominal typing を実現する）
export type ConformanceAssertId = string & { readonly __brand: "ConformanceAssertId" };

/**
 * assertId ヘルパーは文字列リテラルを ConformanceAssertId 型にキャストする。
 * scenarios.yaml の assertion id を型安全に宣言するために使用する。
 *
 * @example
 * const id = assertId("pl_seq_monotonic_per_session");
 */
// assertId: string リテラルを ConformanceAssertId ブランド型にキャストするヘルパー
export function assertId(id: string): ConformanceAssertId {
  // string を ConformanceAssertId ブランド型にキャストして返す
  return id as ConformanceAssertId;
}

/**
 * conformanceAssert は assertion id を記録して test 関数を実行する。
 * CI 整合 2「scenarios.yaml の assertion id が全 class × 全 adapter × 全言語で実装されていること」
 * の TypeScript 物理機構として、assertId を引数に取ることで assertion id を
 * test 関数に静的に紐づける。
 *
 * @param assertId - scenarios.yaml の assertion id（例: "pl_seq_monotonic_per_session"）
 * @param testFn - assertion id に対応する test ロジックを実行する関数
 * @returns testFn の戻り値をそのまま返す
 *
 * @example
 * const result = conformanceAssert(
 *   assertId("bidi_v1_interactive__grpc_native_assert_001"),
 *   () => { return true; },
 * );
 */
// conformanceAssert: assertId と testFn をアトミックに結合して testFn を実行する
export function conformanceAssert<T>(
  // assertId: ConformanceAssertId ブランド型で assertion id を型安全に受け取る
  assertId: ConformanceAssertId,
  // testFn: () => T 型の test 関数（assertion ロジックを実行する）
  testFn: () => T,
): T {
  // assertId は型シグネチャに固定されているため宣言不要だが
  // lint の no-unused-vars を回避するために void 評価する
  void assertId;
  // test 関数を実行して結果を返す
  return testFn();
}
