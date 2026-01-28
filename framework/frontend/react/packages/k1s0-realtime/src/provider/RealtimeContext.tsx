/**
 * Realtime Context
 */

import { createContext } from 'react';
import type { RealtimeContextValue } from './types.js';

const noop = () => { /* no-op */ };

/** デフォルト値 */
const defaultValue: RealtimeContextValue = {
  isOnline: true,
  connections: new Map(),
  registerConnection: noop,
  unregisterConnection: noop,
  queue: noop,
  flush: noop,
  getQueuedItems: () => [],
  clearQueue: noop,
};

export const RealtimeContext = createContext<RealtimeContextValue>(defaultValue);
