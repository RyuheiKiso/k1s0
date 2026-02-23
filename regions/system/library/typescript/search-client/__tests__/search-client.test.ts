import { describe, it, expect } from 'vitest';
import { InMemorySearchClient, SearchError } from '../src/index.js';
import type { IndexDocument, SearchQuery, IndexMapping } from '../src/index.js';

function makeMapping(): IndexMapping {
  return { fields: { name: { type: 'text' }, price: { type: 'integer' } } };
}

describe('InMemorySearchClient', () => {
  it('インデックスを作成しドキュメントを登録できる', async () => {
    const client = new InMemorySearchClient();
    await client.createIndex('products', makeMapping());

    const doc: IndexDocument = { id: 'p-1', fields: { name: 'Rust Programming' } };
    const result = await client.indexDocument('products', doc);
    expect(result.id).toBe('p-1');
    expect(result.version).toBe(1);
  });

  it('バルクインデックスが成功する', async () => {
    const client = new InMemorySearchClient();
    await client.createIndex('items', makeMapping());

    const docs: IndexDocument[] = [
      { id: 'i-1', fields: { name: 'Item 1' } },
      { id: 'i-2', fields: { name: 'Item 2' } },
    ];
    const result = await client.bulkIndex('items', docs);
    expect(result.successCount).toBe(2);
    expect(result.failedCount).toBe(0);
    expect(result.failures).toHaveLength(0);
  });

  it('全文検索ができる', async () => {
    const client = new InMemorySearchClient();
    await client.createIndex('products', makeMapping());
    await client.indexDocument('products', { id: 'p-1', fields: { name: 'Rust Programming' } });
    await client.indexDocument('products', { id: 'p-2', fields: { name: 'Go Language' } });

    const query: SearchQuery = { query: 'Rust', facets: ['name'], page: 0, size: 20 };
    const result = await client.search('products', query);
    expect(result.total).toBe(1);
    expect(result.hits).toHaveLength(1);
    expect(result.facets).toHaveProperty('name');
  });

  it('存在しないインデックスで検索するとエラーになる', async () => {
    const client = new InMemorySearchClient();
    await expect(
      client.search('nonexistent', { query: 'test' }),
    ).rejects.toThrow(SearchError);
  });

  it('存在しないインデックスにドキュメント登録するとエラーになる', async () => {
    const client = new InMemorySearchClient();
    await expect(
      client.indexDocument('nonexistent', { id: '1', fields: {} }),
    ).rejects.toThrow(SearchError);
  });

  it('ドキュメントを削除できる', async () => {
    const client = new InMemorySearchClient();
    await client.createIndex('products', makeMapping());
    await client.indexDocument('products', { id: 'p-1', fields: { name: 'Test' } });

    await client.deleteDocument('products', 'p-1');
    expect(client.documentCount('products')).toBe(0);
  });

  it('空クエリで全件取得できる', async () => {
    const client = new InMemorySearchClient();
    await client.createIndex('items', makeMapping());
    await client.indexDocument('items', { id: 'i-1', fields: { name: 'Item' } });

    const result = await client.search('items', { query: '' });
    expect(result.total).toBe(1);
  });

  it('SearchErrorのcodeプロパティが正しい', () => {
    const err = new SearchError('test', 'INDEX_NOT_FOUND');
    expect(err.code).toBe('INDEX_NOT_FOUND');
    expect(err.name).toBe('SearchError');
    expect(err.message).toBe('test');
  });

  it('ページネーションが機能する', async () => {
    const client = new InMemorySearchClient();
    await client.createIndex('items', makeMapping());
    for (let i = 0; i < 5; i++) {
      await client.indexDocument('items', { id: `i-${i}`, fields: { name: `Item ${i}` } });
    }

    const result = await client.search('items', { query: '', page: 1, size: 2 });
    expect(result.hits).toHaveLength(2);
  });
});
