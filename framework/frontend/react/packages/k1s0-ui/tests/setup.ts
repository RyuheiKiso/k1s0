/**
 * テストセットアップファイル
 */

import { vi } from 'vitest';
import '@testing-library/jest-dom';

// MUI のポータル警告を抑制
vi.mock('@mui/material/Portal', () => ({
  default: ({ children }: { children: React.ReactNode }) => children,
}));

// window.matchMedia のモック（MUI のレスポンシブ対応用）
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
