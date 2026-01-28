/**
 * useSSE フックのテスト
 */

import { describe, expect, it } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useSSE } from '../../src/sse/useSSE.js';
import { SSEClient } from '../../src/sse/SSEClient.js';

describe('useSSE', () => {
  it('autoConnect: false では disconnected 状態で開始する', () => {
    const { result } = renderHook(() =>
      useSSE({ url: '/api/stream', autoConnect: false }),
    );

    expect(result.current.status).toBe('disconnected');
    expect(result.current.lastEvent).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it('autoConnect: true では connecting 状態で開始する', () => {
    const { result } = renderHook(() =>
      useSSE({ url: '/api/stream', autoConnect: true }),
    );

    expect(result.current.status).toBe('connecting');
  });

  it('connect/disconnect 関数を返す', () => {
    const { result } = renderHook(() =>
      useSSE({ url: '/api/stream', autoConnect: false }),
    );

    expect(typeof result.current.connect).toBe('function');
    expect(typeof result.current.disconnect).toBe('function');
  });

  it('アンマウント時にクリーンアップされる', () => {
    const { unmount } = renderHook(() =>
      useSSE({ url: '/api/stream', autoConnect: false }),
    );

    expect(() => unmount()).not.toThrow();
  });
});

describe('SSEClient', () => {
  it('初期状態は disconnected', () => {
    const client = new SSEClient();
    expect(client.getStatus()).toBe('disconnected');
  });

  it('connect で connecting 状態になる', () => {
    const client = new SSEClient();
    const handler = vi.fn();
    client.onStatusChange(handler);

    client.connect('/api/stream');

    expect(handler).toHaveBeenCalledWith('connecting');
  });

  it('disconnect で disconnected 状態になる', () => {
    const client = new SSEClient();
    client.connect('/api/stream');
    client.disconnect();

    expect(client.getStatus()).toBe('disconnected');
  });

  it('removeAllListeners で全ハンドラがクリアされる', () => {
    const client = new SSEClient();
    const handler = vi.fn();
    client.onStatusChange(handler);

    client.removeAllListeners();
    client.connect('/api/stream');

    // リスナーがクリアされているので呼ばれない
    expect(handler).not.toHaveBeenCalled();
  });
});
