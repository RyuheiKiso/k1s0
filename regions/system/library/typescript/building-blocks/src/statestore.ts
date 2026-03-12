import type { Component } from './component.js';

export interface StateEntry {
  key: string;
  value: Uint8Array;
  etag: string;
}

export interface StateStore extends Component {
  get(key: string): Promise<StateEntry | null>;
  set(key: string, value: Uint8Array, etag?: string): Promise<string>;
  delete(key: string, etag?: string): Promise<void>;
  bulkGet(keys: string[]): Promise<StateEntry[]>;
  bulkSet(entries: Array<{ key: string; value: Uint8Array }>): Promise<string[]>;
}
