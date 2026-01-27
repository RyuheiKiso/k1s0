/**
 * テストセットアップファイル
 */

import { vi, beforeAll, afterAll, afterEach } from 'vitest';
import '@testing-library/jest-dom';

// fetch のグローバルモック
const mockFetch = vi.fn();
global.fetch = mockFetch;

// AbortController のモック
const mockAbortController = class {
  signal = {
    aborted: false,
    reason: undefined,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  };
  abort = vi.fn((reason?: unknown) => {
    this.signal.aborted = true;
    this.signal.reason = reason;
  });
};
global.AbortController = mockAbortController as unknown as typeof AbortController;

beforeAll(() => {
  // テスト前の共通セットアップ
});

afterEach(() => {
  vi.clearAllMocks();
});

afterAll(() => {
  // テスト後の共通クリーンアップ
});

export { mockFetch };
