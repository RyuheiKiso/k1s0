import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import MigratePage from '../MigratePage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_migrate_targets') {
      return Promise.resolve([
        {
          service_name: 'auth',
          tier: 'system',
          language: 'Rust',
          migrations_dir: 'regions/system/server/rust/auth/migrations',
          db_name: 'auth_db',
        },
      ]);
    }
    if (command === 'execute_migrate_status') {
      return Promise.resolve([
        {
          number: 1,
          description: 'init',
          applied: true,
          applied_at: '2026-03-10T00:00:00Z',
        },
      ]);
    }
    return Promise.resolve(undefined);
  });
});

describe('MigratePage', () => {
  it('loads migration status for the selected target', async () => {
    const user = userEvent.setup();
    renderWithProviders(<MigratePage />);

    await waitFor(() => expect(screen.getByTestId('select-migrate-target')).toBeInTheDocument());
    await user.click(screen.getByTestId('btn-migrate-status'));

    expect(await screen.findByText(/適用日時/)).toBeInTheDocument();
  });

  it('requires confirmation before rolling back the previous migration', async () => {
    const user = userEvent.setup();
    renderWithProviders(<MigratePage />);

    await waitFor(() => expect(screen.getByTestId('select-migrate-target')).toBeInTheDocument());
    await user.click(screen.getByTestId('btn-migrate-down'));

    expect(await screen.findByTestId('migrate-confirmation')).toBeInTheDocument();
    expect(screen.getByText(/1件のマイグレーションをロールバック/)).toBeInTheDocument();

    await user.click(screen.getByTestId('btn-confirm-migrate'));

    expect(mockInvoke).toHaveBeenCalledWith(
      'execute_migrate_down',
      expect.objectContaining({
        config: expect.objectContaining({
          range: { Steps: 1 },
        }),
      }),
    );
  });
});
