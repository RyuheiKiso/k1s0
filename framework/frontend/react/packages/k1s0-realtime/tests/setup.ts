/**
 * テストセットアップファイル
 */

import { vi, afterEach } from 'vitest';
import '@testing-library/jest-dom';

// WebSocket モック
class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSING = 2;
  readonly CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  url: string;
  protocol = '';
  bufferedAmount = 0;
  extensions = '';
  binaryType: BinaryType = 'blob';

  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(url: string, _protocols?: string | string[]) {
    this.url = url;
    // microtask で open イベントを発火（act 内の await で flush される）
    Promise.resolve().then(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event('open'));
    });
  }

  send = vi.fn();

  close = vi.fn((code?: number, _reason?: string) => {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close', { code: code ?? 1000 }));
  });

  addEventListener = vi.fn();
  removeEventListener = vi.fn();
  dispatchEvent = vi.fn(() => true);

  // テスト用: メッセージを受信させる
  _simulateMessage(data: string) {
    this.onmessage?.(new MessageEvent('message', { data }));
  }

  // テスト用: エラーを発生させる
  _simulateError() {
    this.onerror?.(new Event('error'));
  }

  // テスト用: 接続を切断させる
  _simulateClose(code = 1006) {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close', { code }));
  }
}

global.WebSocket = MockWebSocket as unknown as typeof WebSocket;

// EventSource モック
class MockEventSource {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 2;

  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSED = 2;

  readyState = MockEventSource.CONNECTING;
  url: string;
  withCredentials: boolean;

  onopen: ((event: Event) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  private eventListeners: Map<string, Set<EventListener>> = new Map();

  constructor(url: string, init?: EventSourceInit) {
    this.url = url;
    this.withCredentials = init?.withCredentials ?? false;
    Promise.resolve().then(() => {
      this.readyState = MockEventSource.OPEN;
      this.onopen?.(new Event('open'));
    });
  }

  addEventListener = vi.fn((type: string, listener: EventListener) => {
    if (!this.eventListeners.has(type)) {
      this.eventListeners.set(type, new Set());
    }
    this.eventListeners.get(type)!.add(listener);
  });

  removeEventListener = vi.fn();
  dispatchEvent = vi.fn(() => true);

  close = vi.fn(() => {
    this.readyState = MockEventSource.CLOSED;
  });

  // テスト用
  _simulateMessage(data: string) {
    this.onmessage?.(new MessageEvent('message', { data }));
  }

  _simulateEvent(type: string, data: string) {
    const event = new MessageEvent(type, { data });
    const listeners = this.eventListeners.get(type);
    if (listeners) {
      for (const listener of listeners) {
        listener(event);
      }
    }
  }

  _simulateError() {
    this.readyState = MockEventSource.CLOSED;
    this.onerror?.(new Event('error'));
  }
}

global.EventSource = MockEventSource as unknown as typeof EventSource;

// localStorage モック
const localStorageData: Map<string, string> = new Map();
global.localStorage = {
  getItem: vi.fn((key: string) => localStorageData.get(key) ?? null),
  setItem: vi.fn((key: string, value: string) => { localStorageData.set(key, value); }),
  removeItem: vi.fn((key: string) => { localStorageData.delete(key); }),
  clear: vi.fn(() => { localStorageData.clear(); }),
  get length() { return localStorageData.size; },
  key: vi.fn((index: number) => Array.from(localStorageData.keys())[index] ?? null),
} as Storage;

// navigator.onLine モック
Object.defineProperty(navigator, 'onLine', {
  value: true,
  writable: true,
  configurable: true,
});

afterEach(() => {
  vi.clearAllMocks();
  localStorageData.clear();
});
