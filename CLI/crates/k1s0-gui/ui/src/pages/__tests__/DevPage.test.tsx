import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
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
});
