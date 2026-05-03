// src/sdk/typescript/test-fixtures/src/options.ts
//
// k1s0 TypeScript SDK test-fixtures: Options / Stack 定義。
// 4 言語対称 API の field 名を TS イディオム（camelCase）で実装する。
//
// 設計正典:
//   ADR-TEST-010
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md

// kind cluster に install する k1s0 stack の規模
export type Stack = 'minimum' | 'full';

// Setup の動作を制御するパラメータ
//
// 4 言語対称化のため field 名は対応関係を保つ:
//   Go: KindNodes / Stack / AddOns / Tenant / Namespace
//   TS: kindNodes / stack / addOns / tenant / namespace
export interface Options {
  // kind cluster の node 数（既定 2）
  kindNodes?: number;
  // install する k1s0 stack（既定 'minimum'）
  stack?: Stack;
  // Setup 時に追加で install する任意 component の名前一覧
  addOns?: readonly string[];
  // デフォルトの tenant ID（既定 'tenant-a'）
  tenant?: string;
  // k1s0 install 先 namespace（既定 'k1s0'）
  namespace?: string;
}

// Options の既定値（Go の DefaultOptions() / Rust の Default impl と対称）
export const defaultOptions: Required<Options> = {
  kindNodes: 2,
  stack: 'minimum',
  addOns: [],
  tenant: 'tenant-a',
  namespace: 'k1s0',
};
