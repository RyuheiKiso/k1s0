import { describe, it, expect } from 'vitest';
import { createApiClient } from './apiClient';

describe('createApiClient', () => {
  it('Axios インスタンスを返す', () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    expect(client).toBeDefined();
    expect(typeof client.get).toBe('function');
    expect(typeof client.post).toBe('function');
  });

  it('withCredentials が true に設定される', () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    expect(client.defaults.withCredentials).toBe(true);
  });

  it('baseURL が設定される', () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    expect(client.defaults.baseURL).toBe('https://api.example.com');
  });

  it('Content-Type ヘッダーが設定される', () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    expect(client.defaults.headers['Content-Type']).toBe('application/json');
  });
});
