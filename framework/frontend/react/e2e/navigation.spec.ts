import { test, expect, NavigationHelper, DashboardPage } from './fixtures/test-fixtures';

/**
 * ナビゲーションの E2E テスト
 */
test.describe('ナビゲーション', () => {
  test.describe('サイドバーナビゲーション', () => {
    test('メインメニュー項目が表示される', async ({ authenticatedPage }) => {
      const nav = authenticatedPage.getByRole('navigation');

      await expect(nav.getByRole('link', { name: 'ダッシュボード' })).toBeVisible();
      await expect(nav.getByRole('link', { name: '設定' })).toBeVisible();
    });

    test('メニュー項目クリックで正しいページに遷移する', async ({ authenticatedPage }) => {
      const navHelper = new NavigationHelper(authenticatedPage);

      // 設定ページに移動
      await navHelper.clickNavLink('設定');
      await navHelper.expectCurrentPath('/settings');

      // ダッシュボードに戻る
      await navHelper.clickNavLink('ダッシュボード');
      await navHelper.expectCurrentPath('/dashboard');
    });

    test('現在のページがハイライトされる', async ({ authenticatedPage }) => {
      // ダッシュボードのリンクがアクティブ
      const dashboardLink = authenticatedPage.getByRole('navigation').getByRole('link', { name: 'ダッシュボード' });
      await expect(dashboardLink).toHaveAttribute('aria-current', 'page');

      // 設定に移動
      await authenticatedPage.getByRole('navigation').getByRole('link', { name: '設定' }).click();

      // 設定リンクがアクティブ
      const settingsLink = authenticatedPage.getByRole('navigation').getByRole('link', { name: '設定' });
      await expect(settingsLink).toHaveAttribute('aria-current', 'page');
    });
  });

  test.describe('ブラウザ履歴', () => {
    test('戻るボタンが正常に動作する', async ({ authenticatedPage }) => {
      const navHelper = new NavigationHelper(authenticatedPage);

      // 設定ページに移動
      await navHelper.clickNavLink('設定');
      await navHelper.expectCurrentPath('/settings');

      // 戻る
      await navHelper.goBack();
      await navHelper.expectCurrentPath('/dashboard');
    });

    test('進むボタンが正常に動作する', async ({ authenticatedPage }) => {
      const navHelper = new NavigationHelper(authenticatedPage);

      // 設定ページに移動
      await navHelper.clickNavLink('設定');

      // 戻る
      await navHelper.goBack();

      // 進む
      await navHelper.goForward();
      await navHelper.expectCurrentPath('/settings');
    });
  });

  test.describe('ブレッドクラム', () => {
    test('ブレッドクラムが現在のパスを表示する', async ({ authenticatedPage }) => {
      // 設定ページに移動
      await authenticatedPage.getByRole('navigation').getByRole('link', { name: '設定' }).click();

      // ブレッドクラムを確認
      const breadcrumb = authenticatedPage.getByRole('navigation', { name: 'breadcrumb' });
      await expect(breadcrumb.getByText('ホーム')).toBeVisible();
      await expect(breadcrumb.getByText('設定')).toBeVisible();
    });

    test('ブレッドクラムのリンクで遷移できる', async ({ authenticatedPage }) => {
      // 設定ページに移動
      await authenticatedPage.getByRole('navigation').getByRole('link', { name: '設定' }).click();

      // ブレッドクラムのホームリンクをクリック
      const breadcrumb = authenticatedPage.getByRole('navigation', { name: 'breadcrumb' });
      await breadcrumb.getByRole('link', { name: 'ホーム' }).click();

      // ダッシュボードに戻る
      await expect(authenticatedPage).toHaveURL(/.*dashboard/);
    });
  });

  test.describe('404 ページ', () => {
    test('存在しないページで 404 が表示される', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/nonexistent-page-12345');

      await expect(authenticatedPage.getByRole('heading', { name: /ページが見つかりません|404/ })).toBeVisible();
    });

    test('404 ページからホームに戻れる', async ({ authenticatedPage }) => {
      await authenticatedPage.goto('/nonexistent-page-12345');

      await authenticatedPage.getByRole('link', { name: /ホームに戻る|トップページへ/ }).click();

      await expect(authenticatedPage).toHaveURL(/.*dashboard/);
    });
  });

  test.describe('レスポンシブナビゲーション', () => {
    test('モバイルビューでハンバーガーメニューが表示される', async ({ authenticatedPage }) => {
      // モバイルビューポートに変更
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      // ハンバーガーメニューが表示される
      await expect(authenticatedPage.getByRole('button', { name: /メニュー/ })).toBeVisible();
    });

    test('ハンバーガーメニューでナビゲーションが開閉する', async ({ authenticatedPage }) => {
      await authenticatedPage.setViewportSize({ width: 375, height: 667 });

      // メニューボタンをクリック
      await authenticatedPage.getByRole('button', { name: /メニュー/ }).click();

      // ナビゲーションメニューが表示される
      const nav = authenticatedPage.getByRole('navigation');
      await expect(nav.getByRole('link', { name: 'ダッシュボード' })).toBeVisible();

      // 閉じる
      await authenticatedPage.getByRole('button', { name: /閉じる/ }).click();

      // ナビゲーションが非表示
      await expect(nav.getByRole('link', { name: 'ダッシュボード' })).not.toBeVisible();
    });
  });

  test.describe('キーボードナビゲーション', () => {
    test('Tab キーでナビゲーション要素を移動できる', async ({ authenticatedPage }) => {
      // 最初の要素にフォーカス
      await authenticatedPage.keyboard.press('Tab');

      // スキップリンクまたは最初のナビゲーション要素にフォーカスが当たる
      const focused = await authenticatedPage.evaluate(() => document.activeElement?.tagName);
      expect(['A', 'BUTTON']).toContain(focused);
    });

    test('Enter キーでリンクをアクティベートできる', async ({ authenticatedPage }) => {
      // 設定リンクにフォーカス
      await authenticatedPage.getByRole('link', { name: '設定' }).focus();

      // Enter で遷移
      await authenticatedPage.keyboard.press('Enter');

      await expect(authenticatedPage).toHaveURL(/.*settings/);
    });
  });
});
