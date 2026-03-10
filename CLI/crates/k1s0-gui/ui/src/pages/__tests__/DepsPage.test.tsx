import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import DepsPage from '../DepsPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_services') {
      return Promise.resolve([
        {
          name: 'auth',
          tier: 'system',
          domain: null,
          language: 'rust',
          path: 'regions/system/server/rust/auth',
        },
      ]);
    }
    if (command === 'execute_deps') {
      return Promise.resolve({
        services: [],
        dependencies: [],
        violations: [],
      });
    }
    return Promise.resolve(undefined);
  });
});

describe('DepsPage', () => {
  it('runs the dependency scan for selected services', async () => {
    const user = userEvent.setup();
    renderWithProviders(<DepsPage />);

    await user.click(screen.getByRole('radio', { name: 'Selected services' }));
    await waitFor(() => screen.getByText('auth'));
    await user.click(screen.getAllByRole('checkbox')[0]);
    await user.click(screen.getByTestId('btn-run-deps'));

    expect(mockInvoke).toHaveBeenCalledWith('execute_deps', {
      config: {
        scope: { Services: ['auth'] },
        output: 'Terminal',
        no_cache: false,
      },
      baseDir: '.',
    });
  });
});
