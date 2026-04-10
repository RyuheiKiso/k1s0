/// ValidatePage.test.tsx: ValidatePage のユニットテスト。
/// LOW-009 監査対応: ValidatePage のテストカバレッジを追加する。
/// Tauri コマンドをモックして UI の状態遷移を検証する。

import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import ValidatePage from '../ValidatePage';

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('ValidatePage', () => {
  it('ページが正常にレンダリングされる', () => {
    renderWithProviders(<ValidatePage />);
    expect(screen.getByTestId('validate-page')).toBeInTheDocument();
  });

  it('検証ボタンが初期状態で表示される', () => {
    renderWithProviders(<ValidatePage />);
    expect(screen.getByTestId('btn-validate')).toBeInTheDocument();
  });

  it('デフォルトのファイルパスが設定されている', () => {
    renderWithProviders(<ValidatePage />);
    const input = screen.getByTestId('input-file-path') as HTMLInputElement;
    expect(input.value).toBe('config/config-schema.yaml');
  });

  it('成功時に診断結果が表示される', async () => {
    const user = userEvent.setup();
    // 検証成功: 空の診断リストを返す
    mockInvoke.mockResolvedValueOnce([]);

    renderWithProviders(<ValidatePage />);

    await user.click(screen.getByTestId('btn-validate'));

    await waitFor(() => {
      expect(screen.getByTestId('validate-result')).toBeInTheDocument();
    });
  });

  it('エラー時にエラーメッセージが表示される', async () => {
    const user = userEvent.setup();
    // 検証失敗: Tauri コマンドがエラーを返す
    mockInvoke.mockRejectedValueOnce('バリデーションコマンドに失敗しました');

    renderWithProviders(<ValidatePage />);

    await user.click(screen.getByTestId('btn-validate'));

    await waitFor(() => {
      expect(screen.getByTestId('error-message')).toBeInTheDocument();
    });
  });

  it('認証済み状態では検証ボタンが有効', () => {
    renderWithProviders(<ValidatePage />, {
      auth: { isAuthenticated: true, loading: false },
    });
    expect(screen.getByTestId('btn-validate')).not.toBeDisabled();
  });

  it('未認証状態では検証ボタンが無効', () => {
    renderWithProviders(<ValidatePage />, {
      auth: { isAuthenticated: false, loading: false },
    });
    expect(screen.getByTestId('btn-validate')).toBeDisabled();
  });
});
