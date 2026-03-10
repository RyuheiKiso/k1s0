import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { ConfigEditorPage } from './ConfigEditorPage';
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
        { key: 'timeout', label: 'Timeout', type: 'integer', min: 1, max: 300, default: 30, unit: 'sec' },
        { key: 'enabled', label: 'Enabled', type: 'boolean', default: false },
      ],
    },
    {
      id: 'advanced',
      label: 'Advanced',
      namespaces: ['test.advanced'],
      fields: [
        { key: 'log_level', label: 'Log Level', type: 'enum', options: ['debug', 'info', 'warn', 'error'], default: 'info' },
      ],
    },
  ],
};

const mockValues: ServiceConfigResultResponse = {
  service_name: 'test-service',
  entries: [
    { namespace: 'test.general', key: 'timeout', value: 60, version: 2 },
    { namespace: 'test.general', key: 'enabled', value: true, version: 4 },
    { namespace: 'test.advanced', key: 'log_level', value: 'warn', version: 6 },
  ],
};

const server = setupServer(
  http.get('http://localhost/api/v1/config-schema/:service', () => {
    return HttpResponse.json(mockSchema);
  }),
  http.get('http://localhost/api/v1/config/services/:service', () => {
    return HttpResponse.json(mockValues);
  }),
  http.put('http://localhost/api/v1/config/:namespace/:key', async ({ params, request }) => {
    const body = await request.json() as { value: unknown; version: number };
    return HttpResponse.json({
      namespace: decodeURIComponent(String(params.namespace)),
      key: decodeURIComponent(String(params.key)),
      value: body.value,
      version: body.version + 1,
    });
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('ConfigEditorPage', () => {
  it('renders categories and fields using only serviceName plus apiBaseURL', async () => {
    render(<ConfigEditorPage serviceName="test-service" apiBaseURL="http://localhost" />);

    await waitFor(() => {
      expect(screen.getByText('test-service Config')).toBeInTheDocument();
    });

    expect(screen.getByText('General')).toBeInTheDocument();
    expect(screen.getByText('Advanced')).toBeInTheDocument();
    expect(screen.getByLabelText('Timeout')).toBeInTheDocument();
  });

  it('switches category content when another category is selected', async () => {
    render(<ConfigEditorPage serviceName="test-service" apiBaseURL="http://localhost" />);

    await waitFor(() => {
      expect(screen.getByText('General')).toBeInTheDocument();
    });

    expect(screen.getByLabelText('Timeout')).toBeInTheDocument();

    fireEvent.click(screen.getByText('Advanced'));

    await waitFor(() => {
      expect(screen.getByLabelText('Log Level')).toBeInTheDocument();
    });
  });

  it('keeps Save and Discard disabled before any edits', async () => {
    render(<ConfigEditorPage serviceName="test-service" apiBaseURL="http://localhost" />);

    await waitFor(() => {
      expect(screen.getByText('Save')).toBeInTheDocument();
      expect(screen.getByText('Discard')).toBeInTheDocument();
    });

    expect(screen.getByText('Save')).toBeDisabled();
    expect(screen.getByText('Discard')).toBeDisabled();
  });
});
