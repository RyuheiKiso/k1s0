import { beforeEach, describe, expect, it } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import TestPage from '../TestPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_testable_targets') {
      return Promise.resolve([]);
    }
    return Promise.resolve(undefined);
  });
});

describe('TestPage', () => {
  it('renders the test page', async () => {
    render(<TestPage />);
    await waitFor(() => {
      expect(screen.getByTestId('test-page')).toBeInTheDocument();
    });
  });

  it('renders all test kind radio buttons', async () => {
    render(<TestPage />);
    await waitFor(() => {
      expect(screen.getByTestId('test-page')).toBeInTheDocument();
    });
    expect(screen.getByText('Unit')).toBeInTheDocument();
    expect(screen.getByText('Integration')).toBeInTheDocument();
    expect(screen.getByText('All')).toBeInTheDocument();
  });

  it('hides the target list when All is selected', async () => {
    const user = userEvent.setup();
    render(<TestPage />);

    await user.click(screen.getByRole('radio', { name: 'All' }));

    expect(screen.queryByText('Targets')).not.toBeInTheDocument();
  });

  it('shows target list for Unit kind', async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'scan_testable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }
      return Promise.resolve(undefined);
    });

    render(<TestPage />);
    expect(await screen.findByText('regions/system/server/rust/auth')).toBeInTheDocument();
  });

  it('enables the run button for All without selecting targets', async () => {
    const user = userEvent.setup();
    render(<TestPage />);

    await user.click(screen.getByRole('radio', { name: 'All' }));

    expect(screen.getByTestId('btn-test')).not.toBeDisabled();
  });

  it('shows success after a successful terminal event', async () => {
    const user = userEvent.setup();

    mockInvoke.mockImplementation((command: string, args?: { onEvent?: { onmessage?: (event: unknown) => void } }) => {
      if (command === 'scan_testable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }

      if (command === 'execute_test_with_progress_at') {
        args?.onEvent?.onmessage?.({
          kind: 'Finished',
          success: true,
          message: 'Tests completed',
        });
      }

      return Promise.resolve(undefined);
    });

    render(<TestPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox', { name: /regions\/system\/server\/rust\/auth/i }));
    await user.click(screen.getByTestId('btn-test'));

    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
  });

  it('shows an error when the terminal event fails', async () => {
    const user = userEvent.setup();

    mockInvoke.mockImplementation((command: string, args?: { onEvent?: { onmessage?: (event: unknown) => void } }) => {
      if (command === 'scan_testable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }

      if (command === 'execute_test_with_progress_at') {
        args?.onEvent?.onmessage?.({
          kind: 'Finished',
          success: false,
          message: 'Tests failed',
        });
      }

      return Promise.resolve(undefined);
    });

    render(<TestPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox', { name: /regions\/system\/server\/rust\/auth/i }));
    await user.click(screen.getByTestId('btn-test'));

    expect(await screen.findByTestId('error-message')).toHaveTextContent('Tests failed');
  });
});
