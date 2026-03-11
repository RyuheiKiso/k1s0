import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import DevPage from '../DevPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_dev_targets') {
      return Promise.resolve([['auth', 'regions/system/server/rust/auth']]);
    }
    if (command === 'preview_dev_up') {
      return Promise.resolve({
        dependencies: {
          databases: [{ name: 'auth_db', service: 'auth' }],
          has_kafka: false,
          has_redis: true,
          has_redis_session: false,
          kafka_topics: [],
        },
        ports: {
          postgres: 5432,
          kafka: 9092,
          redis: 6379,
          redis_session: 6380,
          pgadmin: 5050,
          kafka_ui: 8081,
          keycloak: 8180,
        },
        additional_services: [{ name: 'pgAdmin', url: 'http://localhost:5050' }],
      });
    }
    if (command === 'execute_dev_status') {
      return Promise.resolve('Local development environment is not running.');
    }
    return Promise.resolve(undefined);
  });
});

describe('DevPage', () => {
  it('loads local development status', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevPage />);

    await waitFor(() => screen.getByText('auth'));
    await user.click(screen.getByTestId('btn-dev-status'));

    expect(await screen.findByText('Local development environment is not running.')).toBeInTheDocument();
  });

  it('requires preview confirmation before starting local development', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DevPage />);

    await waitFor(() => screen.getByText('auth'));
    await user.click(screen.getByRole('checkbox'));
    await user.click(screen.getByTestId('btn-dev-up'));

    const preview = await screen.findByTestId('dev-up-preview');
    expect(preview).toBeInTheDocument();
    expect(within(preview).getByText(/auth_db/)).toBeInTheDocument();
    expect(within(preview).getAllByText(/http:\/\/localhost:5050/)).toHaveLength(2);

    await user.click(screen.getByTestId('btn-confirm-dev-up'));

    expect(mockInvoke).toHaveBeenCalledWith(
      'execute_dev_up',
      expect.objectContaining({
        config: {
          services: ['regions/system/server/rust/auth'],
          auth_mode: 'Skip',
        },
      }),
    );
  });
});
