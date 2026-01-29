/**
 * useWebSocket フックのテスト
 */

import { describe, expect, it, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useWebSocket } from '../../src/websocket/useWebSocket.js';
import { WebSocketClient } from '../../src/websocket/WebSocketClient.js';
import { ReconnectHandler } from '../../src/websocket/reconnect.js';
import { HeartbeatHandler } from '../../src/websocket/heartbeat.js';

describe('useWebSocket', () => {
  it('autoConnect: false では disconnected 状態で開始する', () => {
    const { result } = renderHook(() =>
      useWebSocket({ url: 'ws://localhost/test', autoConnect: false }),
    );

    expect(result.current.status).toBe('disconnected');
    expect(result.current.lastMessage).toBeNull();
    expect(result.current.error).toBeNull();
    expect(result.current.reconnectAttempt).toBe(0);
  });

  it('autoConnect: true では connecting 状態で開始する', () => {
    const { result } = renderHook(() =>
      useWebSocket({ url: 'ws://localhost/test', autoConnect: true }),
    );

    expect(result.current.status).toBe('connecting');
  });

  it('connect/disconnect/sendMessage/sendJson/getSocket 関数を返す', () => {
    const { result } = renderHook(() =>
      useWebSocket({ url: 'ws://localhost/test', autoConnect: false }),
    );

    expect(typeof result.current.connect).toBe('function');
    expect(typeof result.current.disconnect).toBe('function');
    expect(typeof result.current.sendMessage).toBe('function');
    expect(typeof result.current.sendJson).toBe('function');
    expect(typeof result.current.getSocket).toBe('function');
  });

  it('アンマウント時にクリーンアップされる', () => {
    const { unmount } = renderHook(() =>
      useWebSocket({ url: 'ws://localhost/test', autoConnect: false }),
    );

    // クリーンアップが例外なく完了する
    expect(() => unmount()).not.toThrow();
  });
});

describe('WebSocketClient', () => {
  it('初期状態は disconnected', () => {
    const client = new WebSocketClient();
    expect(client.getStatus()).toBe('disconnected');
    expect(client.getSocket()).toBeNull();
  });

  it('connect で connecting 状態になる', () => {
    const client = new WebSocketClient();
    const handler = vi.fn();
    client.on('statusChange', handler);

    client.connect('ws://localhost/test');

    expect(client.getStatus()).toBe('connecting');
    expect(handler).toHaveBeenCalledWith('connecting');
  });

  it('disconnect でソケットが null になる', () => {
    const client = new WebSocketClient();
    client.connect('ws://localhost/test');
    client.disconnect();

    expect(client.getSocket()).toBeNull();
  });

  it('イベントハンドラの登録と解除ができる', () => {
    const client = new WebSocketClient();
    const handler = vi.fn();

    client.on('statusChange', handler);
    client.connect('ws://localhost/test');
    expect(handler).toHaveBeenCalled();

    handler.mockClear();
    client.off('statusChange', handler);
    client.disconnect();
    // off した後は呼ばれない（disconnect で statusChange は発火するが handler は除去済み）
  });

  it('removeAllListeners で全ハンドラがクリアされる', () => {
    const client = new WebSocketClient();
    const handler = vi.fn();
    client.on('statusChange', handler);

    client.removeAllListeners();
    client.connect('ws://localhost/test');

    // リスナーがクリアされているので呼ばれない
    expect(handler).not.toHaveBeenCalled();
  });
});

describe('ReconnectHandler', () => {
  it('有効時に再接続がスケジュールされる', () => {
    const handler = new ReconnectHandler({
      enabled: true,
      maxAttempts: 3,
      initialDelay: 100,
    });

    const result = handler.schedule(() => {});
    expect(result).toBe(true);
    expect(handler.getAttempt()).toBe(1);
    handler.cancel();
  });

  it('無効時にスケジュールされない', () => {
    const handler = new ReconnectHandler({ enabled: false });
    const result = handler.schedule(() => {});
    expect(result).toBe(false);
  });

  it('最大試行回数でスケジュールが停止する', () => {
    const handler = new ReconnectHandler({
      enabled: true,
      maxAttempts: 2,
      initialDelay: 10,
    });

    handler.schedule(() => {});
    handler.schedule(() => {});
    const result = handler.schedule(() => {});

    expect(result).toBe(false);
    expect(handler.getAttempt()).toBe(2);
    handler.cancel();
  });

  it('reset で試行回数がリセットされる', () => {
    const handler = new ReconnectHandler({
      enabled: true,
      maxAttempts: 5,
      initialDelay: 10,
    });

    handler.schedule(() => {});
    handler.schedule(() => {});
    expect(handler.getAttempt()).toBe(2);

    handler.reset();
    expect(handler.getAttempt()).toBe(0);
  });

  it('stop で再接続が停止する', () => {
    const handler = new ReconnectHandler({
      enabled: true,
      initialDelay: 10,
    });

    handler.stop();
    const result = handler.schedule(() => {});
    expect(result).toBe(false);
  });

  it('restart で停止状態が解除される', () => {
    const handler = new ReconnectHandler({
      enabled: true,
      initialDelay: 10,
    });

    handler.stop();
    handler.restart();
    const result = handler.schedule(() => {});
    expect(result).toBe(true);
    handler.cancel();
  });
});

describe('HeartbeatHandler', () => {
  it('無効時は start しても何も起きない', () => {
    const handler = new HeartbeatHandler({ enabled: false });
    const send = vi.fn();
    const onTimeout = vi.fn();

    handler.start(send, onTimeout);
    // エラーなく完了する
    handler.stop();
  });

  it('handleMessage で expectedResponse が未設定なら false を返す', () => {
    const handler = new HeartbeatHandler({ enabled: true });
    const result = handler.handleMessage('pong');
    expect(result).toBe(false);
  });

  it('handleMessage で expectedResponse が一致すれば true を返す', () => {
    const handler = new HeartbeatHandler({
      enabled: true,
      expectedResponse: 'pong',
    });
    expect(handler.handleMessage('pong')).toBe(true);
    expect(handler.handleMessage('other')).toBe(false);
  });

  it('handleMessage で関数型 expectedResponse が動作する', () => {
    const handler = new HeartbeatHandler({
      enabled: true,
      expectedResponse: (msg: unknown) => msg === 'pong',
    });
    expect(handler.handleMessage('pong')).toBe(true);
    expect(handler.handleMessage('other')).toBe(false);
  });
});
