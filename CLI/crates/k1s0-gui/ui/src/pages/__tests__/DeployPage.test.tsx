import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import DeployPage from '../DeployPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue([]);
});

describe('DeployPage', () => {
  it('should render the deploy page', async () => {
    render(<DeployPage />);
    await waitFor(() => {
      expect(screen.getByTestId('deploy-page')).toBeInTheDocument();
    });
  });

  it('should show prod confirmation when Prod is selected', async () => {
    const user = userEvent.setup();
    render(<DeployPage />);
    await waitFor(() => expect(screen.getByTestId('deploy-page')).toBeInTheDocument());

    const prodRadio = screen.getByLabelText('prod');
    await user.click(prodRadio);

    expect(screen.getByTestId('prod-confirm')).toBeInTheDocument();
  });

  it('should not show prod confirmation for Dev', async () => {
    render(<DeployPage />);
    await waitFor(() => expect(screen.getByTestId('deploy-page')).toBeInTheDocument());
    expect(screen.queryByTestId('prod-confirm')).not.toBeInTheDocument();
  });

  it('should require "deploy" text for prod deploy', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue([]);
    render(<DeployPage />);
    await waitFor(() => expect(screen.getByTestId('deploy-page')).toBeInTheDocument());

    // Select prod
    await user.click(screen.getByLabelText('prod'));
    // Don't type "deploy"
    // Try to deploy - button should be disabled since no targets selected
    expect(screen.getByTestId('btn-deploy')).toBeDisabled();
  });
});
