/**
 * useOfflineQueue フック
 */

import { useContext } from 'react';
import { RealtimeContext } from './RealtimeContext.js';

/**
 * オフラインキュー操作を提供するフック
 */
export function useOfflineQueue() {
  const { queue, flush, getQueuedItems, clearQueue } = useContext(RealtimeContext);
  return { queue, flush, getQueuedItems, clearQueue };
}
