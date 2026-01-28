/**
 * 統合テスト
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { render, screen } from '@testing-library/react';
import { useContext, type ReactNode } from 'react';
import { RealtimeProvider } from '../src/provider/RealtimeProvider.js';
import { RealtimeContext } from '../src/provider/RealtimeContext.js';
import { useWebSocket } from '../src/websocket/useWebSocket.js';

function wrapper({ children }: { children: ReactNode }) {
  return (
    <RealtimeProvider
      config={{
        offlineQueue: { enabled: true, maxSize: 50, persistToStorage: false },
      }}
    >
      {children}
    </RealtimeProvider>
  );
}

describe('統合テスト', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  it('WebSocket + Provider でメッセージのキューイングができる', async () => {
    const { result } = renderHook(
      () => {
        const ws = useWebSocket({ url: 'ws://localhost/test', autoConnect: false });
        const ctx = useContext(RealtimeContext);
        return { ws, ctx };
      },
      { wrapper },
    );

    // オフライン状態でメッセージをキューに追加
    act(() => {
      result.current.ctx.queue('test-conn', { type: 'chat', text: 'hello' });
    });

    const items = result.current.ctx.getQueuedItems('test-conn');
    expect(items).toHaveLength(1);
    expect(items[0]).toEqual({ type: 'chat', text: 'hello' });
  });

  it('Provider がネットワーク状態のデフォルト値を提供する', () => {
    function StatusDisplay() {
      const { isOnline } = useContext(RealtimeContext);
      return <span data-testid="status">{isOnline ? 'online' : 'offline'}</span>;
    }

    render(
      <RealtimeProvider>
        <StatusDisplay />
      </RealtimeProvider>,
    );

    expect(screen.getByTestId('status').textContent).toBe('online');
  });
});
