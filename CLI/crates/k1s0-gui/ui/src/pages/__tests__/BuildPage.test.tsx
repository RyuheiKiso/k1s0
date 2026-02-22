import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import BuildPage from '../BuildPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue([]);
});

describe('BuildPage', () => {
  it('should render the build page', async () => {
    render(<BuildPage />);
    await waitFor(() => {
      expect(screen.getByTestId('build-page')).toBeInTheDocument();
    });
  });

  it('should show empty state when no targets are found', async () => {
    mockInvoke.mockResolvedValue([]);
    render(<BuildPage />);
    await waitFor(() => {
      expect(screen.getByText('ビルド対象が見つかりません。')).toBeInTheDocument();
    });
  });

  it('should render targets returned by scanBuildableTargets', async () => {
    mockInvoke.mockResolvedValue(['regions/system/server/rust/auth', 'regions/system/server/rust/saga']);
    render(<BuildPage />);
    await waitFor(() => {
      expect(screen.getByText('regions/system/server/rust/auth')).toBeInTheDocument();
      expect(screen.getByText('regions/system/server/rust/saga')).toBeInTheDocument();
    });
  });

  it('should disable the build button when no targets are selected', async () => {
    mockInvoke.mockResolvedValue(['regions/system/server/rust/auth']);
    render(<BuildPage />);
    await waitFor(() => {
      expect(screen.getByTestId('btn-build')).toBeDisabled();
    });
  });

  it('should enable the build button after selecting a target', async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(['regions/system/server/rust/auth']);
    render(<BuildPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox'));
    expect(screen.getByTestId('btn-build')).not.toBeDisabled();
  });

  it('should show success message after successful build', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(['regions/system/server/rust/auth'])
      .mockResolvedValue(undefined);
    render(<BuildPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox'));
    await user.click(screen.getByTestId('btn-build'));

    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
  });

  it('should show error message on build failure', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(['regions/system/server/rust/auth'])
      .mockRejectedValue('build failed');
    render(<BuildPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox'));
    await user.click(screen.getByTestId('btn-build'));

    expect(await screen.findByTestId('error-message')).toBeInTheDocument();
  });
});
