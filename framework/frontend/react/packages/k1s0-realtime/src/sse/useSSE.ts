/**
 * useSSE フック
 *
 * Server-Sent Events の状態管理と操作を提供する React フック。
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import type { ConnectionStatus } from '../types.js';
import type { SSEEvent, UseSSEOptions, UseSSEReturn } from './types.js';
import { SSEClient } from './SSEClient.js';

/**
 * SSE 接続を管理する React フック
 * @param options - SSE オプション
 * @returns 接続状態と操作関数
 */
export function useSSE<T = unknown>(
  options: UseSSEOptions<T>,
): UseSSEReturn<T> {
  const {
    url,
    withCredentials,
    eventHandlers,
    onMessage,
    onError,
    reconnect: reconnectConfig,
    deserialize = (data: string) => JSON.parse(data) as T,
    autoConnect = true,
  } = options;

  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [lastEvent, setLastEvent] = useState<SSEEvent<T> | null>(null);
  const [error, setError] = useState<Error | null>(null);

  const clientRef = useRef<SSEClient | null>(null);
  const reconnectAttemptRef = useRef(0);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const intentionalDisconnect = useRef(false);

  const callbacksRef = useRef({ onMessage, onError, eventHandlers });
  callbacksRef.current = { onMessage, onError, eventHandlers };

  const doConnect = useCallback(() => {
    if (!clientRef.current) {
      clientRef.current = new SSEClient();
    }

    const client = clientRef.current;
    intentionalDisconnect.current = false;
    reconnectAttemptRef.current = 0;

    client.removeAllListeners();

    client.onStatusChange((s) => {
      setStatus(s);

      // EventSource の自動再接続が失敗し closed になった場合のカスタム再接続
      if (s === 'disconnected' && !intentionalDisconnect.current && reconnectConfig?.enabled) {
        const maxAttempts = reconnectConfig.maxAttempts ?? 0;
        if (maxAttempts === 0 || reconnectAttemptRef.current < maxAttempts) {
          reconnectAttemptRef.current++;
          const interval = reconnectConfig.interval ?? 5000;
          reconnectTimerRef.current = setTimeout(() => {
            reconnectTimerRef.current = null;
            doConnect();
          }, interval);
        }
      }
    });

    client.onError((event) => {
      setError(new Error('SSE connection error'));
      callbacksRef.current.onError?.(event);
    });

    client.onMessage((event) => {
      try {
        const data = deserialize(event.data as string);
        setLastEvent({ type: 'message', data });
        callbacksRef.current.onMessage?.(data);
      } catch (e) {
        setError(e instanceof Error ? e : new Error('Failed to deserialize SSE data'));
      }
    });

    // イベントタイプ別ハンドラの登録
    if (callbacksRef.current.eventHandlers) {
      for (const [eventType, handler] of Object.entries(callbacksRef.current.eventHandlers)) {
        client.addEventListener(eventType, (event) => {
          try {
            const data = deserialize(event.data as string);
            setLastEvent({ type: eventType, data });
            handler(data);
          } catch (e) {
            setError(e instanceof Error ? e : new Error(`Failed to deserialize SSE event: ${eventType}`));
          }
        });
      }
    }

    client.connect(url, withCredentials);
  }, [url, withCredentials, reconnectConfig, deserialize]);

  const connect = useCallback(() => {
    doConnect();
  }, [doConnect]);

  const disconnect = useCallback(() => {
    intentionalDisconnect.current = true;
    if (reconnectTimerRef.current !== null) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
    clientRef.current?.disconnect();
  }, []);

  // 自動接続
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      intentionalDisconnect.current = true;
      if (reconnectTimerRef.current !== null) {
        clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
      clientRef.current?.disconnect();
      clientRef.current?.removeAllListeners();
    };
  }, [autoConnect, connect]);

  return {
    status,
    lastEvent,
    error,
    connect,
    disconnect,
  };
}
