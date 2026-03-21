import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ErrorBoundary } from './ErrorBoundary';

// エラーをスローするコンポーネント（テスト用）
function ThrowError({ message }: { message: string }) {
  throw new Error(message);
}

// ErrorBoundary のテストでは React が console.error を出力するため抑制する
const originalConsoleError = console.error;

beforeEach(() => {
  console.error = vi.fn();
});

afterEach(() => {
  console.error = originalConsoleError;
});

describe('ErrorBoundary', () => {
  it('エラーがない場合は子コンポーネントを表示する', () => {
    render(
      <ErrorBoundary>
        <p>正常コンテンツ</p>
      </ErrorBoundary>
    );
    expect(screen.getByText('正常コンテンツ')).toBeInTheDocument();
  });

  it('エラー発生時はデフォルトの fallback を表示する', () => {
    render(
      <ErrorBoundary>
        <ThrowError message="テストエラー" />
      </ErrorBoundary>
    );
    expect(screen.getByRole('alert')).toBeInTheDocument();
    expect(screen.getByText('エラーが発生しました')).toBeInTheDocument();
  });

  it('カスタム fallback が指定された場合はそちらを表示する', () => {
    render(
      <ErrorBoundary fallback={<p>カスタムエラー表示</p>}>
        <ThrowError message="テストエラー" />
      </ErrorBoundary>
    );
    expect(screen.getByText('カスタムエラー表示')).toBeInTheDocument();
  });

  it('エラー発生後に子コンポーネントが表示されない', () => {
    render(
      <ErrorBoundary>
        <ThrowError message="テストエラー" />
        <p>子コンテンツ</p>
      </ErrorBoundary>
    );
    expect(screen.queryByText('子コンテンツ')).not.toBeInTheDocument();
  });
});
