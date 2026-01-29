/**
 * Realtime Provider コンポーネント
 *
 * グローバルなリアルタイム接続管理、ネットワーク監視、オフラインキューを提供する。
 */

import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { NetworkMonitor } from '../utils/networkMonitor.js';
import { storage } from '../utils/storage.js';
import { RealtimeContext } from './RealtimeContext.js';
import type { ConnectionInfo, OfflineQueueConfig, RealtimeConfig, RealtimeContextValue } from './types.js';

const DEFAULT_QUEUE_CONFIG: OfflineQueueConfig = {
  enabled: true,
  maxSize: 50,
  persistToStorage: true,
  storageKey: 'k1s0_realtime_queue',
};

interface RealtimeProviderProps {
  config?: RealtimeConfig;
  children: ReactNode;
}

/**
 * リアルタイム通信のグローバル Provider
 */
export function RealtimeProvider({ config = {}, children }: RealtimeProviderProps) {
  const [isOnline, setIsOnline] = useState(true);
  const [connections, setConnections] = useState<Map<string, ConnectionInfo>>(new Map());
  const queueRef = useRef<Map<string, unknown[]>>(new Map());

  const queueConfig = useMemo(
    () => ({ ...DEFAULT_QUEUE_CONFIG, ...config.offlineQueue }),
    [config.offlineQueue],
  );

  // localStorage からキューを復元
  useEffect(() => {
    if (queueConfig.persistToStorage && queueConfig.storageKey) {
      const saved = storage.get<Record<string, unknown[]>>(queueConfig.storageKey);
      if (saved) {
        queueRef.current = new Map(Object.entries(saved));
      }
    }
  }, [queueConfig.persistToStorage, queueConfig.storageKey]);

  // ネットワーク監視
  useEffect(() => {
    if (!config.networkMonitor?.enabled) return;

    const monitor = new NetworkMonitor();
    setIsOnline(monitor.isOnline());

    const removeListener = monitor.addListener((online) => {
      setIsOnline(online);
      if (online) {
        config.networkMonitor?.onOnline?.();
      } else {
        config.networkMonitor?.onOffline?.();
      }
    });

    monitor.start();

    return () => {
      removeListener();
      monitor.stop();
    };
  }, [config.networkMonitor]);

  const registerConnection = useCallback((id: string, info: ConnectionInfo) => {
    setConnections((prev) => {
      const next = new Map(prev);
      next.set(id, info);
      return next;
    });
  }, []);

  const unregisterConnection = useCallback((id: string) => {
    setConnections((prev) => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
  }, []);

  const persistQueue = useCallback(() => {
    if (queueConfig.persistToStorage && queueConfig.storageKey) {
      const obj: Record<string, unknown[]> = {};
      for (const [key, value] of queueRef.current) {
        obj[key] = value;
      }
      storage.set(queueConfig.storageKey, obj);
    }
  }, [queueConfig]);

  const queue = useCallback(<T,>(connectionId: string, item: T) => {
    if (!queueConfig.enabled) return;

    const items = queueRef.current.get(connectionId) ?? [];
    if (items.length >= queueConfig.maxSize) {
      items.shift(); // 最古のアイテムを削除
    }
    items.push(item);
    queueRef.current.set(connectionId, items);
    persistQueue();
  }, [queueConfig, persistQueue]);

  const flush = useCallback((connectionId: string, send: (item: unknown) => void) => {
    const items = queueRef.current.get(connectionId);
    if (!items || items.length === 0) return;

    for (const item of items) {
      send(item);
    }
    queueRef.current.delete(connectionId);
    persistQueue();
  }, [persistQueue]);

  const getQueuedItems = useCallback((connectionId: string): unknown[] => {
    return queueRef.current.get(connectionId) ?? [];
  }, []);

  const clearQueue = useCallback((connectionId: string) => {
    queueRef.current.delete(connectionId);
    persistQueue();
  }, [persistQueue]);

  const value = useMemo<RealtimeContextValue>(
    () => ({
      isOnline,
      connections,
      registerConnection,
      unregisterConnection,
      queue,
      flush,
      getQueuedItems,
      clearQueue,
    }),
    [isOnline, connections, registerConnection, unregisterConnection, queue, flush, getQueuedItems, clearQueue],
  );

  return (
    <RealtimeContext.Provider value={value}>
      {children}
    </RealtimeContext.Provider>
  );
}
