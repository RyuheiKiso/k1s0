import { describe, it, expect, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import InitPage from '../InitPage';

beforeEach(() => {
  mockInvoke.mockReset();
});

describe('InitPage', () => {
  it('renders the explicit destination controls', () => {
    renderWithProviders(<InitPage />);
    expect(screen.getByTestId('init-page')).toBeInTheDocument();
    expect(screen.getByTestId('input-base-dir')).toBeInTheDocument();
    expect(screen.getByTestId('input-project-name')).toBeInTheDocument();
    expect(screen.getByTestId('destination-preview')).toBeInTheDocument();
    expect(screen.getByTestId('checkbox-git-init')).toBeInTheDocument();
    expect(screen.getByTestId('checkbox-sparse')).toBeInTheDocument();
    expect(screen.getByTestId('btn-submit')).toBeInTheDocument();
  });

  it('shows validation error for empty project name', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue('C:/work');
    renderWithProviders(<InitPage />);

    await user.clear(screen.getByTestId('input-base-dir'));
    await user.type(screen.getByTestId('input-base-dir'), 'C:/work');
    await user.click(screen.getByTestId('btn-submit'));

    expect(await screen.findByTestId('error-project-name')).toBeInTheDocument();
  });

  it('shows validation error for empty base directory', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InitPage />);

    await user.type(screen.getByTestId('input-project-name'), 'my-project');
    await user.clear(screen.getByTestId('input-base-dir'));
    await user.click(screen.getByTestId('btn-submit'));

    expect(await screen.findByTestId('error-base-dir')).toBeInTheDocument();
  });

  it('shows tier selection when sparse-checkout is enabled', async () => {
    const user = userEvent.setup();
    renderWithProviders(<InitPage />);
    await user.click(screen.getByTestId('checkbox-sparse'));
    expect(screen.getByTestId('tier-selection')).toBeInTheDocument();
  });

  it('calls execute_init_at and adopts the created workspace on valid submission', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValueOnce('C:/work').mockResolvedValueOnce('C:/work/my-project');
    const adoptWorkspace = async () => true;

    renderWithProviders(<InitPage />, {
      workspace: {
        draftPath: '',
        adoptWorkspace,
      },
    });

    await user.clear(screen.getByTestId('input-base-dir'));
    await user.type(screen.getByTestId('input-base-dir'), 'C:/work');
    await user.type(screen.getByTestId('input-project-name'), 'my-project');
    await user.click(screen.getByTestId('btn-submit'));

    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('execute_init_at', expect.objectContaining({
      config: expect.objectContaining({ project_name: 'my-project' }),
      baseDir: 'C:/work',
    }));
  });

  it('shows an error when workspace adoption fails after initialization', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValueOnce('C:/work').mockResolvedValueOnce('C:/work/my-project');

    renderWithProviders(<InitPage />, {
      workspace: {
        draftPath: '',
        adoptWorkspace: async () => false,
      },
    });

    await user.clear(screen.getByTestId('input-base-dir'));
    await user.type(screen.getByTestId('input-base-dir'), 'C:/work');
    await user.type(screen.getByTestId('input-project-name'), 'my-project');
    await user.click(screen.getByTestId('btn-submit'));

    expect(await screen.findByTestId('error-message')).toBeInTheDocument();
  });
});
