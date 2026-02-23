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
