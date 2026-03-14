import { describe, it, expect, beforeEach } from 'vitest';
import { InMemoryInputBinding, InMemoryOutputBinding } from './inmemory_binding.js';
import { ComponentError } from './errors.js';

// InMemoryInputBinding のテスト: インメモリキューからデータを読み取る入力バインディングの動作を検証する。
describe('InMemoryInputBinding', () => {
  let binding: InMemoryInputBinding;

  beforeEach(() => {
    binding = new InMemoryInputBinding();
  });

  it('初期状態は uninitialized', async () => {
    expect(await binding.status()).toBe('uninitialized');
  });

  it('init 後は ready になる', async () => {
    await binding.init();
    expect(await binding.status()).toBe('ready');
  });

  it('close 後は closed になりキューがクリアされる', async () => {
    await binding.init();
    binding.push({ data: new Uint8Array([1]), metadata: {} });
    await binding.close();
    expect(await binding.status()).toBe('closed');
  });

  it('デフォルト name は inmemory-input-binding', () => {
    expect(binding.name).toBe('inmemory-input-binding');
    expect(binding.componentType).toBe('binding.input');
  });

  it('コンストラクタで name を指定できる', () => {
    const named = new InMemoryInputBinding('custom-input');
    expect(named.name).toBe('custom-input');
  });

  it('metadata は backend=memory と direction=input を返す', () => {
    expect(binding.metadata()).toEqual({ backend: 'memory', direction: 'input' });
  });

  it('push したデータを read で取得できる（FIFO順）', async () => {
    await binding.init();
    binding.push({ data: new Uint8Array([1]), metadata: { seq: '1' } });
    binding.push({ data: new Uint8Array([2]), metadata: { seq: '2' } });

    const first = await binding.read();
    const second = await binding.read();

    expect(first.data).toEqual(new Uint8Array([1]));
    expect(first.metadata?.seq).toBe('1');
    expect(second.data).toEqual(new Uint8Array([2]));
  });

  it('キューが空のときに read すると ComponentError をスローする', async () => {
    await binding.init();
    await expect(binding.read()).rejects.toBeInstanceOf(ComponentError);
  });
});

// InMemoryOutputBinding のテスト: invoke の呼び出し履歴記録とモックレスポンス設定機能を検証する。
describe('InMemoryOutputBinding', () => {
  let binding: InMemoryOutputBinding;

  beforeEach(() => {
    binding = new InMemoryOutputBinding();
  });

  it('初期状態は uninitialized', async () => {
    expect(await binding.status()).toBe('uninitialized');
  });

  it('init 後は ready になる', async () => {
    await binding.init();
    expect(await binding.status()).toBe('ready');
  });

  it('close 後は closed になり呼び出し履歴がクリアされる', async () => {
    await binding.init();
    await binding.invoke('op', new Uint8Array([1]));
    await binding.close();
    expect(await binding.status()).toBe('closed');
    expect(binding.lastInvocation()).toBeUndefined();
  });

  it('デフォルト name は inmemory-output-binding', () => {
    expect(binding.name).toBe('inmemory-output-binding');
    expect(binding.componentType).toBe('binding.output');
  });

  it('コンストラクタで name を指定できる', () => {
    const named = new InMemoryOutputBinding('custom-output');
    expect(named.name).toBe('custom-output');
  });

  it('metadata は backend=memory と direction=output を返す', () => {
    expect(binding.metadata()).toEqual({ backend: 'memory', direction: 'output' });
  });

  it('invoke の前は lastInvocation が undefined', async () => {
    await binding.init();
    expect(binding.lastInvocation()).toBeUndefined();
  });

  it('invoke が呼び出し履歴を記録する', async () => {
    await binding.init();
    await binding.invoke('send', new Uint8Array([1, 2]), { key: 'val' });

    const inv = binding.lastInvocation();
    expect(inv).toBeDefined();
    expect(inv!.operation).toBe('send');
    expect(inv!.data).toEqual(new Uint8Array([1, 2]));
    expect(inv!.metadata).toEqual({ key: 'val' });
  });

  it('invoke はデフォルトで入力データをそのまま返す', async () => {
    await binding.init();
    const resp = await binding.invoke('op', new Uint8Array([42]));
    expect(resp.data).toEqual(new Uint8Array([42]));
  });

  it('setResponse でモックレスポンスを設定できる', async () => {
    await binding.init();
    binding.setResponse({ data: new Uint8Array([99]), metadata: {} });
    const resp = await binding.invoke('op', new Uint8Array([1]));
    expect(resp.data).toEqual(new Uint8Array([99]));
  });

  it('setResponse でモックエラーを設定できる', async () => {
    await binding.init();
    const mockError = new Error('invoke error');
    binding.setResponse(undefined, mockError);
    await expect(binding.invoke('op', new Uint8Array([1]))).rejects.toBe(mockError);
  });

  it('allInvocations で全履歴を取得できる', async () => {
    await binding.init();
    await binding.invoke('op1', new Uint8Array([1]));
    await binding.invoke('op2', new Uint8Array([2]));

    const all = binding.allInvocations();
    expect(all).toHaveLength(2);
    expect(all[0].operation).toBe('op1');
    expect(all[1].operation).toBe('op2');
  });

  it('reset で履歴とモック設定をクリアできる', async () => {
    await binding.init();
    binding.setResponse({ data: new Uint8Array([99]), metadata: {} });
    await binding.invoke('op', new Uint8Array([1]));

    binding.reset();

    expect(binding.lastInvocation()).toBeUndefined();
    // reset 後はデフォルト動作（入力をそのまま返す）に戻る
    const resp = await binding.invoke('op', new Uint8Array([7]));
    expect(resp.data).toEqual(new Uint8Array([7]));
  });
});
