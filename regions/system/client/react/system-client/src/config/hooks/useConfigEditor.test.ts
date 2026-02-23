import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import axios from 'axios';
import { useConfigEditor } from './useConfigEditor';

const mockSchema = {
  service: 'test-service',
  namespace_prefix: 'test',
  categories: [
    {
      id: 'general',
      label: '一般設定',
      namespaces: ['test.general'],
      fields: [
        { key: 'timeout', label: 'タイムアウト', type: 'integer', default: 30 },
        { key: 'name', label: '名前', type: 'string', default: 'default' },
      ],
    },
  ],
};

const mockValues = [
  { namespace: 'test.general', key: 'timeout', value: 60 },
  { namespace: 'test.general', key: 'name', value: 'current' },
];

const server = setupServer(
  http.get('http://localhost/api/v1/config-schema/:service', () => {
    return HttpResponse.json(mockSchema);
  }),
  http.get('http://localhost/api/v1/config/services/:service', () => {
    return HttpResponse.json(mockValues);
  }),
  http.put('http://localhost/api/v1/config/:namespace/:key', () => {
    return HttpResponse.json({ ok: true });
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const client = axios.create({ baseURL: 'http://localhost' });

describe('useConfigEditor', () => {
  it('初期ロード後に config を返す', async () => {
    const { result } = renderHook(() =>
      useConfigEditor({ client, serviceName: 'test-service' }),
    );

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    expect(result.current.config?.service).toBe('test-service');
    expect(result.current.isDirty).toBe(false);
  });

  it('updateField でフィールドを更新し isDirty が true になる', async () => {
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
    expect(result.current.config?.dirtyCount).toBe(1);
    expect(result.current.config?.categories[0].fieldValues['timeout'].value).toBe(120);
  });

  it('reset で全変更を元に戻す', async () => {
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
    expect(result.current.config?.categories[0].fieldValues['timeout'].value).toBe(60);
  });

  it('save で変更を保存し isDirty が false になる', async () => {
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

    expect(result.current.isDirty).toBe(false);
    expect(result.current.hasConflict).toBe(false);
  });

  it('save で 409 の場合 hasConflict が true になる', async () => {
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
