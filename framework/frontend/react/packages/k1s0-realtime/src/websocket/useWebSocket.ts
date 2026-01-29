/**
 * useWebSocket フック
 *
 * WebSocket 接続の状態管理と操作を提供する React フック。
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import type { ConnectionStatus, UseWebSocketOptions, UseWebSocketReturn } from '../types.js';
import { WebSocketClient } from './WebSocketClient.js';
import { ReconnectHandler } from './reconnect.js';
import { HeartbeatHandler } from './heartbeat.js';

/**
 * WebSocket 接続を管理する React フック
 * @param options - WebSocket オプション
 * @returns 接続状態と操作関数
 */
export function useWebSocket<T = unknown>(
  options: UseWebSocketOptions<T>,
): UseWebSocketReturn<T> {
  const {
    url,
    protocols,
    reconnect: reconnectConfig,
    heartbeat: heartbeatConfig,
    getAuthToken,
    serialize = JSON.stringify,
    deserialize = (data: string) => JSON.parse(data) as T,
    onOpen,
    onClose,
    onError,
    onMessage,
    onReconnecting,
    onReconnected,
    autoConnect = true,
  } = options;

  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [lastMessage, setLastMessage] = useState<T | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [reconnectAttempt, setReconnectAttempt] = useState(0);

  const clientRef = useRef<WebSocketClient | null>(null);
  const reconnectRef = useRef<ReconnectHandler | null>(null);
  const heartbeatRef = useRef<HeartbeatHandler | null>(null);
  const intentionalDisconnect = useRef(false);

  // コールバックの最新値を保持
  const callbacksRef = useRef({
    onOpen, onClose, onError, onMessage, onReconnecting, onReconnected,
  });
  callbacksRef.current = { onOpen, onClose, onError, onMessage, onReconnecting, onReconnected };

  const doConnect = useCallback(async () => {
    if (!clientRef.current) {
      clientRef.current = new WebSocketClient();
    }
    if (!reconnectRef.current) {
      reconnectRef.current = new ReconnectHandler(reconnectConfig);
    }
    if (!heartbeatRef.current) {
      heartbeatRef.current = new HeartbeatHandler(heartbeatConfig);
    }

    const client = clientRef.current;
    const reconnect = reconnectRef.current;
    const heartbeat = heartbeatRef.current;

    intentionalDisconnect.current = false;

    // 認証トークン付き URL の構築
    let connectUrl = url;
    if (getAuthToken) {
      const token = await getAuthToken();
      const separator = url.includes('?') ? '&' : '?';
      connectUrl = `${url}${separator}token=${encodeURIComponent(token)}`;
    }

    client.removeAllListeners();

    client.on('statusChange', (s) => {
      setStatus(s);
    });

    client.on('open', (event) => {
      setError(null);
      reconnect.reset();
      setReconnectAttempt(0);

      heartbeat.start(
        (data) => client.send(data),
        () => {
          // pong タイムアウト → 切断して再接続
          client.disconnect();
        },
      );

      if (reconnect.getAttempt() > 0) {
        callbacksRef.current.onReconnected?.();
      }
      callbacksRef.current.onOpen?.(event);
    });

    client.on('close', (event) => {
      heartbeat.stop();
      callbacksRef.current.onClose?.(event);

      if (!intentionalDisconnect.current) {
        reconnect.schedule(
          () => { void doConnect(); },
          (attempt) => {
            setReconnectAttempt(attempt);
            callbacksRef.current.onReconnecting?.(attempt);
          },
        );
      }
    });

    client.on('error', (event) => {
      setError(new Error('WebSocket error'));
      callbacksRef.current.onError?.(event);
    });

    client.on('message', (event) => {
      // ハートビート応答チェック
      if (heartbeat.handleMessage(event.data)) return;

      try {
        const data = deserialize(event.data as string);
        setLastMessage(data);
        callbacksRef.current.onMessage?.(data);
      } catch (e) {
        setError(e instanceof Error ? e : new Error('Failed to deserialize message'));
      }
    });

    client.connect(connectUrl, protocols);
  }, [url, protocols, reconnectConfig, heartbeatConfig, getAuthToken, deserialize]);

  const connect = useCallback(() => {
    void doConnect();
  }, [doConnect]);

  const disconnect = useCallback((code?: number, reason?: string) => {
    intentionalDisconnect.current = true;
    reconnectRef.current?.stop();
    heartbeatRef.current?.stop();
    clientRef.current?.disconnect(code, reason);
  }, []);

  const sendMessage = useCallback((data: unknown) => {
    if (!clientRef.current) {
      throw new Error('WebSocket is not connected');
    }
    clientRef.current.send(serialize(data));
  }, [serialize]);

  const sendJson = useCallback((data: object) => {
    sendMessage(data);
  }, [sendMessage]);

  const getSocket = useCallback(() => {
    return clientRef.current?.getSocket() ?? null;
  }, []);

  // 自動接続
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      intentionalDisconnect.current = true;
      reconnectRef.current?.stop();
      heartbeatRef.current?.stop();
      clientRef.current?.disconnect();
      clientRef.current?.removeAllListeners();
    };
  }, [autoConnect, connect]);

  return {
    status,
    lastMessage,
    error,
    reconnectAttempt,
    connect,
    disconnect,
    sendMessage,
    sendJson,
    getSocket,
  };
}
