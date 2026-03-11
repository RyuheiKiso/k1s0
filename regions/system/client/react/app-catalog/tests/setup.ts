import '@testing-library/jest-dom';
import { vi } from 'vitest';

// system-client の AuthProvider と ProtectedRoute をモック
vi.mock('system-client', async () => {
  const actual = await vi.importActual<Record<string, unknown>>('system-client');
  return {
    ...actual,
    AuthProvider: ({ children }: { children: React.ReactNode }) => children,
    ProtectedRoute: ({ children }: { children: React.ReactNode }) => children,
    LoadingSpinner: ({ message }: { message?: string }) => {
      const { createElement } = require('react');
      return createElement('div', { role: 'status' }, message ?? 'Loading');
    },
    createApiClient: actual.createApiClient,
  };
});
