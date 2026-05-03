// src/sdk/typescript/test-fixtures/src/index.ts
//
// @k1s0/sdk-test-fixtures の public re-export entry。
//
// 設計正典:
//   ADR-TEST-010（test-fixtures 4 言語 SDK 同梱）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md

export { fixtures, Fixture, SDKClient } from './fixture.js';
export type { Options, Stack } from './options.js';
export { defaultOptions } from './options.js';
export {
  MockBuilderRoot,
  StateMockBuilder,
  AuditMockBuilder,
  PubSubMockBuilder,
  FixturePhaseUnsupportedError,
} from './mockBuilder.js';
export type { StateEntry, AuditEntry, PubSubMessage } from './mockBuilder.js';
export { waitFor, assertPodReady, WaitTimeoutError, formatWaitFailure } from './waitAssert.js';

// playwright fixture は別 export path（peerDependency 任意）
// 利用者は以下のように import:
//   import { browserContext } from '@k1s0/sdk-test-fixtures/playwright';
