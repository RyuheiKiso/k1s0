import { vi, describe, it, expect } from 'vitest';
import {
  AiClientError,
  InMemoryAiClient,
  HttpAiClient,
  type CompleteRequest,
  type EmbedRequest,
} from '../src/index.js';

// InMemoryAiClient のテスト
describe('InMemoryAiClient', () => {
  it('デフォルトでモックレスポンスを返すこと', async () => {
    const client = new InMemoryAiClient();
    const res = await client.complete({
      model: 'test-model',
      messages: [{ role: 'user', content: 'hello' }],
    });
    expect(res.id).toBe('mock-id');
    expect(res.model).toBe('test-model');
    expect(res.content).toBe('mock response');
  });

  it('カスタム complete 関数を使用できること', async () => {
    const client = new InMemoryAiClient({
      complete: async (req) => ({
        id: 'custom-id',
        model: req.model,
        content: 'custom response',
        usage: { inputTokens: 10, outputTokens: 20 },
      }),
    });
    const res = await client.complete({
      model: 'gpt-4',
      messages: [{ role: 'user', content: 'test' }],
    });
    expect(res.id).toBe('custom-id');
    expect(res.usage.inputTokens).toBe(10);
  });

  it('embed でデフォルト空埋め込みを返すこと', async () => {
    const client = new InMemoryAiClient();
    const res = await client.embed({ model: 'embed-model', texts: ['hello', 'world'] });
    expect(res.model).toBe('embed-model');
    expect(res.embeddings).toHaveLength(2);
  });

  it('listModels でデフォルト空配列を返すこと', async () => {
    const client = new InMemoryAiClient();
    const models = await client.listModels();
    expect(models).toEqual([]);
  });
});

// HttpAiClient のテスト
describe('HttpAiClient', () => {
  it('complete で正しいエンドポイントにPOSTすること', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        id: 'resp-id',
        model: 'claude-3',
        content: 'Hello!',
        usage: { input_tokens: 5, output_tokens: 10 },
      }),
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new HttpAiClient({ baseUrl: 'https://api.example.com', apiKey: 'test-key' });
    const res = await client.complete({
      model: 'claude-3',
      messages: [{ role: 'user', content: 'Hi' }],
    });

    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.example.com/v1/complete',
      expect.objectContaining({ method: 'POST' }),
    );
    expect(res.content).toBe('Hello!');
    expect(res.usage.inputTokens).toBe(5);
    expect(res.usage.outputTokens).toBe(10);

    vi.unstubAllGlobals();
  });

  it('embed で正しいエンドポイントにPOSTすること', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        model: 'embed-v1',
        embeddings: [[0.1, 0.2], [0.3, 0.4]],
      }),
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new HttpAiClient({ baseUrl: 'https://api.example.com' });
    const res = await client.embed({ model: 'embed-v1', texts: ['a', 'b'] });

    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.example.com/v1/embed',
      expect.objectContaining({ method: 'POST' }),
    );
    expect(res.embeddings).toHaveLength(2);

    vi.unstubAllGlobals();
  });

  it('listModels で正しいエンドポイントにGETすること', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => [
        { id: 'model-1', name: 'Model One', description: 'desc' },
      ],
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new HttpAiClient({ baseUrl: 'https://api.example.com' });
    const models = await client.listModels();

    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.example.com/v1/models',
      expect.objectContaining({ method: 'GET' }),
    );
    expect(models).toHaveLength(1);
    expect(models[0].id).toBe('model-1');

    vi.unstubAllGlobals();
  });

  it('APIエラー時に AiClientError をスローすること', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
      statusText: 'Unauthorized',
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new HttpAiClient({ baseUrl: 'https://api.example.com' });
    await expect(
      client.complete({ model: 'test', messages: [{ role: 'user', content: 'hi' }] }),
    ).rejects.toThrow(AiClientError);

    vi.unstubAllGlobals();
  });
});

// AiClientError のテスト
describe('AiClientError', () => {
  it('正しいフィールドを持つこと', () => {
    const err = new AiClientError('test error', 500);
    expect(err.message).toBe('test error');
    expect(err.statusCode).toBe(500);
    expect(err.name).toBe('AiClientError');
    expect(err).toBeInstanceOf(Error);
  });
});
