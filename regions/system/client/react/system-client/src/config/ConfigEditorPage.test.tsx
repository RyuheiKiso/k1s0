import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import axios from 'axios';
import { ConfigEditorPage } from './ConfigEditorPage';

const mockSchema = {
  service: 'test-service',
  namespace_prefix: 'test',
  categories: [
    {
      id: 'general',
      label: '一般設定',
      namespaces: ['test.general'],
      fields: [
        { key: 'timeout', label: 'タイムアウト', type: 'integer', min: 1, max: 300, default: 30, unit: '秒' },
        { key: 'enabled', label: '有効', type: 'boolean', default: false },
      ],
    },
    {
      id: 'advanced',
      label: '詳細設定',
      namespaces: ['test.advanced'],
      fields: [
        { key: 'log_level', label: 'ログレベル', type: 'enum', options: ['debug', 'info', 'warn', 'error'], default: 'info' },
      ],
    },
  ],
};

const mockValues = [
  { namespace: 'test.general', key: 'timeout', value: 60 },
  { namespace: 'test.general', key: 'enabled', value: true },
  { namespace: 'test.advanced', key: 'log_level', value: 'warn' },
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

describe('ConfigEditorPage', () => {
  it('カテゴリとフィールドをレンダリングする', async () => {
    render(<ConfigEditorPage serviceName="test-service" client={client} />);

    await waitFor(() => {
      expect(screen.getByText('test-service 設定')).toBeInTheDocument();
    });

    expect(screen.getByText('一般設定')).toBeInTheDocument();
    expect(screen.getByText('詳細設定')).toBeInTheDocument();
  });

  it('カテゴリ切り替えでフィールドが変わる', async () => {
    render(<ConfigEditorPage serviceName="test-service" client={client} />);

    await waitFor(() => {
      expect(screen.getByText('一般設定')).toBeInTheDocument();
    });

    // 最初は一般設定がアクティブ
    expect(screen.getByLabelText('タイムアウト')).toBeInTheDocument();

    // 詳細設定に切り替え
    fireEvent.click(screen.getByText('詳細設定'));

    await waitFor(() => {
      expect(screen.getByLabelText('ログレベル')).toBeInTheDocument();
    });
  });

  it('保存ボタンは初期状態で無効', async () => {
    render(<ConfigEditorPage serviceName="test-service" client={client} />);

    await waitFor(() => {
      expect(screen.getByText('保存')).toBeInTheDocument();
    });

    expect(screen.getByText('保存')).toBeDisabled();
    expect(screen.getByText('破棄')).toBeDisabled();
  });
});
