import { vi } from 'vitest';

// Mock Tauri invoke
const mockInvoke = vi.fn();

// Mock Channel class
class MockChannel {
  onmessage: ((event: unknown) => void) | null = null;
}

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
  Channel: MockChannel,
}));

export { mockInvoke, MockChannel };
