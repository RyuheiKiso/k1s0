import { describe, it, expect } from 'vitest';
import { InMemoryGraphQlClient } from '../src/index.js';
import type { GraphQlQuery, GraphQlResponse } from '../src/index.js';

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
