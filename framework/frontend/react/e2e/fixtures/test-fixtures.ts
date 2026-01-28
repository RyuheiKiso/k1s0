import { test as base, expect, Page } from '@playwright/test';

/**
 * k1s0 E2E テスト共通フィクスチャ
 */

// テストユーザー情報
export interface TestUser {
  email: string;
  password: string;
  name: string;
}

// テストフィクスチャの型定義
export interface TestFixtures {
  // 認証済みページ
  authenticatedPage: Page;
  // テストユーザー
  testUser: TestUser;
}

// 拡張されたテストオブジェクト
export const test = base.extend<TestFixtures>({
  // テストユーザー
  testUser: async ({}, use) => {
    const user: TestUser = {
      email: 'test@example.com',
      password: 'Test@Password123',
      name: 'Test User',
    };
    await use(user);
  },

  // 認証済みページ
  authenticatedPage: async ({ page, testUser }, use) => {
    // ログインページに移動
    await page.goto('/login');

    // ログインフォームに入力
    await page.getByLabel('メールアドレス').fill(testUser.email);
    await page.getByLabel('パスワード').fill(testUser.password);

    // ログインボタンをクリック
    await page.getByRole('button', { name: 'ログイン' }).click();

    // ダッシュボードに遷移するまで待機
    await page.waitForURL('**/dashboard', { timeout: 10000 });

    await use(page);
  },
});

export { expect };

/**
 * ページオブジェクトのベースクラス
 */
export abstract class BasePage {
  constructor(protected readonly page: Page) {}

  // ページへの遷移
  abstract goto(): Promise<void>;

  // ページのロード完了を待機
  async waitForLoad(): Promise<void> {
    await this.page.waitForLoadState('networkidle');
  }

  // スクリーンショットを撮影
  async screenshot(name: string): Promise<void> {
    await this.page.screenshot({ path: `screenshots/${name}.png`, fullPage: true });
  }
}

/**
 * ログインページオブジェクト
 */
export class LoginPage extends BasePage {
  async goto(): Promise<void> {
    await this.page.goto('/login');
    await this.waitForLoad();
  }

  async login(email: string, password: string): Promise<void> {
    await this.page.getByLabel('メールアドレス').fill(email);
    await this.page.getByLabel('パスワード').fill(password);
    await this.page.getByRole('button', { name: 'ログイン' }).click();
  }

  async expectError(message: string): Promise<void> {
    const error = this.page.getByRole('alert');
    await expect(error).toBeVisible();
    await expect(error).toContainText(message);
  }

  async expectSuccess(): Promise<void> {
    await this.page.waitForURL('**/dashboard', { timeout: 10000 });
  }
}

/**
 * ダッシュボードページオブジェクト
 */
export class DashboardPage extends BasePage {
  async goto(): Promise<void> {
    await this.page.goto('/dashboard');
    await this.waitForLoad();
  }

  async expectWelcomeMessage(name: string): Promise<void> {
    const welcome = this.page.getByRole('heading', { level: 1 });
    await expect(welcome).toContainText(name);
  }

  async navigateTo(menuItem: string): Promise<void> {
    await this.page.getByRole('navigation').getByText(menuItem).click();
  }

  async logout(): Promise<void> {
    await this.page.getByRole('button', { name: 'ログアウト' }).click();
    await this.page.waitForURL('**/login');
  }
}

/**
 * ナビゲーションヘルパー
 */
export class NavigationHelper {
  constructor(private readonly page: Page) {}

  async clickNavLink(text: string): Promise<void> {
    await this.page.getByRole('navigation').getByRole('link', { name: text }).click();
  }

  async expectCurrentPath(path: string): Promise<void> {
    await expect(this.page).toHaveURL(new RegExp(`.*${path}$`));
  }

  async goBack(): Promise<void> {
    await this.page.goBack();
  }

  async goForward(): Promise<void> {
    await this.page.goForward();
  }
}

/**
 * フォームヘルパー
 */
export class FormHelper {
  constructor(private readonly page: Page) {}

  async fillInput(label: string, value: string): Promise<void> {
    await this.page.getByLabel(label).fill(value);
  }

  async selectOption(label: string, value: string): Promise<void> {
    await this.page.getByLabel(label).selectOption(value);
  }

  async checkCheckbox(label: string): Promise<void> {
    await this.page.getByLabel(label).check();
  }

  async uncheckCheckbox(label: string): Promise<void> {
    await this.page.getByLabel(label).uncheck();
  }

  async submitForm(): Promise<void> {
    await this.page.getByRole('button', { name: /送信|保存|確定/ }).click();
  }
}
