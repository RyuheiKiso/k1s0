// 最小 smoke test。

import { describe, it, expect } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('App コンポーネントは関数として定義されている', () => {
    expect(typeof App).toBe('function');
  });
});
