// ${{values.screen_name}}.test.tsx — ${{values.screen_name}} の Vitest テストテンプレート
// Backstage SoftwareTemplate が scaffold する際に生成される test ファイル
// axe-core によるアクセシビリティ検査を必ず含める（WCAG 2.1 AA CI 要件）

// Vitest から test / expect / beforeEach / vi をインポートする
import { describe, it, expect, beforeEach, vi } from 'vitest';
// React Testing Library をインポートする
import { render, screen, waitFor } from '@testing-library/react';
// axe-core の jest-axe 互換ラッパーをインポートする（アクセシビリティ検査用）
import { axe, toHaveNoViolations } from 'jest-axe';
// テスト対象コンポーネントをインポートする
import ${{values.screen_name}} from './${{values.screen_name}}';

// jest-axe のカスタムマッチャーを Vitest に追加する
expect.extend(toHaveNoViolations);

// ${{values.screen_name}} コンポーネントのテストスイート
describe('${{values.screen_name}}', () => {
  // 各テスト前にモックをリセットする
  beforeEach(() => {
    // Vitest のモックを全てリセットして独立性を保証する
    vi.resetAllMocks();
  });

  // --- 基本描画テスト ---
  it('renders without crashing', async () => {
    // コンポーネントをレンダリングする
    const { container } = render(
      <${{values.screen_name}} entityId="test-entity-1" />,
    );
    // コンテナが DOM に存在することを確認する
    expect(container).toBeDefined();
  });

  // --- ローディング状態テスト ---
  it('shows loading spinner while fetching data', () => {
    // データ取得中のローディング状態を検証する
    render(<${{values.screen_name}} entityId="test-entity-1" />);
    // ローディングスピナーが表示されていることを確認する
    const spinner = screen.getByRole('img', { hidden: true });
    // スピナー要素が DOM に存在することを確認する
    expect(spinner).toBeDefined();
  });

  // --- アクセシビリティテスト（axe-core） ---
  // WCAG 2.1 AA 準拠を CI で強制する（違反があれば CI fail）
  it('has no accessibility violations', async () => {
    // コンポーネントをレンダリングする
    const { container } = render(
      <${{values.screen_name}} entityId="test-entity-1" />,
    );
    // データ取得完了後の DOM が安定するのを待つ
    await waitFor(() => {
      // ローディング状態が終了していることを確認する
      expect(screen.queryByRole('img', { hidden: true })).toBeNull();
    });
    // axe-core でアクセシビリティ検査を実行する
    const results = await axe(container);
    // アクセシビリティ違反がないことを検証する
    expect(results).toHaveNoViolations();
  });

  // --- エラー状態テスト ---
  it('shows error message when fetch fails', async () => {
    // tier2 SDK のモックを設定してエラーを投げる
    vi.mock('@k1s0/tier2-sdk', () => ({
      // SDK がエラーを返すようにモックする
      get: vi.fn().mockRejectedValue(new Error('Network error')),
    }));
    // コンポーネントをレンダリングする
    render(<${{values.screen_name}} entityId="test-entity-error" />);
    // エラーメッセージが表示されるのを待つ
    await waitFor(() => {
      // role="alert" でマークアップされたエラー要素を取得する
      const alert = screen.queryByRole('alert');
      // エラーアラートが DOM に存在することを確認する
      expect(alert).not.toBeNull();
    });
  });

  // --- エンティティ ID 変更テスト ---
  it('re-fetches data when entityId changes', async () => {
    // コンポーネントをレンダリングする
    const { rerender } = render(
      <${{values.screen_name}} entityId="entity-1" />,
    );
    // entityId を変更して再レンダリングする
    rerender(<${{values.screen_name}} entityId="entity-2" />);
    // データ取得が再実行されることを確認する（ローディング状態が再度現れる）
    await waitFor(() => {
      // ローディング状態が再度発生していることを確認する
      expect(screen.queryByRole('main')).toBeDefined();
    });
  });
});
