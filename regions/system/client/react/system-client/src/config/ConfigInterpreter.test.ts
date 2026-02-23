import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import axios from 'axios';
import { ConfigInterpreter } from './ConfigInterpreter';
import type { ConfigEditorSchema } from './types';

const mockSchema: ConfigEditorSchema = {
  service: 'test-service',
  namespace_prefix: 'test',
  categories: [
    {
      id: 'general',
      label: '一般設定',
      namespaces: ['test.general'],
      fields: [
        { key: 'timeout', label: 'タイムアウト', type: 'integer', default: 30 },
        { key: 'enabled', label: '有効', type: 'boolean', default: false },
      ],
    },
  ],
};

const mockValues = [
  { namespace: 'test.general', key: 'timeout', value: 60 },
];

const server = setupServer(
  http.get('http://localhost/api/v1/config-schema/:service', () => {
    return HttpResponse.json(mockSchema);
  }),
  http.get('http://localhost/api/v1/config/services/:service', () => {
    return HttpResponse.json(mockValues);
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('ConfigInterpreter', () => {
  it('スキーマと値をマージして ConfigEditorConfig を返す', async () => {
    const client = axios.create({ baseURL: 'http://localhost' });
    const interpreter = new ConfigInterpreter(client);
    const config = await interpreter.build('test-service');

    expect(config.service).toBe('test-service');
    expect(config.categories).toHaveLength(1);
    expect(config.dirtyCount).toBe(0);

    const general = config.categories[0];
    expect(general.id).toBe('general');

    // timeout は API値(60)で上書きされる
    expect(general.fieldValues['timeout'].value).toBe(60);
    expect(general.fieldValues['timeout'].originalValue).toBe(60);
    expect(general.fieldValues['timeout'].isDirty).toBe(false);

    // enabled は API値がないのでデフォルト(false)を使用
    expect(general.fieldValues['enabled'].value).toBe(false);
    expect(general.fieldValues['enabled'].originalValue).toBe(false);
  });
});
