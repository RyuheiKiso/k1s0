import { describe, it, expect, beforeEach } from 'vitest';
import { ComponentRegistry } from './registry.js';
import type { Component, ComponentStatus } from './component.js';

// テスト用のシンプルなコンポーネント実装。
class TestComponent implements Component {
  readonly name: string;
  readonly componentType = 'test';
  private _status: ComponentStatus = 'uninitialized';

  constructor(name: string) {
    this.name = name;
  }

  async init(): Promise<void> {
    this._status = 'ready';
  }

  async close(): Promise<void> {
    this._status = 'closed';
  }

  async status(): Promise<ComponentStatus> {
    return this._status;
  }

  metadata(): Record<string, string> {
    return {};
  }
}

// ComponentRegistry のテスト: 登録・取得・重複エラー・一括操作の動作を検証する。
describe('ComponentRegistry', () => {
  let registry: ComponentRegistry;

  beforeEach(() => {
    registry = new ComponentRegistry();
  });

  // コンポーネントを登録して名前で取得できることを確認する。
  it('コンポーネントを登録して取得できること', () => {
    const c = new TestComponent('comp-1');
    registry.register(c);

    const got = registry.get('comp-1');
    expect(got).toBe(c);
  });

  // 同名のコンポーネントを重複登録するとエラーがスローされることを確認する。
  it('同名コンポーネントの重複登録でエラーがスローされること', () => {
    registry.register(new TestComponent('dup'));
    expect(() => registry.register(new TestComponent('dup'))).toThrow(
      "コンポーネント 'dup' は既に登録されています"
    );
  });

  // 存在しない名前でコンポーネントを取得すると undefined が返ることを確認する。
  it('未登録のコンポーネントは undefined を返すこと', () => {
    expect(registry.get('missing')).toBeUndefined();
  });

  // initAll が全コンポーネントを初期化することを確認する。
  it('initAll で全コンポーネントが初期化されること', async () => {
    const a = new TestComponent('a');
    const b = new TestComponent('b');
    registry.register(a);
    registry.register(b);

    await registry.initAll();

    expect(await a.status()).toBe('ready');
    expect(await b.status()).toBe('ready');
  });

  // closeAll が全コンポーネントをクローズすることを確認する。
  it('closeAll で全コンポーネントがクローズされること', async () => {
    const a = new TestComponent('a');
    registry.register(a);
    await a.init();

    await registry.closeAll();

    expect(await a.status()).toBe('closed');
  });

  // statusAll が全コンポーネントのステータスを返すことを確認する。
  it('statusAll が全コンポーネントのステータスを返すこと', async () => {
    registry.register(new TestComponent('a'));
    registry.register(new TestComponent('b'));
    await registry.initAll();

    const statuses = await registry.statusAll();
    expect(statuses['a']).toBe('ready');
    expect(statuses['b']).toBe('ready');
  });
});
