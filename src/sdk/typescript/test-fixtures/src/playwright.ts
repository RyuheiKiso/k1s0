// src/sdk/typescript/test-fixtures/src/playwright.ts
//
// k1s0 TypeScript SDK test-fixtures: Playwright fixture（領域 5、TS のみ提供、ADR-TEST-010 §3）。
//
// 利用者は tier3 web frontend を Playwright で test する時に本 fixture を import し、
// Keycloak OIDC handshake を fixture が肩代わりした認証済 BrowserContext を取得する。
//
// owner suite の tier3-web 検証は Go test + chromedp で別経路（ADR-TEST-008 §7 二重提供）。
//
// Playwright の dependency は peerDependency なので、利用者の package.json で
// `@playwright/test` を install していない場合は本 module を import しても compile error
// になる（peerDependencyMeta.optional: true で warning のみ）。

import type { Fixture } from './fixture.js';

// Playwright 統合の Options（既存 Fixture Options に追加）
export interface BrowserContextOptions {
  // tenant ID（Keycloak realm の k1s0 tenant claim に inject）
  tenant: string;
  // user ID（Keycloak user で実 login する時の subject）
  user: string;
  // 追加 cookie / localStorage（任意）
  storage?: Record<string, string>;
}

// browserContext は Keycloak OIDC handshake 済の BrowserContext を返す。
// 採用初期で @playwright/test の chromium.launch + Keycloak login flow を実装する。
//
// 戻り値は Playwright の BrowserContext 型だが、peerDependency 不在時に compile error を避けるため
// `unknown` を返す（利用者側で `as BrowserContext` キャストする想定）。
export async function browserContext(
  _fixture: Fixture,
  _opts: BrowserContextOptions,
): Promise<unknown> {
  // skeleton: 採用初期で以下を実装
  //   1. chromium.launch({ headless: true })
  //   2. context = browser.newContext()
  //   3. Keycloak token endpoint で OIDC token 取得（helpers.keycloakClient.passwordGrantToken）
  //   4. context.addCookies / context.storageState で token を inject
  //   5. context を return
  return null;
}

// 利用者の使い方（README の typical usage）:
//
//   import { fixtures } from '@k1s0/sdk-test-fixtures';
//   import { browserContext } from '@k1s0/sdk-test-fixtures/playwright';
//   import { test } from '@playwright/test';
//
//   test('tier3-web tenant onboarding', async () => {
//     const fx = await fixtures.setup({ stack: 'minimum', addOns: ['workflow'] });
//     const ctx = await browserContext(fx, { tenant: 'tenant-a', user: 'alice' });
//     // 利用者の Playwright test code
//     await fx.teardown();
//   });
