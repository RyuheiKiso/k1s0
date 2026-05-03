// src/sdk/typescript/test-fixtures/src/waitAssert.ts
//
// k1s0 TypeScript SDK test-fixtures: Wait / Assertion helper（領域 4、ADR-TEST-010 §3）。
// failure 時のエラーメッセージは 4 言語共通フォーマット:
//   [k1s0-test-fixtures] WaitFor "<resource>" timeout after Ns

import type { Fixture } from './fixture.js';

// 共通 failure フォーマット（4 言語対称）
export function formatWaitFailure(resource: string, timeoutMs: number): string {
  return `[k1s0-test-fixtures] WaitFor "${resource}" timeout after ${Math.floor(timeoutMs / 1000)}s`;
}

// timeout error class（4 言語対称、Rust の FixtureError::WaitTimeout / .NET の例外と対称）
export class WaitTimeoutError extends Error {
  constructor(
    readonly resource: string,
    readonly timeoutMs: number,
  ) {
    super(formatWaitFailure(resource, timeoutMs));
    this.name = 'WaitTimeoutError';
  }
}

// 指定 resource が ready になるまで polling 待機
// 採用初期で k8s API client（@kubernetes/client-node 等）経由で実装
export async function waitFor(
  _fixture: Fixture,
  resource: string,
  timeoutMs: number,
): Promise<void> {
  // skeleton: リリース時点は即時 return（test code が成立する）
  // 採用初期で polling + WaitTimeoutError throw を実装
  void resource;
  void timeoutMs;
}

// Pod が Ready condition を持つか assert（採用初期で k8s client wrapper として実装）
export async function assertPodReady(
  _fixture: Fixture,
  ns: string,
  podName: string,
): Promise<void> {
  void ns;
  void podName;
}

// fixtures namespace に Wait / Assert helper を追加
// 利用者は `import { fixtures } from '@k1s0/sdk-test-fixtures'` で `fixtures.waitFor(...)` を呼べる。
declare module './fixture.js' {
  interface Fixture {
    waitFor(resource: string, timeoutMs: number): Promise<void>;
    assertPodReady(ns: string, podName: string): Promise<void>;
  }
}
