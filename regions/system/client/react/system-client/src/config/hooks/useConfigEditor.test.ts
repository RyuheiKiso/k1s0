import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import axios from 'axios';
import { useConfigEditor } from './useConfigEditor';
import type { ConfigEditorSchema, ServiceConfigResultResponse } from '../types';

const mockSchema: ConfigEditorSchema = {
  service: 'test-service',
  namespace_prefix: 'test',
  categories: [
    {
      id: 'general',
      label: 'General',
      namespaces: ['test.general'],
      fields: [
        { key: 'timeout', label: 'Timeout', type: 'integer', default: 30 },
        { key: 'name', label: 'Name', type: 'string', default: 'default-name' },
      ],
    },
    {
      id: 'advanced',
      label: 'Advanced',
      namespaces: ['test.advanced'],
      fields: [
        { key: 'timeout', label: 'Advanced Timeout', type: 'integer', default: 15 },
      ],
    },
  ],
};

const mockValues: ServiceConfigResultResponse = {
  service_name: 'test-service',
  entries: [
    { namespace: 'test.general', key: 'timeout', value: 60, version: 3 },
    { namespace: 'test.general', key: 'name', value: 'current-name', version: 4 },
    { namespace: 'test.advanced', key: 'timeout', value: 180, version: 8 },
  ],
};

const receivedUpdates: Array<{
  namespace: string;
  key: string;
  body: { value: unknown; version: number };
}> = [];

const server = setupServer(
  http.get('http://localhost/api/v1/config-schema/:service', () => {
    return HttpResponse.json(mockSchema);
  }),
  http.get('http://localhost/api/v1/config/services/:service', () => {
    return HttpResponse.json(mockValues);
  }),
  http.put('http://localhost/api/v1/config/:namespace/:key', async ({ params, request }) => {
    const body = await request.json() as { value: unknown; version: number };
    receivedUpdates.push({
      namespace: decodeURIComponent(String(params.namespace)),
      key: decodeURIComponent(String(params.key)),
      body,
    });

    return HttpResponse.json({
      namespace: decodeURIComponent(String(params.namespace)),
      key: decodeURIComponent(String(params.key)),
      value: body.value,
      version: body.version + 1,
    });
  }),
);

beforeAll(() => server.listen());
afterEach(() => {
  receivedUpdates.length = 0;
  server.resetHandlers();
});
afterAll(() => server.close());

const client = axios.create({ baseURL: 'http://localhost' });

describe('useConfigEditor', () => {
  it('loads config data with versioned service entries', async () => {
    const { result } = renderHook(() =>
      useConfigEditor({ client, serviceName: 'test-service' }),
    );

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
    expect(result.current.config?.service).toBe('test-service');
    expect(result.current.config?.categories[0].fieldValues.timeout.version).toBe(3);
    expect(result.current.config?.categories[1].fieldValues.timeout.value).toBe(180);
    expect(result.current.isDirty).toBe(false);
  });

  it('updates fields by category without colliding on duplicate keys', async () => {
    const { result } = renderHook(() =>
      useConfigEditor({ client, serviceName: 'test-service' }),
    );

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    act(() => {
      result.current.updateField('advanced', 'timeout', 240);
    });

    expect(result.current.isDirty).toBe(true);
    expect(result.current.config?.dirtyCount).toBe(1);
    expect(result.current.config?.categories[0].fieldValues.timeout.value).toBe(60);
    expect(result.current.config?.categories[1].fieldValues.timeout.value).toBe(240);
  });

  it('reset restores the initial config snapshot', async () => {
    const { result } = renderHook(() =>
      useConfigEditor({ client, serviceName: 'test-service' }),
    );

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    act(() => {
      result.current.updateField('general', 'timeout', 120);
    });
    expect(result.current.isDirty).toBe(true);

    act(() => {
      result.current.reset();
    });

    expect(result.current.isDirty).toBe(false);
    expect(result.current.hasConflict).toBe(false);
    expect(result.current.config?.categories[0].fieldValues.timeout.value).toBe(60);
    expect(result.current.config?.dirtyCount).toBe(0);
  });

  it('save sends namespace-specific versioned updates and refreshes local versions', async () => {
    const { result } = renderHook(() =>
      useConfigEditor({ client, serviceName: 'test-service' }),
    );

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    act(() => {
      result.current.updateField('advanced', 'timeout', 240);
    });

    await act(async () => {
      await result.current.save();
    });

    expect(receivedUpdates).toEqual([
      {
        namespace: 'test.advanced',
        key: 'timeout',
        body: { value: 240, version: 8 },
      },
    ]);
    expect(result.current.isDirty).toBe(false);
    expect(result.current.hasConflict).toBe(false);
    expect(result.current.config?.categories[1].fieldValues.timeout.value).toBe(240);
    expect(result.current.config?.categories[1].fieldValues.timeout.version).toBe(9);
    expect(result.current.config?.categories[1].fieldValues.timeout.originalVersion).toBe(9);
  });

  it('marks hasConflict when save receives 409', async () => {
    server.use(
      http.put('http://localhost/api/v1/config/:namespace/:key', () => {
        return new HttpResponse(null, { status: 409 });
      }),
    );

    const { result } = renderHook(() =>
      useConfigEditor({ client, serviceName: 'test-service' }),
    );

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    act(() => {
      result.current.updateField('general', 'timeout', 120);
    });

    await act(async () => {
      await result.current.save();
    });

    expect(result.current.hasConflict).toBe(true);
  });
});
