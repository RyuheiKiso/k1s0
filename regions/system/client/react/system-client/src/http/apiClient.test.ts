import { describe, it, expect, vi, afterEach } from 'vitest';
import MockAdapter from 'axios-mock-adapter';
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

describe('レスポンスエラーインターセプター', () => {
  let mock: MockAdapter;

  afterEach(() => {
    mock?.restore();
  });

  // 401 エラー時に onUnauthorized コールバックが呼び出されることを検証
  it('401 エラーで onUnauthorized コールバックが呼ばれる', async () => {
    const onUnauthorized = vi.fn();
    const client = createApiClient({
      baseURL: 'https://api.example.com',
      onUnauthorized,
    });
    mock = new MockAdapter(client);
    mock.onGet('/test').reply(401);

    await expect(client.get('/test')).rejects.toThrow();
    expect(onUnauthorized).toHaveBeenCalledOnce();
  });

  // onUnauthorized 未指定時に 401 エラーでもクラッシュしないことを検証
  it('onUnauthorized 未指定時に 401 エラーでもクラッシュしない', async () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    mock = new MockAdapter(client);
    mock.onGet('/test').reply(401);

    await expect(client.get('/test')).rejects.toThrow();
  });

  it('403 エラーでエラーログを出力する', async () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    mock = new MockAdapter(client);
    mock.onGet('/forbidden').reply(403);

    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await expect(client.get('/forbidden')).rejects.toThrow();
    expect(consoleSpy).toHaveBeenCalledWith(
      '[API] 403 Forbidden:',
      expect.stringContaining('/forbidden'),
    );

    consoleSpy.mockRestore();
  });

  it('500 エラーでエラーログを出力する', async () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    mock = new MockAdapter(client);
    mock.onGet('/error').reply(500);

    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await expect(client.get('/error')).rejects.toThrow();
    expect(consoleSpy).toHaveBeenCalledWith(
      '[API] Server Error:',
      500,
      expect.stringContaining('/error'),
    );

    consoleSpy.mockRestore();
  });

  it('502 エラーでもサーバーエラーログを出力する', async () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    mock = new MockAdapter(client);
    mock.onGet('/gateway').reply(502);

    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    await expect(client.get('/gateway')).rejects.toThrow();
    expect(consoleSpy).toHaveBeenCalledWith(
      '[API] Server Error:',
      502,
      expect.stringContaining('/gateway'),
    );

    consoleSpy.mockRestore();
  });

  it('エラーは reject される（インターセプト後も伝播する）', async () => {
    const client = createApiClient({ baseURL: 'https://api.example.com' });
    mock = new MockAdapter(client);
    mock.onGet('/test').reply(404);

    await expect(client.get('/test')).rejects.toThrow();
  });
});
