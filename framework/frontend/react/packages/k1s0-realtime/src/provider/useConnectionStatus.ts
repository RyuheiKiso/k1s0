/**
 * useConnectionStatus フック
 */

import { useContext } from 'react';
import { RealtimeContext } from './RealtimeContext.js';
import type { ConnectionInfo } from './types.js';

/**
 * グローバルな接続状態を取得するフック
 */
export function useConnectionStatus(): {
  isOnline: boolean;
  connections: Map<string, ConnectionInfo>;
} {
  const { isOnline, connections } = useContext(RealtimeContext);
  return { isOnline, connections };
}
