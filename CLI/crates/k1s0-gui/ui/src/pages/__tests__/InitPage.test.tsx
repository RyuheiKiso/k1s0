import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import InitPage from '../InitPage';

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('InitPage', () => {
  it('should render the form', () => {
    render(<InitPage />);
    expect(screen.getByTestId('init-page')).toBeInTheDocument();
    expect(screen.getByTestId('input-project-name')).toBeInTheDocument();
    expect(screen.getByTestId('checkbox-git-init')).toBeInTheDocument();
    expect(screen.getByTestId('checkbox-sparse')).toBeInTheDocument();
    expect(screen.getByTestId('btn-submit')).toBeInTheDocument();
  });

  it('should show validation error for empty project name', async () => {
    const user = userEvent.setup();
    render(<InitPage />);
    await user.click(screen.getByTestId('btn-submit'));
    expect(await screen.findByTestId('error-project-name')).toBeInTheDocument();
  });

  it('should show tier selection when sparse-checkout is enabled', async () => {
    const user = userEvent.setup();
    render(<InitPage />);
    await user.click(screen.getByTestId('checkbox-sparse'));
    expect(screen.getByTestId('tier-selection')).toBeInTheDocument();
  });

  it('should hide tier selection when sparse-checkout is disabled', () => {
    render(<InitPage />);
    expect(screen.queryByTestId('tier-selection')).not.toBeInTheDocument();
  });

  it('should call executeInit on valid submission', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(undefined);
    render(<InitPage />);

    await user.type(screen.getByTestId('input-project-name'), 'my-project');
    await user.click(screen.getByTestId('btn-submit'));

    // Wait for success message
    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('execute_init', expect.objectContaining({
      config: expect.objectContaining({ project_name: 'my-project' }),
    }));
  });

  it('should show error message on failure', async () => {
    const user = userEvent.setup();
    mockInvoke.mockRejectedValue('init failed');
    render(<InitPage />);

    await user.type(screen.getByTestId('input-project-name'), 'my-project');
    await user.click(screen.getByTestId('btn-submit'));

    expect(await screen.findByTestId('error-message')).toBeInTheDocument();
  });
});
