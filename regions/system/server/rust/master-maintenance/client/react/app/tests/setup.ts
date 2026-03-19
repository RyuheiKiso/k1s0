import "@testing-library/jest-dom";

// jsdom環境にResizeObserverが存在しないため、antd v6+のコンポーネントが使用するポリフィルを提供
class ResizeObserverMock {
  observe() { /* noop */ }
  unobserve() { /* noop */ }
  disconnect() { /* noop */ }
}
globalThis.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
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

window.getComputedStyle = vi.fn().mockImplementation(() => ({
  getPropertyValue: vi.fn(() => ""),
})) as typeof window.getComputedStyle;
