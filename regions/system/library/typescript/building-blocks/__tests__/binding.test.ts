import { describe, it, expect, vi } from 'vitest';
import type { BindingData, BindingResponse, InputBinding, OutputBinding } from '../src/binding.js';
import type { ComponentStatus } from '../src/component.js';

describe('BindingData', () => {
  it('should create binding data with data and metadata', () => {
    const bd: BindingData = {
      data: new Uint8Array([1, 2, 3]),
      metadata: { source: 'queue' },
    };

    expect(bd.data).toBeInstanceOf(Uint8Array);
    expect(bd.data).toHaveLength(3);
    expect(bd.metadata).toEqual({ source: 'queue' });
  });

  it('should support empty data and metadata', () => {
    const bd: BindingData = {
      data: new Uint8Array(),
      metadata: {},
    };

    expect(bd.data).toHaveLength(0);
    expect(bd.metadata).toEqual({});
  });
});

describe('BindingResponse', () => {
  it('should create binding response with data and metadata', () => {
    const resp: BindingResponse = {
      data: new Uint8Array([10, 20]),
      metadata: { status: 'ok' },
    };

    expect(resp.data).toEqual(new Uint8Array([10, 20]));
    expect(resp.metadata).toEqual({ status: 'ok' });
  });
});

describe('InputBinding interface', () => {
  it('should be implementable with read method', async () => {
    const mock: InputBinding = {
      name: 'test-input',
      componentType: 'binding',
      init: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      read: vi.fn().mockResolvedValue({
        data: new Uint8Array([99]),
        metadata: { queue: 'orders' },
      } satisfies BindingData),
    };

    const result = await mock.read();
    expect(result.data).toEqual(new Uint8Array([99]));
    expect(result.metadata).toEqual({ queue: 'orders' });
  });
});

describe('OutputBinding interface', () => {
  it('should be implementable with invoke method', async () => {
    const mock: OutputBinding = {
      name: 'test-output',
      componentType: 'binding',
      init: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      invoke: vi.fn().mockResolvedValue({
        data: new Uint8Array([1]),
        metadata: { result: 'success' },
      } satisfies BindingResponse),
    };

    const resp = await mock.invoke('create', new Uint8Array([5]), { key: 'val' });
    expect(resp.data).toEqual(new Uint8Array([1]));
    expect(resp.metadata).toEqual({ result: 'success' });
    expect(mock.invoke).toHaveBeenCalledWith('create', expect.any(Uint8Array), { key: 'val' });
  });

  it('should support invoke without optional metadata', async () => {
    const mock: OutputBinding = {
      name: 'output-no-meta',
      componentType: 'binding',
      init: vi.fn(),
      close: vi.fn(),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: () => ({}),
      invoke: vi.fn().mockResolvedValue({
        data: new Uint8Array(),
        metadata: {},
      }),
    };

    await mock.invoke('delete', new Uint8Array());
    expect(mock.invoke).toHaveBeenCalledWith('delete', expect.any(Uint8Array));
  });
});
