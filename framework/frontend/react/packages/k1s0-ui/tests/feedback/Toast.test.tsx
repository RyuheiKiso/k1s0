/**
 * Toast コンポーネントのテスト
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import React from 'react';
import { ToastProvider, useToast } from '../../src/feedback/Toast';

// useToast を使用するテストコンポーネント
function TestComponent() {
  const toast = useToast();

  return (
    <div>
      <button onClick={() => toast.success('Success message')}>Show Success</button>
      <button onClick={() => toast.error('Error message')}>Show Error</button>
      <button onClick={() => toast.warning('Warning message')}>Show Warning</button>
      <button onClick={() => toast.info('Info message')}>Show Info</button>
      <button
        onClick={() =>
          toast.show({
            message: 'Custom message',
            severity: 'info',
            duration: 0,
            actionLabel: 'Action',
            onAction: () => console.log('Action clicked'),
          })
        }
      >
        Show Custom
      </button>
    </div>
  );
}

describe('ToastProvider', () => {
  describe('基本的な動作', () => {
    it('子要素をレンダリングすること', () => {
      render(
        <ToastProvider>
          <div data-testid="child">Child Content</div>
        </ToastProvider>
      );

      expect(screen.getByTestId('child')).toBeInTheDocument();
    });

    it('ToastProvider 外で useToast を使用するとエラーになること', () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => {
        render(<TestComponent />);
      }).toThrow('useToast must be used within a ToastProvider');

      consoleError.mockRestore();
    });
  });

  describe('トースト表示', () => {
    it('success トーストが表示されること', async () => {
      render(
        <ToastProvider>
          <TestComponent />
        </ToastProvider>
      );

      await act(async () => {
        fireEvent.click(screen.getByText('Show Success'));
      });

      expect(screen.getByText('Success message')).toBeInTheDocument();
    });

    it('error トーストが表示されること', async () => {
      render(
        <ToastProvider>
          <TestComponent />
        </ToastProvider>
      );

      await act(async () => {
        fireEvent.click(screen.getByText('Show Error'));
      });

      expect(screen.getByText('Error message')).toBeInTheDocument();
    });

    it('warning トーストが表示されること', async () => {
      render(
        <ToastProvider>
          <TestComponent />
        </ToastProvider>
      );

      await act(async () => {
        fireEvent.click(screen.getByText('Show Warning'));
      });

      expect(screen.getByText('Warning message')).toBeInTheDocument();
    });

    it('info トーストが表示されること', async () => {
      render(
        <ToastProvider>
          <TestComponent />
        </ToastProvider>
      );

      await act(async () => {
        fireEvent.click(screen.getByText('Show Info'));
      });

      expect(screen.getByText('Info message')).toBeInTheDocument();
    });
  });

  describe('カスタムオプション', () => {
    it('アクションボタンが表示されること', async () => {
      render(
        <ToastProvider>
          <TestComponent />
        </ToastProvider>
      );

      await act(async () => {
        fireEvent.click(screen.getByText('Show Custom'));
      });

      expect(screen.getByText('Action')).toBeInTheDocument();
    });
  });

  describe('最大表示数', () => {
    it('最大表示数を超えると古いトーストが削除されること', async () => {
      render(
        <ToastProvider maxToasts={2}>
          <TestComponent />
        </ToastProvider>
      );

      // 3つのトーストを連続表示
      await act(async () => {
        fireEvent.click(screen.getByText('Show Success'));
        fireEvent.click(screen.getByText('Show Error'));
        fireEvent.click(screen.getByText('Show Warning'));
      });

      // 最大2つなので、最初のトーストは削除されている
      expect(screen.queryByText('Success message')).not.toBeInTheDocument();
      expect(screen.getByText('Error message')).toBeInTheDocument();
      expect(screen.getByText('Warning message')).toBeInTheDocument();
    });
  });

  describe('閉じる機能', () => {
    it('閉じるボタンでトーストを閉じられること', async () => {
      vi.useFakeTimers();

      render(
        <ToastProvider>
          <TestComponent />
        </ToastProvider>
      );

      await act(async () => {
        fireEvent.click(screen.getByText('Show Success'));
      });

      expect(screen.getByText('Success message')).toBeInTheDocument();

      // Alert の閉じるボタンをクリック
      const closeButtons = screen.getAllByRole('button', { name: /close/i });
      await act(async () => {
        fireEvent.click(closeButtons[0]);
      });

      // アニメーション完了を待つ
      await act(async () => {
        vi.advanceTimersByTime(300);
      });

      expect(screen.queryByText('Success message')).not.toBeInTheDocument();

      vi.useRealTimers();
    });
  });
});

describe('useToast', () => {
  it('全てのメソッドが利用可能であること', () => {
    let toastMethods: ReturnType<typeof useToast> | null = null;

    function CaptureToast() {
      toastMethods = useToast();
      return null;
    }

    render(
      <ToastProvider>
        <CaptureToast />
      </ToastProvider>
    );

    expect(toastMethods).not.toBeNull();
    expect(typeof toastMethods!.show).toBe('function');
    expect(typeof toastMethods!.success).toBe('function');
    expect(typeof toastMethods!.error).toBe('function');
    expect(typeof toastMethods!.warning).toBe('function');
    expect(typeof toastMethods!.info).toBe('function');
  });
});
