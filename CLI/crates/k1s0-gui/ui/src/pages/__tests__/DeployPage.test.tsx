import { beforeEach, describe, expect, it } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import DeployPage from '../DeployPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_deployable_targets') {
      return Promise.resolve([]);
    }
    return Promise.resolve(undefined);
  });
});

describe('DeployPage', () => {
  it('renders the deploy page', async () => {
    render(<DeployPage />);
    await waitFor(() => {
      expect(screen.getByTestId('deploy-page')).toBeInTheDocument();
    });
  });

  it('shows prod confirmation when Prod is selected', async () => {
    const user = userEvent.setup();
    render(<DeployPage />);

    await user.click(screen.getByRole('radio', { name: 'prod' }));

    expect(screen.getByTestId('prod-confirm')).toBeInTheDocument();
  });

  it('requires at least one target before deploy', async () => {
    render(<DeployPage />);
    await waitFor(() => expect(screen.getByTestId('btn-deploy')).toBeDisabled());
  });

  it('shows an error when prod confirmation is missing', async () => {
    const user = userEvent.setup();

    mockInvoke.mockImplementation((command: string) => {
      if (command === 'scan_deployable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }
      return Promise.resolve(undefined);
    });

    render(<DeployPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox', { name: /regions\/system\/server\/rust\/auth/i }));
    await user.click(screen.getByRole('radio', { name: 'prod' }));
    await user.click(screen.getByTestId('btn-deploy'));

    expect(await screen.findByTestId('error-message')).toHaveTextContent(
      'Type "deploy" to confirm a production deployment.',
    );
  });

  it('shows rollback after a failed production deploy', async () => {
    const user = userEvent.setup();

    mockInvoke.mockImplementation((command: string, args?: { onEvent?: { onmessage?: (event: unknown) => void } }) => {
      if (command === 'scan_deployable_targets') {
        return Promise.resolve(['regions/system/server/rust/auth']);
      }

      if (command === 'execute_deploy_with_progress') {
        args?.onEvent?.onmessage?.({
          kind: 'Finished',
          success: false,
          message: 'Deploy failed',
        });
      }

      if (command === 'execute_deploy_rollback') {
        return Promise.resolve('Rollback completed');
      }

      return Promise.resolve(undefined);
    });

    render(<DeployPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox', { name: /regions\/system\/server\/rust\/auth/i }));
    await user.click(screen.getByRole('radio', { name: 'prod' }));
    await user.type(screen.getByTestId('input-prod-confirm'), 'deploy');
    await user.click(screen.getByTestId('btn-deploy'));

    const rollbackButton = await screen.findByTestId('btn-rollback');
    expect(rollbackButton).toBeInTheDocument();

    await user.click(rollbackButton);

    expect(await screen.findByText('Rollback completed')).toBeInTheDocument();
  });
});
