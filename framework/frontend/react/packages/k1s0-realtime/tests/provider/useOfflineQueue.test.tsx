/**
 * useOfflineQueue のテスト
 */

import { describe, expect, it, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import type { ReactNode } from 'react';
import { RealtimeProvider } from '../../src/provider/RealtimeProvider.js';
import { useOfflineQueue } from '../../src/provider/useOfflineQueue.js';

function wrapper({ children }: { children: ReactNode }) {
  return (
    <RealtimeProvider config={{ offlineQueue: { enabled: true, maxSize: 10, persistToStorage: false } }}>
      {children}
    </RealtimeProvider>
  );
}

describe('useOfflineQueue', () => {
  it('アイテムをキューに追加して取得できる', () => {
    const { result } = renderHook(() => useOfflineQueue(), { wrapper });

    act(() => {
      result.current.queue('conn1', { type: 'message', data: 'hello' });
      result.current.queue('conn1', { type: 'message', data: 'world' });
    });

    const items = result.current.getQueuedItems('conn1');
    expect(items).toHaveLength(2);
    expect(items[0]).toEqual({ type: 'message', data: 'hello' });
  });

  it('flush でキュー内のアイテムが送信される', () => {
    const { result } = renderHook(() => useOfflineQueue(), { wrapper });
    const send = vi.fn();

    act(() => {
      result.current.queue('conn1', 'msg1');
      result.current.queue('conn1', 'msg2');
    });

    act(() => {
      result.current.flush('conn1', send);
    });

    expect(send).toHaveBeenCalledTimes(2);
    expect(send).toHaveBeenCalledWith('msg1');
    expect(send).toHaveBeenCalledWith('msg2');

    // flush 後はキューが空
    expect(result.current.getQueuedItems('conn1')).toHaveLength(0);
  });

  it('clearQueue でキューがクリアされる', () => {
    const { result } = renderHook(() => useOfflineQueue(), { wrapper });

    act(() => {
      result.current.queue('conn1', 'msg1');
    });

    expect(result.current.getQueuedItems('conn1')).toHaveLength(1);

    act(() => {
      result.current.clearQueue('conn1');
    });

    expect(result.current.getQueuedItems('conn1')).toHaveLength(0);
  });
});
