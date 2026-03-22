export interface FileMetadata {
  path: string;
  sizeBytes: number;
  contentType: string;
  etag: string;
  lastModified: Date;
  tags: Record<string, string>;
}

export interface PresignedUrl {
  url: string;
  method: 'PUT' | 'GET';
  expiresAt: Date;
  headers: Record<string, string>;
}

export interface FileClient {
  generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl>;
  generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl>;
  delete(path: string): Promise<void>;
  getMetadata(path: string): Promise<FileMetadata>;
  list(prefix: string): Promise<FileMetadata[]>;
  copy(src: string, dst: string): Promise<void>;
}

export class FileClientError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly cause?: Error,
  ) {
    super(message);
    this.name = 'FileClientError';
  }
}

/** ファイルクライアントの設定。S3/AWS SDK 依存を除去し、file-server 経由のみをサポートする。 */
export interface FileClientConfig {
  /** file-server モードのエンドポイント URL */
  serverUrl?: string;
  /** リクエストタイムアウト (ms)。デフォルト 30_000。 */
  timeoutMs?: number;
}

export class InMemoryFileClient implements FileClient {
  private files = new Map<string, FileMetadata>();

  async generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl> {
    this.files.set(path, {
      path,
      sizeBytes: 0,
      contentType,
      etag: '',
      lastModified: new Date(),
      tags: {},
    });
    return {
      url: `https://storage.example.com/upload/${path}`,
      method: 'PUT',
      expiresAt: new Date(Date.now() + expiresInMs),
      headers: {},
    };
  }

  async generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl> {
    if (!this.files.has(path)) {
      throw new FileClientError(`File not found: ${path}`, 'NOT_FOUND');
    }
    return {
      url: `https://storage.example.com/download/${path}`,
      method: 'GET',
      expiresAt: new Date(Date.now() + expiresInMs),
      headers: {},
    };
  }

  async delete(path: string): Promise<void> {
    if (!this.files.has(path)) {
      throw new FileClientError(`File not found: ${path}`, 'NOT_FOUND');
    }
    this.files.delete(path);
  }

  async getMetadata(path: string): Promise<FileMetadata> {
    const meta = this.files.get(path);
    if (!meta) {
      throw new FileClientError(`File not found: ${path}`, 'NOT_FOUND');
    }
    return { ...meta };
  }

  async list(prefix: string): Promise<FileMetadata[]> {
    const result: FileMetadata[] = [];
    for (const [key, meta] of this.files) {
      if (key.startsWith(prefix)) {
        result.push({ ...meta });
      }
    }
    return result;
  }

  async copy(src: string, dst: string): Promise<void> {
    const source = this.files.get(src);
    if (!source) {
      throw new FileClientError(`File not found: ${src}`, 'NOT_FOUND');
    }
    this.files.set(dst, { ...source, path: dst });
  }

  getStoredFiles(): FileMetadata[] {
    return Array.from(this.files.values()).map((f) => ({ ...f }));
  }
}

// ---------------------------------------------------------------------------
// ServerFileClient — file-server 経由の HTTP 実装
// ---------------------------------------------------------------------------

export class ServerFileClient implements FileClient {
  private readonly baseUrl: string;
  private readonly timeoutMs: number;

  constructor(config: FileClientConfig) {
    if (!config.serverUrl) {
      throw new FileClientError('serverUrl が設定されていません', 'INVALID_CONFIG');
    }
    this.baseUrl = config.serverUrl.replace(/\/$/, '');
    this.timeoutMs = config.timeoutMs ?? 30_000;
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    try {
      const res = await fetch(`${this.baseUrl}${path}`, {
        method,
        headers: body ? { 'Content-Type': 'application/json' } : undefined,
        body: body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });
      const text = await res.text();
      if (res.status === 404) throw new FileClientError(text || path, 'NOT_FOUND');
      if (res.status === 401 || res.status === 403) throw new FileClientError(text, 'UNAUTHORIZED');
      if (!res.ok) throw new FileClientError(`HTTP ${res.status}: ${text}`, 'INTERNAL');
      return text ? (JSON.parse(text) as T) : (undefined as unknown as T);
    } catch (e) {
      if (e instanceof FileClientError) throw e;
      throw new FileClientError(String(e), 'CONNECTION_ERROR', e instanceof Error ? e : undefined);
    } finally {
      clearTimeout(timer);
    }
  }

  async generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl> {
    const data = await this.request<{ url: string; method: string; expires_at: string; headers: Record<string, string> }>(
      'POST',
      '/api/v1/files/upload-url',
      { path, content_type: contentType, expires_in_secs: Math.floor(expiresInMs / 1000) },
    );
    return { url: data.url, method: data.method as 'PUT', expiresAt: new Date(data.expires_at), headers: data.headers };
  }

  async generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl> {
    const data = await this.request<{ url: string; method: string; expires_at: string; headers: Record<string, string> }>(
      'POST',
      '/api/v1/files/download-url',
      { path, expires_in_secs: Math.floor(expiresInMs / 1000) },
    );
    return { url: data.url, method: data.method as 'GET', expiresAt: new Date(data.expires_at), headers: data.headers };
  }

  async delete(path: string): Promise<void> {
    await this.request('DELETE', `/api/v1/files/${encodeURIComponent(path)}`);
  }

  async getMetadata(path: string): Promise<FileMetadata> {
    const data = await this.request<{ path: string; size_bytes: number; content_type: string; etag: string; last_modified: string; tags: Record<string, string> }>(
      'GET',
      `/api/v1/files/${encodeURIComponent(path)}/metadata`,
    );
    return {
      path: data.path,
      sizeBytes: data.size_bytes,
      contentType: data.content_type,
      etag: data.etag,
      lastModified: new Date(data.last_modified),
      tags: data.tags,
    };
  }

  async list(prefix: string): Promise<FileMetadata[]> {
    const data = await this.request<Array<{ path: string; size_bytes: number; content_type: string; etag: string; last_modified: string; tags: Record<string, string> }>>(
      'GET',
      `/api/v1/files?prefix=${encodeURIComponent(prefix)}`,
    );
    return data.map((d) => ({
      path: d.path,
      sizeBytes: d.size_bytes,
      contentType: d.content_type,
      etag: d.etag,
      lastModified: new Date(d.last_modified),
      tags: d.tags,
    }));
  }

  async copy(src: string, dst: string): Promise<void> {
    await this.request('POST', '/api/v1/files/copy', { src, dst });
  }
}

// ---------------------------------------------------------------------------
// MockFileClient — テスト用モック実装
// ---------------------------------------------------------------------------

/**
 * MockFileClient は FileClient インターフェースを実装したテスト用モッククラス。
 *
 * jest.fn() を使って各メソッドをオーバーライドすることで、テストコード内で
 * スタブ応答の注入・呼び出し検証が可能。
 *
 * @example
 * ```typescript
 * const mock = new MockFileClient();
 * mock.getMetadata = jest.fn().mockResolvedValue({ path: 'a.png', ... });
 * expect(mock.getMetadata).toHaveBeenCalledWith('a.png');
 * ```
 */
export class MockFileClient implements FileClient {
  async generateUploadUrl(path: string, contentType: string, expiresInMs: number): Promise<PresignedUrl> {
    return {
      url: `https://mock.example.com/upload/${path}`,
      method: 'PUT',
      expiresAt: new Date(Date.now() + expiresInMs),
      headers: {},
    };
  }

  async generateDownloadUrl(path: string, expiresInMs: number): Promise<PresignedUrl> {
    return {
      url: `https://mock.example.com/download/${path}`,
      method: 'GET',
      expiresAt: new Date(Date.now() + expiresInMs),
      headers: {},
    };
  }

  async delete(_path: string): Promise<void> {
    // デフォルト実装は no-op（jest.fn() で上書き可能）
  }

  async getMetadata(path: string): Promise<FileMetadata> {
    return {
      path,
      sizeBytes: 0,
      contentType: 'application/octet-stream',
      etag: '',
      lastModified: new Date(),
      tags: {},
    };
  }

  async list(_prefix: string): Promise<FileMetadata[]> {
    return [];
  }

  async copy(_src: string, _dst: string): Promise<void> {
    // デフォルト実装は no-op（jest.fn() で上書き可能）
  }
}

