// 本ファイルは docs-site の App コンポーネントの最小単体テスト。
// vitest 環境で App エクスポートが関数として配置されていることを確認する。

import { describe, it, expect } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('App コンポーネントは関数として定義されている', () => {
    expect(typeof App).toBe('function');
  });
});
