import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import BuildPage from '../BuildPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_buildable_targets') {
      return Promise.resolve([]);
    }
    return Promise.resolve(undefined);
  });
});

describe('BuildPage', () => {
  it('renders the build page', async () => {
    renderWithProviders(<BuildPage />);
    await waitFor(() => {
      expect(screen.getByTestId('build-page')).toBeInTheDocument();
    });
  });

  it('shows an empty state when no targets are found', async () => {
    renderWithProviders(<BuildPage />);
    expect(await screen.findByText('ビルド可能なターゲットが見つかりませんでした。')).toBeInTheDocument();
  });

  it('renders targets returned by scanBuildableTargets', async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'scan_buildable_targets') {
        return Promise.resolve([
          'regions/system/server/rust/auth',
          'regions/system/server/rust/saga',
        ]);
      }
      return Promise.resolve(undefined);
    });

    renderWithProviders(<BuildPage />);

    expect(await screen.findByText('regions/system/server/rust/auth')).toBeInTheDocument();
    expect(screen.getByText('regions/system/server/rust/saga')).toBeInTheDocument();
  });

  it('disables the build button when no targets are selected', async () => {
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'scan_buildable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }
      return Promise.resolve(undefined);
    });

    renderWithProviders(<BuildPage />);

    await waitFor(() => {
      expect(screen.getByTestId('btn-build')).toBeDisabled();
    });
  });

  it('shows success only after a successful Finished event', async () => {
    const user = userEvent.setup();

    mockInvoke.mockImplementation((command: string, args?: { onEvent?: { onmessage?: (event: unknown) => void } }) => {
      if (command === 'scan_buildable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }

      if (command === 'execute_build_with_progress') {
        args?.onEvent?.onmessage?.({
          kind: 'StepStarted',
          step: 1,
          total: 1,
          message: 'Building auth',
        });
        args?.onEvent?.onmessage?.({
          kind: 'Finished',
          success: true,
          message: 'Build completed',
        });
      }

      return Promise.resolve(undefined);
    });

    renderWithProviders(<BuildPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox', { name: /regions\/system\/server\/rust\/auth/i }));
    await user.click(screen.getByTestId('btn-build'));

    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
  });

  it('shows an error when the terminal event reports failure', async () => {
    const user = userEvent.setup();

    mockInvoke.mockImplementation((command: string, args?: { onEvent?: { onmessage?: (event: unknown) => void } }) => {
      if (command === 'scan_buildable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }

      if (command === 'execute_build_with_progress') {
        args?.onEvent?.onmessage?.({
          kind: 'Finished',
          success: false,
          message: 'Build failed',
        });
      }

      return Promise.resolve(undefined);
    });

    renderWithProviders(<BuildPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox', { name: /regions\/system\/server\/rust\/auth/i }));
    await user.click(screen.getByTestId('btn-build'));

    expect(await screen.findByTestId('error-message')).toHaveTextContent('Build failed');
  });
});
