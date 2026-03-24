import { beforeEach, describe, it, expect, vi } from 'vitest';
import { GraphQlHttpClient, InMemoryGraphQlClient, ClientError, ClientErrorKind } from '../src/index.js';
import type { GraphQlQuery, GraphQlResponse } from '../src/index.js';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

describe('InMemoryGraphQlClient', () => {
  it('execute でセットした応答が返る', async () => {
    const client = new InMemoryGraphQlClient();
    const expected = { user: { id: '1', name: 'Alice' } };
    client.setResponse('GetUser', expected);

    const resp = await client.execute({
      query: 'query GetUser($id: ID!) { user(id: $id) { id name } }',
      variables: { id: '1' },
      operationName: 'GetUser',
    });
    expect(resp.data).toEqual(expected);
    expect(resp.errors).toBeUndefined();
  });

  it('executeMutation でセットした応答が返る', async () => {
    const client = new InMemoryGraphQlClient();
    const expected = { createUser: { id: '2', name: 'Bob' } };
    client.setResponse('CreateUser', expected);

    const resp = await client.executeMutation({
      query: 'mutation CreateUser($name: String!) { createUser(name: $name) { id name } }',
      variables: { name: 'Bob' },
      operationName: 'CreateUser',
    });
    expect(resp.data).toEqual(expected);
    expect(resp.errors).toBeUndefined();
  });

  it('未設定のオペレーションはエラーを返す', async () => {
    const client = new InMemoryGraphQlClient();
    const resp = await client.execute({
      query: 'query Unknown { unknown }',
      operationName: 'Unknown',
    });
    expect(resp.data).toBeUndefined();
    expect(resp.errors).toBeDefined();
    expect(resp.errors![0].message).toContain('Unknown');
  });

  it('応答を上書きできる', async () => {
    const client = new InMemoryGraphQlClient();
    client.setResponse('Op', 'first');
    client.setResponse('Op', 'second');

    const resp = await client.execute({
      query: 'query Op { op }',
      operationName: 'Op',
    });
    expect(resp.data).toBe('second');
  });

  it('subscribe emits registered events', async () => {
    const client = new InMemoryGraphQlClient();
    client.setSubscriptionEvents('OnUserCreated', [
      { userCreated: { id: '1', name: 'Alice' } },
      { userCreated: { id: '2', name: 'Bob' } },
    ]);

    const subscription: GraphQlQuery = {
      query: 'subscription { userCreated { id name } }',
      operationName: 'OnUserCreated',
    };

    const results: GraphQlResponse<unknown>[] = [];
    for await (const event of client.subscribe(subscription)) {
      results.push(event);
    }
    expect(results).toHaveLength(2);
    expect(results[0].data).toBeDefined();
  });
});

describe('GraphQlHttpClient', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  it('execute で GraphQL レスポンスを返す', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => ({ data: { health: 'ok' } }),
    } as Response);

    const client = new GraphQlHttpClient('http://localhost:8080/graphql');
    const resp = await client.execute<{ health: string }>({ query: '{ health }' });

    expect(resp.data?.health).toBe('ok');
    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8080/graphql',
      expect.objectContaining({ method: 'POST' }),
    );
  });

  it('HTTP エラー時に ClientError を投げる', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 503,
      json: async () => ({}),
    } as Response);

    const client = new GraphQlHttpClient('http://localhost:8080/graphql');
    await expect(client.execute({ query: '{ health }' })).rejects.toThrow(
      'GraphQL request failed: 503',
    );
    await mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
      json: async () => ({}),
    } as Response);
    try {
      await client.execute({ query: '{ health }' });
    } catch (e: unknown) {
      expect(e).toBeInstanceOf(ClientError);
      expect((e as ClientError).kind).toBe(ClientErrorKind.Request);
    }
  });
});

describe('ClientError', () => {
  it('request ファクトリメソッドで kind が request になる', () => {
    const err = ClientError.request('connection failed');
    expect(err).toBeInstanceOf(ClientError);
    expect(err).toBeInstanceOf(Error);
    expect(err.kind).toBe(ClientErrorKind.Request);
    expect(err.message).toBe('connection failed');
    expect(err.name).toBe('ClientError');
  });

  it('deserialization ファクトリメソッドで kind が deserialization になる', () => {
    const err = ClientError.deserialization('invalid JSON');
    expect(err.kind).toBe(ClientErrorKind.Deserialization);
    expect(err.message).toBe('invalid JSON');
  });

  it('graphQl ファクトリメソッドで kind が graphql になる', () => {
    const err = ClientError.graphQl('field not found');
    expect(err.kind).toBe(ClientErrorKind.GraphQl);
    expect(err.message).toBe('field not found');
  });

  it('notFound ファクトリメソッドで kind が not_found になる', () => {
    const err = ClientError.notFound('user not found');
    expect(err.kind).toBe(ClientErrorKind.NotFound);
    expect(err.message).toBe('user not found');
  });

  it('コンストラクタで直接生成できる', () => {
    const err = new ClientError(ClientErrorKind.Request, 'test');
    expect(err.kind).toBe('request');
    expect(err.message).toBe('test');
  });
});
