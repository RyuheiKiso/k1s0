import { describe, it, expect } from 'vitest';
import {
  createPageResponse,
  encodeCursor,
  decodeCursor,
} from '../src/index.js';

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
    const id = 'abc-123';
    const cursor = encodeCursor(id);
    expect(decodeCursor(cursor)).toBe(id);
  });

  it('base64文字列を返す', () => {
    const cursor = encodeCursor('test-id');
    expect(cursor).toBe(btoa('test-id'));
  });

  it('空文字でも動作する', () => {
    expect(decodeCursor(encodeCursor(''))).toBe('');
  });
});
