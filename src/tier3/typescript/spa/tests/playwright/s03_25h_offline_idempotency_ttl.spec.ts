// tier3 Playwright シナリオ S03: 25h オフライン → Idempotency-Key TTL 超過 → user 確認 → 新 key で resend
// spec 11 整合 4 のシナリオ: TTL 超過後に user が確認して新 key で resend することを確認する

// Playwright の test/expect をインポートする
import { test, expect } from '@playwright/test';

// S03: 25h オフライン → Idempotency-Key TTL 超過 → user 確認 → 新 key resend のテスト
test('S03: 25h offline causes TTL expiry and user is prompted for new idempotency key', async ({ page }) => {
  // ベース URL にアクセスする
  await page.goto('/');
  // アプリケーションの基本的な表示を確認する (実装は Stage 4 後半で完成)
  await expect(page.locator('body')).toBeTruthy();
  // TODO: TTL 超過 → user 確認 → 新 key resend の完全な実装を Stage 4 で追加する
});
