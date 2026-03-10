import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import axios from 'axios';
import { ConfigInterpreter } from './ConfigInterpreter';
import type { ConfigEditorSchema, ServiceConfigResultResponse } from './types';

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
        { key: 'enabled', label: 'Enabled', type: 'boolean', default: false },
      ],
    },
    {
      id: 'advanced',
      label: 'Advanced',
      namespaces: ['test.advanced'],
      fields: [
        { key: 'timeout', label: 'Advanced Timeout', type: 'integer', default: 10 },
      ],
    },
  ],
};

const mockValues: ServiceConfigResultResponse = {
  service_name: 'test-service',
  entries: [
    { namespace: 'test.general', key: 'timeout', value: 60, version: 3 },
    { namespace: 'test.general', key: 'enabled', value: true, version: 7 },
    { namespace: 'test.advanced', key: 'timeout', value: 120, version: 5 },
  ],
};

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
  it('merges schema and service config entries using namespace plus key identities', async () => {
    const client = axios.create({ baseURL: 'http://localhost' });
    const interpreter = new ConfigInterpreter(client);
    const config = await interpreter.build('test-service');

    expect(config.service).toBe('test-service');
    expect(config.categories).toHaveLength(2);
    expect(config.dirtyCount).toBe(0);

    const general = config.categories[0];
    expect(general.fieldValues.timeout.id).toBe('test.general::timeout');
    expect(general.fieldValues.timeout.value).toBe(60);
    expect(general.fieldValues.timeout.version).toBe(3);
    expect(general.fieldValues.timeout.originalVersion).toBe(3);
    expect(general.fieldValues.enabled.value).toBe(true);
    expect(general.fieldValues.enabled.version).toBe(7);

    const advanced = config.categories[1];
    expect(advanced.fieldValues.timeout.id).toBe('test.advanced::timeout');
    expect(advanced.fieldValues.timeout.value).toBe(120);
    expect(advanced.fieldValues.timeout.version).toBe(5);
    expect(advanced.fieldValues.timeout.originalValue).toBe(120);
    expect(advanced.fieldValues.timeout.isDirty).toBe(false);
  });
});
