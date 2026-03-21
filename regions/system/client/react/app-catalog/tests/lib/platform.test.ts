import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { platformLabels, detectClientPlatform, formatArch, formatBytes } from '../../src/lib/platform';

describe('platformLabels', () => {
  it('windows のラベルは Windows', () => {
    expect(platformLabels.windows).toBe('Windows');
  });

  it('linux のラベルは Linux', () => {
    expect(platformLabels.linux).toBe('Linux');
  });

  it('macos のラベルは macOS', () => {
    expect(platformLabels.macos).toBe('macOS');
  });
});

describe('detectClientPlatform', () => {
  const originalNavigator = globalThis.navigator;

  afterEach(() => {
    Object.defineProperty(globalThis, 'navigator', {
      value: originalNavigator,
      writable: true,
    });
  });

  it('navigator が undefined の場合は null を返す', () => {
    Object.defineProperty(globalThis, 'navigator', {
      value: undefined,
      writable: true,
    });
    expect(detectClientPlatform()).toBeNull();
  });

  it('Windows UserAgent では windows を返す', () => {
    Object.defineProperty(globalThis, 'navigator', {
      value: { userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)' },
      writable: true,
    });
    expect(detectClientPlatform()).toBe('windows');
  });

  it('Mac UserAgent では macos を返す', () => {
    Object.defineProperty(globalThis, 'navigator', {
      value: { userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15)' },
      writable: true,
    });
    expect(detectClientPlatform()).toBe('macos');
  });

  it('Linux UserAgent では linux を返す', () => {
    Object.defineProperty(globalThis, 'navigator', {
      value: { userAgent: 'Mozilla/5.0 (X11; Linux x86_64)' },
      writable: true,
    });
    expect(detectClientPlatform()).toBe('linux');
  });

  it('不明な UserAgent では null を返す', () => {
    Object.defineProperty(globalThis, 'navigator', {
      value: { userAgent: 'unknown-browser/1.0' },
      writable: true,
    });
    expect(detectClientPlatform()).toBeNull();
  });
});

describe('formatArch', () => {
  it('amd64 は x64 に変換される', () => {
    expect(formatArch('amd64')).toBe('x64');
  });

  it('arm64 は ARM64 に変換される', () => {
    expect(formatArch('arm64')).toBe('ARM64');
  });

  it('未知のアーキテクチャはそのまま返す', () => {
    expect(formatArch('riscv64')).toBe('riscv64');
  });

  it('空文字列はそのまま返す', () => {
    expect(formatArch('')).toBe('');
  });
});

describe('formatBytes', () => {
  it('null は - を返す', () => {
    expect(formatBytes(null)).toBe('-');
  });

  it('0 は 0 B を返す', () => {
    expect(formatBytes(0)).toBe('0 B');
  });

  it('1023 は 1023 B を返す', () => {
    expect(formatBytes(1023)).toBe('1023 B');
  });

  it('1024 は 1.0 KB を返す', () => {
    expect(formatBytes(1024)).toBe('1.0 KB');
  });

  it('1536 は 1.5 KB を返す', () => {
    expect(formatBytes(1536)).toBe('1.5 KB');
  });

  it('1MB は 1.0 MB を返す', () => {
    expect(formatBytes(1024 * 1024)).toBe('1.0 MB');
  });

  it('1GB は 1.0 GB を返す', () => {
    expect(formatBytes(1024 * 1024 * 1024)).toBe('1.0 GB');
  });
});
