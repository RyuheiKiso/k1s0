export class IdempotencyError extends Error {
  constructor(
    message: string,
    public readonly code: string,
  ) {
    super(message);
    this.name = 'IdempotencyError';
  }
}

export class DuplicateKeyError extends Error {
  constructor(public readonly key: string) {
    super(`duplicate key: ${key}`);
    this.name = 'DuplicateKeyError';
  }
}

export type IdempotencyStatus = 'pending' | 'completed' | 'failed';

export interface IdempotencyRecord {
  key: string;
  status: IdempotencyStatus;
  responseBody?: string;
  statusCode?: number;
  createdAt: Date;
  expiresAt?: Date;
  completedAt?: Date;
}

export function createIdempotencyRecord(key: string, ttlSecs?: number): IdempotencyRecord {
  const now = new Date();
  return {
    key,
    status: 'pending',
    createdAt: now,
    expiresAt: ttlSecs != null ? new Date(now.getTime() + ttlSecs * 1000) : undefined,
  };
}

export interface IdempotencyStore {
  get(key: string): Promise<IdempotencyRecord | null>;
  insert(record: IdempotencyRecord): Promise<void>;
  update(key: string, status: IdempotencyStatus, body?: string, code?: number): Promise<void>;
  delete(key: string): Promise<boolean>;
}

export class InMemoryIdempotencyStore implements IdempotencyStore {
  private records = new Map<string, IdempotencyRecord>();

  private cleanupExpired(): void {
    const now = new Date();
    for (const [key, record] of this.records) {
      if (record.expiresAt && record.expiresAt <= now) {
        this.records.delete(key);
      }
    }
  }

  async get(key: string): Promise<IdempotencyRecord | null> {
    this.cleanupExpired();
    return this.records.get(key) ?? null;
  }

  async insert(record: IdempotencyRecord): Promise<void> {
    this.cleanupExpired();
    if (this.records.has(record.key)) {
      throw new DuplicateKeyError(record.key);
    }
    this.records.set(record.key, { ...record });
  }

  async update(key: string, status: IdempotencyStatus, body?: string, code?: number): Promise<void> {
    const record = this.records.get(key);
    if (!record) {
      throw new IdempotencyError(`キーが見つかりません: ${key}`, 'NOT_FOUND');
    }
    record.status = status;
    record.responseBody = body;
    record.statusCode = code;
    record.completedAt = new Date();
  }

  async delete(key: string): Promise<boolean> {
    return this.records.delete(key);
  }
}
