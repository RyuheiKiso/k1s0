// 最小 smoke test（コンポーネントが import 可能なことを検証）。
//
// React DOM レンダリングが必要な深いテストは @testing-library/react を導入してから（リリース時点 で実装）。

import { describe, it, expect } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('App コンポーネントは関数として定義されている', () => {
    expect(typeof App).toBe('function');
  });
});
