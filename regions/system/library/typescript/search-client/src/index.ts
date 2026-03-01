export interface Filter {
  field: string;
  operator: 'eq' | 'lt' | 'gt' | 'range' | 'in';
  value: unknown;
  valueTo?: unknown;
}

export interface FacetBucket {
  value: string;
  count: number;
}

export interface SearchQuery {
  query: string;
  filters?: Filter[];
  facets?: string[];
  page?: number;
  size?: number;
}

export interface SearchResult<T = Record<string, unknown>> {
  hits: T[];
  total: number;
  facets: Record<string, FacetBucket[]>;
  tookMs: number;
}

export interface IndexDocument {
  id: string;
  fields: Record<string, unknown>;
}

export interface IndexResult {
  id: string;
  version: number;
}

export interface BulkFailure {
  id: string;
  error: string;
}

export interface BulkResult {
  successCount: number;
  failedCount: number;
  failures: BulkFailure[];
}

export interface FieldMapping {
  type: string;
  indexed?: boolean;
}

export interface IndexMapping {
  fields: Record<string, FieldMapping>;
}

export interface SearchClient {
  indexDocument(index: string, doc: IndexDocument): Promise<IndexResult>;
  bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult>;
  search<T = Record<string, unknown>>(index: string, query: SearchQuery): Promise<SearchResult<T>>;
  deleteDocument(index: string, id: string): Promise<void>;
  createIndex(name: string, mapping: IndexMapping): Promise<void>;
}

export class SearchError extends Error {
  constructor(
    message: string,
    public readonly code: 'INDEX_NOT_FOUND' | 'INVALID_QUERY' | 'SERVER_ERROR' | 'TIMEOUT',
  ) {
    super(message);
    this.name = 'SearchError';
  }
}

export class InMemorySearchClient implements SearchClient {
  private indexes = new Map<string, IndexDocument[]>();

  async createIndex(name: string, _mapping: IndexMapping): Promise<void> {
    this.indexes.set(name, []);
  }

  async indexDocument(index: string, doc: IndexDocument): Promise<IndexResult> {
    const docs = this.indexes.get(index);
    if (!docs) {
      throw new SearchError(`Index not found: ${index}`, 'INDEX_NOT_FOUND');
    }
    docs.push(doc);
    return { id: doc.id, version: docs.length };
  }

  async bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult> {
    const existing = this.indexes.get(index);
    if (!existing) {
      throw new SearchError(`Index not found: ${index}`, 'INDEX_NOT_FOUND');
    }
    existing.push(...docs);
    return { successCount: docs.length, failedCount: 0, failures: [] };
  }

  async search<T = Record<string, unknown>>(index: string, query: SearchQuery): Promise<SearchResult<T>> {
    const docs = this.indexes.get(index);
    if (!docs) {
      throw new SearchError(`Index not found: ${index}`, 'INDEX_NOT_FOUND');
    }

    let filtered = docs;
    if (query.query) {
      filtered = docs.filter((doc) =>
        Object.values(doc.fields).some(
          (v) => typeof v === 'string' && v.includes(query.query),
        ),
      );
    }

    const page = query.page ?? 0;
    const size = query.size ?? 20;
    const start = page * size;
    const paged = filtered.slice(start, start + size);

    const facets: Record<string, FacetBucket[]> = {};
    for (const f of query.facets ?? []) {
      facets[f] = [{ value: 'default', count: paged.length }];
    }

    return {
      hits: paged as unknown as T[],
      total: filtered.length,
      facets,
      tookMs: 1,
    };
  }

  async deleteDocument(index: string, id: string): Promise<void> {
    const docs = this.indexes.get(index);
    if (!docs) return;
    const idx = docs.findIndex((d) => d.id === id);
    if (idx !== -1) docs.splice(idx, 1);
  }

  documentCount(index: string): number {
    return this.indexes.get(index)?.length ?? 0;
  }
}

// HTTP レスポンスのボディ型
interface IndexResultResponse {
  id: string;
  version: number;
}

interface BulkResultResponse {
  successCount: number;
  failedCount: number;
  failures: BulkFailure[];
}

interface SearchResultResponse<T> {
  hits: T[];
  total: number;
  facets: Record<string, FacetBucket[]>;
  tookMs: number;
}

/**
 * GrpcSearchClient は search-server への HTTP クライアント実装。
 * HTTP/JSON API 経由で search-server と通信する。
 */
export class GrpcSearchClient implements SearchClient {
  private readonly baseUrl: string;

  constructor(serverUrl: string) {
    const url = serverUrl.startsWith('http://') || serverUrl.startsWith('https://')
      ? serverUrl
      : `http://${serverUrl}`;
    this.baseUrl = url.replace(/\/$/, '');
  }

  async createIndex(name: string, mapping: IndexMapping): Promise<void> {
    const resp = await fetch(`${this.baseUrl}/api/v1/indexes/${encodeURIComponent(name)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, mapping }),
    });
    if (!resp.ok) {
      const text = await resp.text();
      throw this.parseError('create_index', resp.status, text);
    }
  }

  async indexDocument(index: string, doc: IndexDocument): Promise<IndexResult> {
    const resp = await fetch(
      `${this.baseUrl}/api/v1/indexes/${encodeURIComponent(index)}/documents`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(doc),
      },
    );
    if (!resp.ok) {
      const text = await resp.text();
      throw this.parseError('index_document', resp.status, text);
    }
    const data = (await resp.json()) as IndexResultResponse;
    return { id: data.id, version: data.version };
  }

  async bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult> {
    const resp = await fetch(
      `${this.baseUrl}/api/v1/indexes/${encodeURIComponent(index)}/documents/_bulk`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ documents: docs }),
      },
    );
    if (!resp.ok) {
      const text = await resp.text();
      throw this.parseError('bulk_index', resp.status, text);
    }
    const data = (await resp.json()) as BulkResultResponse;
    return {
      successCount: data.successCount,
      failedCount: data.failedCount,
      failures: data.failures ?? [],
    };
  }

  async search<T = Record<string, unknown>>(
    index: string,
    query: SearchQuery,
  ): Promise<SearchResult<T>> {
    const resp = await fetch(
      `${this.baseUrl}/api/v1/indexes/${encodeURIComponent(index)}/_search`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(query),
      },
    );
    if (!resp.ok) {
      const text = await resp.text();
      throw this.parseError('search', resp.status, text);
    }
    const data = (await resp.json()) as SearchResultResponse<T>;
    return {
      hits: data.hits,
      total: data.total,
      facets: data.facets ?? {},
      tookMs: data.tookMs,
    };
  }

  async deleteDocument(index: string, id: string): Promise<void> {
    const resp = await fetch(
      `${this.baseUrl}/api/v1/indexes/${encodeURIComponent(index)}/documents/${encodeURIComponent(id)}`,
      { method: 'DELETE' },
    );
    if (!resp.ok && resp.status !== 204) {
      const text = await resp.text();
      throw this.parseError('delete_document', resp.status, text);
    }
  }

  /** gRPC チャネル相当のリソースを解放する（HTTP 実装では no-op）。 */
  async close(): Promise<void> {}

  private parseError(op: string, status: number, body: string): SearchError {
    if (status === 404) {
      return new SearchError(`${op}: index not found: ${body}`, 'INDEX_NOT_FOUND');
    }
    if (status === 400) {
      return new SearchError(`${op}: invalid query: ${body}`, 'INVALID_QUERY');
    }
    if (status === 408 || status === 504) {
      return new SearchError(`${op}: timeout`, 'TIMEOUT');
    }
    return new SearchError(`${op}: server error (status=${status}): ${body}`, 'SERVER_ERROR');
  }
}
