import { vi, describe, it, expect, beforeEach } from 'vitest';
import {
  subjectName,
  validateSchemaRegistryConfig,
  HttpSchemaRegistryClient,
  NotFoundError,
  isNotFound,
  SchemaRegistryError,
} from '../src/index.js';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function mockResponse(body: unknown, status = 200) {
  return Promise.resolve({
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
  } as Response);
}

describe('subjectName', () => {
  it('topic-value を生成する', () => {
    expect(subjectName('user.created.v1', 'value')).toBe('user.created.v1-value');
  });

  it('topic-key を生成する', () => {
    expect(subjectName('user.created.v1', 'key')).toBe('user.created.v1-key');
  });
});

describe('validateSchemaRegistryConfig', () => {
  it('空URLでエラーを投げる', () => {
    expect(() => validateSchemaRegistryConfig({ url: '' })).toThrow(
      'schema registry URL must not be empty',
    );
  });

  it('有効なURLではエラーを投げない', () => {
    expect(() => validateSchemaRegistryConfig({ url: 'http://localhost:8081' })).not.toThrow();
  });
});

describe('NotFoundError', () => {
  it('正しく生成される', () => {
    const err = new NotFoundError('test-resource');
    expect(err.message).toBe('not found: test-resource');
    expect(err.resource).toBe('test-resource');
    expect(err.name).toBe('NotFoundError');
  });
});

describe('isNotFound', () => {
  it('NotFoundError で true を返す', () => {
    expect(isNotFound(new NotFoundError('x'))).toBe(true);
  });

  it('通常の Error で false を返す', () => {
    expect(isNotFound(new Error('x'))).toBe(false);
  });

  it('null で false を返す', () => {
    expect(isNotFound(null)).toBe(false);
  });
});

describe('SchemaRegistryError', () => {
  it('正しいステータスコードを持つ', () => {
    const err = new SchemaRegistryError(500, 'internal error');
    expect(err.statusCode).toBe(500);
    expect(err.message).toBe('schema registry error (status 500): internal error');
    expect(err.name).toBe('SchemaRegistryError');
  });
});

describe('HttpSchemaRegistryClient', () => {
  let client: HttpSchemaRegistryClient;

  beforeEach(() => {
    mockFetch.mockReset();
    client = new HttpSchemaRegistryClient({ url: 'http://localhost:8081' });
  });

  describe('registerSchema', () => {
    it('成功した場合にIDを返す', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ id: 42 }));

      const id = await client.registerSchema('test-value', '{"type":"string"}', 'AVRO');
      expect(id).toBe(42);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8081/subjects/test-value/versions',
        expect.objectContaining({ method: 'POST' }),
      );
    });

    it('404の場合NotFoundErrorを投げる', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ message: 'not found' }, 404));

      await expect(
        client.registerSchema('unknown-value', '{"type":"string"}', 'AVRO'),
      ).rejects.toThrow(NotFoundError);
    });
  });

  describe('getSchemaById', () => {
    it('成功した場合にスキーマを返す', async () => {
      mockFetch.mockReturnValueOnce(
        mockResponse({
          id: 1,
          subject: 'test-value',
          version: 1,
          schema: '{"type":"string"}',
          schemaType: 'AVRO',
        }),
      );

      const schema = await client.getSchemaById(1);
      expect(schema.id).toBe(1);
      expect(schema.schema).toBe('{"type":"string"}');
    });

    it('404の場合NotFoundErrorを投げる', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ message: 'not found' }, 404));

      await expect(client.getSchemaById(999)).rejects.toThrow(NotFoundError);
    });
  });

  describe('getLatestSchema', () => {
    it('成功した場合にスキーマを返す', async () => {
      mockFetch.mockReturnValueOnce(
        mockResponse({
          id: 5,
          subject: 'test-value',
          version: 3,
          schema: '{"type":"string"}',
          schemaType: 'AVRO',
        }),
      );

      const schema = await client.getLatestSchema('test-value');
      expect(schema.version).toBe(3);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8081/subjects/test-value/versions/latest',
        expect.any(Object),
      );
    });

    it('404の場合NotFoundErrorを投げる', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ message: 'not found' }, 404));

      await expect(client.getLatestSchema('unknown')).rejects.toThrow(NotFoundError);
    });
  });

  describe('getSchemaVersion', () => {
    it('成功した場合にスキーマを返す', async () => {
      mockFetch.mockReturnValueOnce(
        mockResponse({
          id: 5,
          subject: 'test-value',
          version: 2,
          schema: '{"type":"string"}',
          schemaType: 'AVRO',
        }),
      );

      const schema = await client.getSchemaVersion('test-value', 2);
      expect(schema.version).toBe(2);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8081/subjects/test-value/versions/2',
        expect.any(Object),
      );
    });
  });

  describe('listSubjects', () => {
    it('成功した場合にサブジェクト一覧を返す', async () => {
      mockFetch.mockReturnValueOnce(mockResponse(['subject-1', 'subject-2']));

      const subjects = await client.listSubjects();
      expect(subjects).toEqual(['subject-1', 'subject-2']);
    });

    it('空の場合に空配列を返す', async () => {
      mockFetch.mockReturnValueOnce(mockResponse([]));

      const subjects = await client.listSubjects();
      expect(subjects).toEqual([]);
    });
  });

  describe('checkCompatibility', () => {
    it('互換性ありの場合にtrueを返す', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ is_compatible: true }));

      const result = await client.checkCompatibility('test-value', '{"type":"string"}');
      expect(result).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8081/compatibility/subjects/test-value/versions/latest',
        expect.objectContaining({ method: 'POST' }),
      );
    });

    it('互換性なしの場合にfalseを返す', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ is_compatible: false }));

      const result = await client.checkCompatibility('test-value', '{"type":"int"}');
      expect(result).toBe(false);
    });
  });

  describe('healthCheck', () => {
    it('成功した場合にエラーを投げない', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({}));

      await expect(client.healthCheck()).resolves.toBeUndefined();
    });

    it('失敗した場合にSchemaRegistryErrorを投げる', async () => {
      mockFetch.mockReturnValueOnce(mockResponse({ message: 'unavailable' }, 503));

      await expect(client.healthCheck()).rejects.toThrow(SchemaRegistryError);
    });
  });
});
