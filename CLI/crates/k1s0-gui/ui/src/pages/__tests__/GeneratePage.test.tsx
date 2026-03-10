import { beforeEach, describe, expect, it } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import { renderWithProviders } from '../../test/render';
import GeneratePage from '../GeneratePage';

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((command: string) => {
    if (command === 'scan_placements') {
      return Promise.resolve([]);
    }
    if (command === 'validate_name') {
      return Promise.resolve(undefined);
    }
    return Promise.resolve(undefined);
  });
});

async function renderGeneratePage() {
  renderWithProviders(<GeneratePage />);
  await waitFor(() => {
    expect(mockInvoke).toHaveBeenCalledWith(
      'scan_databases',
      expect.objectContaining({ tier: 'System', baseDir: '.' }),
    );
  });
}

describe('GeneratePage', () => {
  it('renders the stepper', async () => {
    await renderGeneratePage();
    expect(screen.getByTestId('generate-page')).toBeInTheDocument();
    expect(screen.getByTestId('stepper')).toBeInTheDocument();
  });

  it('starts at the kind step', async () => {
    await renderGeneratePage();
    expect(screen.getByTestId('step-kind')).toBeInTheDocument();
  });

  it('navigates to the tier step on next', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByTestId('btn-next'));

    expect(screen.getByTestId('step-tier')).toBeInTheDocument();
  });

  it('navigates through server system flow and skips placement', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-tier')).toBeInTheDocument();

    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-langfw')).toBeInTheDocument();

    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-detail')).toBeInTheDocument();

    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-confirm')).toBeInTheDocument();
  });

  it('calls executeGenerateAt on confirm', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-generate'));

    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('execute_generate_at', expect.anything());
  });

  it('shows the placement step for Business tier', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByText('Client'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByText('business'));
    await user.click(screen.getByTestId('btn-next'));

    expect(screen.getByTestId('step-placement')).toBeInTheDocument();
  });

  it('skips detail for Database kind', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByText('Database'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));

    expect(screen.getByTestId('step-confirm')).toBeInTheDocument();
  });

  it('skips detail for Client plus Service tier', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByText('Client'));
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByText('service'));
    await user.click(screen.getByTestId('btn-next'));
    await user.type(screen.getByPlaceholderText('placement-name'), 'order');
    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));

    expect(screen.getByTestId('step-confirm')).toBeInTheDocument();
  });

  it('shows only Go and Rust for Server kind', async () => {
    const user = userEvent.setup();
    await renderGeneratePage();

    await user.click(screen.getByTestId('btn-next'));
    await user.click(screen.getByTestId('btn-next'));

    expect(screen.getByTestId('step-langfw')).toBeInTheDocument();
    expect(screen.getByText('Go')).toBeInTheDocument();
    expect(screen.getByText('Rust')).toBeInTheDocument();
    expect(screen.queryByText('TypeScript')).not.toBeInTheDocument();
    expect(screen.queryByText('Dart')).not.toBeInTheDocument();
  });
});
