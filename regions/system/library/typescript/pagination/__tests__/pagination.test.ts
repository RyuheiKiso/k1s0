import { describe, it, expect } from 'vitest';
import {
  createPageResponse,
  encodeCursor,
  decodeCursor,
  validatePerPage,
  PerPageValidationError,
  defaultPageRequest,
  pageOffset,
  hasNextPage,
} from '../src/index.js';
import type { CursorRequest, CursorMeta, PaginationMeta } from '../src/index.js';

describe('createPageResponse', () => {
  it('正しいページレスポンスを生成する', () => {
    const resp = createPageResponse([1, 2, 3], 10, { page: 1, perPage: 3 });
    expect(resp.items).toEqual([1, 2, 3]);
    expect(resp.total).toBe(10);
    expect(resp.page).toBe(1);
    expect(resp.perPage).toBe(3);
    expect(resp.totalPages).toBe(4);
  });

  it('アイテムが空の場合にtotalPages=0を返す', () => {
    const resp = createPageResponse([], 0, { page: 1, perPage: 10 });
    expect(resp.items).toEqual([]);
    expect(resp.totalPages).toBe(0);
  });

  it('totalPagesを切り上げる', () => {
    const resp = createPageResponse([1], 11, { page: 1, perPage: 5 });
    expect(resp.totalPages).toBe(3);
  });
});

describe('encodeCursor / decodeCursor', () => {
  it('エンコードとデコードが可逆である', () => {
    const sortKey = '2024-01-15';
    const id = 'abc-123';
    const cursor = encodeCursor(sortKey, id);
    const decoded = decodeCursor(cursor);
    expect(decoded.sortKey).toBe(sortKey);
    expect(decoded.id).toBe(id);
  });

  it('base64url文字列（no padding）を返す', () => {
    const cursor = encodeCursor('key', 'test-id');
    const expected = btoa('key|test-id').replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
    expect(cursor).toBe(expected);
  });

  it('セパレータがないカーソルはエラーになる', () => {
    const legacy = btoa('noseparator');
    expect(() => decodeCursor(legacy)).toThrow('missing separator');
  });
});

describe('validatePerPage', () => {
  it('有効な値を受け入れる', () => {
    expect(validatePerPage(1)).toBe(1);
    expect(validatePerPage(50)).toBe(50);
    expect(validatePerPage(100)).toBe(100);
  });

  it('0はエラーになる', () => {
    expect(() => validatePerPage(0)).toThrow(PerPageValidationError);
  });

  it('最大値超過はエラーになる', () => {
    expect(() => validatePerPage(101)).toThrow(PerPageValidationError);
  });
});

describe('CursorRequest type', () => {
  it('型が正しく使える', () => {
    const req: CursorRequest = { limit: 20 };
    expect(req.limit).toBe(20);
    expect(req.cursor).toBeUndefined();
  });
});

describe('CursorMeta type', () => {
  it('型が正しく使える', () => {
    const meta: CursorMeta = { nextCursor: 'next', hasMore: true };
    expect(meta.nextCursor).toBe('next');
    expect(meta.hasMore).toBe(true);
  });
});

describe('PaginationMeta type', () => {
  it('型が正しく使える', () => {
    const meta: PaginationMeta = { total: 100, page: 2, perPage: 10, totalPages: 10 };
    expect(meta.total).toBe(100);
    expect(meta.totalPages).toBe(10);
  });
});

describe('defaultPageRequest', () => {
  it('page=1, perPage=20を返す', () => {
    const req = defaultPageRequest();
    expect(req.page).toBe(1);
    expect(req.perPage).toBe(20);
  });
});

describe('pageOffset', () => {
  it('page=1のオフセットは0', () => {
    expect(pageOffset({ page: 1, perPage: 20 })).toBe(0);
  });

  it('page=2のオフセットはperPage', () => {
    expect(pageOffset({ page: 2, perPage: 20 })).toBe(20);
  });

  it('page=3, perPage=10のオフセットは20', () => {
    expect(pageOffset({ page: 3, perPage: 10 })).toBe(20);
  });
});

describe('hasNextPage', () => {
  it('次のページがある場合trueを返す', () => {
    expect(hasNextPage({ page: 1, perPage: 10 }, 15)).toBe(true);
  });

  it('最後のページの場合falseを返す', () => {
    expect(hasNextPage({ page: 2, perPage: 10 }, 20)).toBe(false);
  });

  it('最後のページを超えた場合falseを返す', () => {
    expect(hasNextPage({ page: 3, perPage: 10 }, 20)).toBe(false);
  });

  it('ちょうど次のページがある境界値', () => {
    expect(hasNextPage({ page: 1, perPage: 10 }, 11)).toBe(true);
  });
});
