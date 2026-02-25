import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import TestPage from '../TestPage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue([]);
});

describe('TestPage', () => {
  it('should render the test page', async () => {
    render(<TestPage />);
    await waitFor(() => {
      expect(screen.getByTestId('test-page')).toBeInTheDocument();
    });
  });

  it('should render all test kind radio buttons', () => {
    render(<TestPage />);
    expect(screen.getByText('Unit')).toBeInTheDocument();
    expect(screen.getByText('Integration')).toBeInTheDocument();
    expect(screen.getByText('All')).toBeInTheDocument();
  });

  it('should hide target list when All is selected', async () => {
    const user = userEvent.setup();
    render(<TestPage />);
    // Radix RadioGroup.Item renders as role="radio" button; labels are not connected via htmlFor
    const allRadio = screen.getAllByRole('radio')[2]; // Unit/Integration/All
    await user.click(allRadio);
    expect(screen.queryByText('テスト対象')).not.toBeInTheDocument();
  });

  it('should show target list for Unit kind', async () => {
    mockInvoke.mockResolvedValue(['regions/system/server/rust/auth']);
    render(<TestPage />);
    await waitFor(() => {
      expect(screen.getByText('regions/system/server/rust/auth')).toBeInTheDocument();
    });
  });

  it('should reload targets when switching from Unit to Integration', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(['regions/system/server/rust/auth'])
      .mockResolvedValueOnce(['regions/system/library/go/auth']);
    render(<TestPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    const integrationRadio = screen.getAllByRole('radio')[1]; // Unit/Integration/All
    await user.click(integrationRadio);
    await waitFor(() => {
      expect(screen.getByText('regions/system/library/go/auth')).toBeInTheDocument();
    });
  });

  it('should disable the run button when no targets are selected (Unit)', async () => {
    mockInvoke.mockResolvedValue(['regions/system/server/rust/auth']);
    render(<TestPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));
    expect(screen.getByTestId('btn-test')).toBeDisabled();
  });

  it('should enable the run button for All kind without selecting targets', async () => {
    const user = userEvent.setup();
    render(<TestPage />);
    const allRadio = screen.getAllByRole('radio')[2]; // Unit/Integration/All
    await user.click(allRadio);
    expect(screen.getByTestId('btn-test')).not.toBeDisabled();
  });

  it('should show success message after successful test run', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(['regions/system/server/rust/auth'])
      .mockResolvedValue(undefined);
    render(<TestPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox'));
    await user.click(screen.getByTestId('btn-test'));

    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
  });

  it('should show error message on test failure', async () => {
    const user = userEvent.setup();
    mockInvoke
      .mockResolvedValueOnce(['regions/system/server/rust/auth'])
      .mockRejectedValue('test failed');
    render(<TestPage />);
    await waitFor(() => screen.getByText('regions/system/server/rust/auth'));

    await user.click(screen.getByRole('checkbox'));
    await user.click(screen.getByTestId('btn-test'));

    expect(await screen.findByTestId('error-message')).toBeInTheDocument();
  });
});
