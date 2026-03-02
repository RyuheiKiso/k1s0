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
  it('creates a response with computed total pages', () => {
    const resp = createPageResponse([1, 2, 3], 10, { page: 1, perPage: 3 });
    expect(resp.items).toEqual([1, 2, 3]);
    expect(resp.total).toBe(10);
    expect(resp.page).toBe(1);
    expect(resp.perPage).toBe(3);
    expect(resp.totalPages).toBe(4);
  });

  it('returns totalPages=0 when total is zero', () => {
    const resp = createPageResponse([], 0, { page: 1, perPage: 10 });
    expect(resp.items).toEqual([]);
    expect(resp.totalPages).toBe(0);
  });

  it('rounds up total pages', () => {
    const resp = createPageResponse([1], 11, { page: 1, perPage: 5 });
    expect(resp.totalPages).toBe(3);
  });

  it('meta() returns pagination metadata', () => {
    const resp = createPageResponse([1, 2], 11, { page: 2, perPage: 5 });
    expect(resp.meta()).toEqual({
      total: 11,
      page: 2,
      perPage: 5,
      totalPages: 3,
    });
  });
});

describe('encodeCursor / decodeCursor', () => {
  it('encodes and decodes cursor values', () => {
    const sortKey = '2024-01-15';
    const id = 'abc-123';
    const cursor = encodeCursor(sortKey, id);
    const decoded = decodeCursor(cursor);
    expect(decoded.sortKey).toBe(sortKey);
    expect(decoded.id).toBe(id);
  });

  it('throws for cursor without separator', () => {
    const legacy = btoa('noseparator');
    expect(() => decodeCursor(legacy)).toThrow('missing separator');
  });
});

describe('validatePerPage', () => {
  it('accepts valid values', () => {
    expect(validatePerPage(1)).toBe(1);
    expect(validatePerPage(50)).toBe(50);
    expect(validatePerPage(100)).toBe(100);
  });

  it('rejects invalid values', () => {
    expect(() => validatePerPage(0)).toThrow(PerPageValidationError);
    expect(() => validatePerPage(101)).toThrow(PerPageValidationError);
  });
});

describe('CursorRequest type', () => {
  it('can be used without cursor', () => {
    const req: CursorRequest = { limit: 20 };
    expect(req.limit).toBe(20);
    expect(req.cursor).toBeUndefined();
  });
});

describe('CursorMeta type', () => {
  it('can be instantiated with fields', () => {
    const meta: CursorMeta = { nextCursor: 'next', hasMore: true };
    expect(meta.nextCursor).toBe('next');
    expect(meta.hasMore).toBe(true);
  });
});

describe('PaginationMeta type', () => {
  it('can be instantiated with fields', () => {
    const meta: PaginationMeta = { total: 100, page: 2, perPage: 10, totalPages: 10 };
    expect(meta.total).toBe(100);
    expect(meta.totalPages).toBe(10);
  });
});

describe('defaultPageRequest', () => {
  it('returns page=1 and perPage=20', () => {
    const req = defaultPageRequest();
    expect(req.page).toBe(1);
    expect(req.perPage).toBe(20);
  });
});

describe('pageOffset', () => {
  it('computes expected offsets', () => {
    expect(pageOffset({ page: 1, perPage: 20 })).toBe(0);
    expect(pageOffset({ page: 2, perPage: 20 })).toBe(20);
    expect(pageOffset({ page: 3, perPage: 10 })).toBe(20);
  });
});

describe('hasNextPage', () => {
  it('returns whether another page exists', () => {
    expect(hasNextPage({ page: 1, perPage: 10 }, 15)).toBe(true);
    expect(hasNextPage({ page: 2, perPage: 10 }, 20)).toBe(false);
    expect(hasNextPage({ page: 3, perPage: 10 }, 20)).toBe(false);
  });
});
