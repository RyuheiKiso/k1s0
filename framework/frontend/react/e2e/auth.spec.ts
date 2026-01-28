import { test, expect, LoginPage, DashboardPage } from './fixtures/test-fixtures';

/**
 * 認証フローの E2E テスト
 */
test.describe('認証フロー', () => {
  test.describe('ログイン', () => {
    test('正常なログインが成功する', async ({ page, testUser }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();

      await loginPage.login(testUser.email, testUser.password);
      await loginPage.expectSuccess();

      // ダッシュボードに遷移していることを確認
      await expect(page).toHaveURL(/.*dashboard/);
    });

    test('無効な認証情報でエラーが表示される', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();

      await loginPage.login('invalid@example.com', 'wrongpassword');
      await loginPage.expectError('メールアドレスまたはパスワードが正しくありません');
    });

    test('空のフォームでバリデーションエラーが表示される', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();

      // 空のまま送信
      await page.getByRole('button', { name: 'ログイン' }).click();

      // バリデーションエラーを確認
      await expect(page.getByText('メールアドレスは必須です')).toBeVisible();
      await expect(page.getByText('パスワードは必須です')).toBeVisible();
    });

    test('不正なメールアドレス形式でエラーが表示される', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();

      await page.getByLabel('メールアドレス').fill('invalid-email');
      await page.getByLabel('パスワード').fill('password');
      await page.getByRole('button', { name: 'ログイン' }).click();

      await expect(page.getByText('有効なメールアドレスを入力してください')).toBeVisible();
    });
  });

  test.describe('ログアウト', () => {
    test('ログアウトが正常に動作する', async ({ authenticatedPage }) => {
      const dashboardPage = new DashboardPage(authenticatedPage);

      // ログアウトを実行
      await dashboardPage.logout();

      // ログインページに遷移していることを確認
      await expect(authenticatedPage).toHaveURL(/.*login/);
    });

    test('ログアウト後に保護されたページにアクセスできない', async ({ authenticatedPage }) => {
      const dashboardPage = new DashboardPage(authenticatedPage);

      // ログアウト
      await dashboardPage.logout();

      // ダッシュボードに直接アクセスを試みる
      await authenticatedPage.goto('/dashboard');

      // ログインページにリダイレクトされることを確認
      await expect(authenticatedPage).toHaveURL(/.*login/);
    });
  });

  test.describe('セッション管理', () => {
    test('ページリロード後もログイン状態が維持される', async ({ authenticatedPage }) => {
      // ページをリロード
      await authenticatedPage.reload();

      // まだダッシュボードにいることを確認
      await expect(authenticatedPage).toHaveURL(/.*dashboard/);

      // ユーザー情報が表示されていることを確認
      await expect(authenticatedPage.getByTestId('user-name')).toBeVisible();
    });

    test('新しいタブでもセッションが共有される', async ({ authenticatedPage, context }) => {
      // 新しいタブを開く
      const newPage = await context.newPage();
      await newPage.goto('/dashboard');

      // ログイン状態であることを確認
      await expect(newPage).toHaveURL(/.*dashboard/);

      await newPage.close();
    });
  });

  test.describe('パスワードリセット', () => {
    test('パスワードリセットフローが開始できる', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();

      // パスワードリセットリンクをクリック
      await page.getByRole('link', { name: 'パスワードを忘れた方' }).click();

      // パスワードリセットページに遷移
      await expect(page).toHaveURL(/.*forgot-password/);

      // メールアドレス入力フォームが表示される
      await expect(page.getByLabel('メールアドレス')).toBeVisible();
    });
  });
});
