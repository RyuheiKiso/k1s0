import { describe, it, expect, vi } from 'vitest';
import type { ComponentStatus, Component } from '../src/component.js';

describe('ComponentStatus', () => {
  it('should accept all valid status values', () => {
    const statuses: ComponentStatus[] = [
      'uninitialized',
      'ready',
      'degraded',
      'closed',
      'error',
    ];
    expect(statuses).toHaveLength(5);
  });
});

describe('Component interface', () => {
  it('should be implementable with all required members', () => {
    const mock: Component = {
      name: 'test-component',
      componentType: 'pubsub',
      init: vi.fn().mockResolvedValue(undefined),
      close: vi.fn().mockResolvedValue(undefined),
      status: vi.fn().mockResolvedValue('ready' as ComponentStatus),
      metadata: vi.fn().mockReturnValue({ version: '1.0.0' }),
    };

    expect(mock.name).toBe('test-component');
    expect(mock.componentType).toBe('pubsub');
    expect(mock.metadata()).toEqual({ version: '1.0.0' });
  });

  it('should support async init and close lifecycle', async () => {
    const initFn = vi.fn().mockResolvedValue(undefined);
    const closeFn = vi.fn().mockResolvedValue(undefined);

    const mock: Component = {
      name: 'lifecycle-test',
      componentType: 'statestore',
      init: initFn,
      close: closeFn,
      status: vi.fn().mockResolvedValue('uninitialized' as ComponentStatus),
      metadata: () => ({}),
    };

    await mock.init();
    expect(initFn).toHaveBeenCalledOnce();

    await mock.close();
    expect(closeFn).toHaveBeenCalledOnce();
  });

  it('should return status asynchronously', async () => {
    const mock: Component = {
      name: 'status-test',
      componentType: 'binding',
      init: vi.fn(),
      close: vi.fn(),
      status: vi.fn().mockResolvedValue('degraded' as ComponentStatus),
      metadata: () => ({}),
    };

    const status = await mock.status();
    expect(status).toBe('degraded');
  });
});
