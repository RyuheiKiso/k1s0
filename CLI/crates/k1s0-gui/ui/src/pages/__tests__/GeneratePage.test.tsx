import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { mockInvoke } from '../../test/mocks';
import GeneratePage from '../GeneratePage';

beforeEach(() => {
  mockInvoke.mockReset();
  // Default mock for scanPlacements returns empty
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'scan_placements') return Promise.resolve([]);
    return Promise.resolve(undefined);
  });
});

describe('GeneratePage', () => {
  it('should render the stepper', () => {
    render(<GeneratePage />);
    expect(screen.getByTestId('generate-page')).toBeInTheDocument();
    expect(screen.getByTestId('stepper')).toBeInTheDocument();
  });

  it('should start at step 0 (Kind)', () => {
    render(<GeneratePage />);
    expect(screen.getByTestId('step-kind')).toBeInTheDocument();
  });

  it('should navigate to step 1 on next', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);
    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-tier')).toBeInTheDocument();
  });

  it('should navigate back from step 1', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);
    await user.click(screen.getByTestId('btn-next')); // go to step 1
    await user.click(screen.getByTestId('btn-back')); // go back to step 0
    expect(screen.getByTestId('step-kind')).toBeInTheDocument();
  });

  it('should navigate through all steps to confirm (Server + System: skip placement)', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);
    // Step 0 (Kind) → 1 (Tier)
    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-tier')).toBeInTheDocument();
    // Step 1 (Tier, System) → 3 (LangFW) — skips step 2 placement
    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-langfw')).toBeInTheDocument();
    // Step 3 (LangFW) → 4 (Detail)
    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-detail')).toBeInTheDocument();
    // Step 4 (Detail) → 5 (Confirm)
    await user.click(screen.getByTestId('btn-next'));
    expect(screen.getByTestId('step-confirm')).toBeInTheDocument();
  });

  it('should call executeGenerate on confirm', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);

    // Navigate to confirm (Server + System: skip placement)
    // Step 0 → 1
    await user.click(screen.getByTestId('btn-next'));
    // Step 1 → 3 (skip placement for System)
    await user.click(screen.getByTestId('btn-next'));
    // Step 3 → 4
    await user.click(screen.getByTestId('btn-next'));
    // Step 4 → 5
    await user.click(screen.getByTestId('btn-next'));

    await user.click(screen.getByTestId('btn-generate'));
    expect(await screen.findByTestId('success-message')).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith('execute_generate', expect.anything());
  });

  it('should show placement step for Business tier', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);

    // Step 0: select Client (which supports Business)
    await user.click(screen.getByText('Client'));
    await user.click(screen.getByTestId('btn-next'));

    // Step 1: select Business tier
    await user.click(screen.getByText('business'));
    await user.click(screen.getByTestId('btn-next'));

    // Step 2: placement should be visible
    expect(screen.getByTestId('step-placement')).toBeInTheDocument();
  });

  it('should skip detail step for Database kind', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);

    // Step 0: select Database
    await user.click(screen.getByText('Database'));
    await user.click(screen.getByTestId('btn-next'));

    // Step 1: Tier (System is default)
    await user.click(screen.getByTestId('btn-next'));

    // Step 3: LangFW (skipped placement for System)
    expect(screen.getByTestId('step-langfw')).toBeInTheDocument();
    await user.click(screen.getByTestId('btn-next'));

    // Should go to step 5 (confirm), skipping step 4 (detail)
    expect(screen.getByTestId('step-confirm')).toBeInTheDocument();
  });

  it('should show only Go and Rust for Server kind language options', async () => {
    const user = userEvent.setup();
    render(<GeneratePage />);

    // Step 0 → 1 (Server is default)
    await user.click(screen.getByTestId('btn-next'));
    // Step 1 → 3 (System tier, skip placement)
    await user.click(screen.getByTestId('btn-next'));

    // Step 3: should show Go, Rust but NOT TypeScript, Dart
    expect(screen.getByTestId('step-langfw')).toBeInTheDocument();
    expect(screen.getByText('Go')).toBeInTheDocument();
    expect(screen.getByText('Rust')).toBeInTheDocument();
    expect(screen.queryByText('TypeScript')).not.toBeInTheDocument();
    expect(screen.queryByText('Dart')).not.toBeInTheDocument();
  });
});
