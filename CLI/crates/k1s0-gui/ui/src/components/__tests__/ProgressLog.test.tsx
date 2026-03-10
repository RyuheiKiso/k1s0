import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import ProgressLog from '../ProgressLog';
import type { ProgressEvent } from '../../lib/tauri-commands';

describe('ProgressLog', () => {
  it('renders an empty state when no events are present', () => {
    render(<ProgressLog events={[]} currentStep={0} totalSteps={0} />);
    expect(screen.getByTestId('log-viewer')).toHaveTextContent('No logs yet.');
  });

  it('renders progress percentage from the current step', () => {
    const events: ProgressEvent[] = [{ kind: 'StepCompleted', step: 1, total: 4, message: 'ok' }];
    render(<ProgressLog events={events} currentStep={2} totalSteps={4} />);
    expect(screen.getByTestId('progress-label')).toHaveTextContent('Step 2 / 4');
    expect(screen.getByTestId('progress-percent')).toHaveTextContent('50%');
    expect(screen.getByTestId('progress-bar-bg')).toHaveAttribute('aria-valuenow', '50');
  });

  it('formats all event kinds into readable log lines', () => {
    const events: ProgressEvent[] = [
      { kind: 'StepStarted', step: 1, total: 2, message: 'build auth' },
      { kind: 'Log', message: 'cargo build' },
      { kind: 'Warning', message: 'rollback available' },
      { kind: 'StepCompleted', step: 1, total: 2, message: 'built auth' },
      { kind: 'Error', message: 'command failed' },
      { kind: 'Finished', success: true, message: 'done' },
    ];

    render(<ProgressLog events={events} currentStep={2} totalSteps={2} />);

    const logViewer = screen.getByTestId('log-viewer');
    expect(logViewer).toHaveTextContent('start build auth');
    expect(logViewer).toHaveTextContent('log cargo build');
    expect(logViewer).toHaveTextContent('warn rollback available');
    expect(logViewer).toHaveTextContent('done built auth');
    expect(logViewer).toHaveTextContent('error command failed');
    expect(logViewer).toHaveTextContent('success done');
  });

  it('hides the progress bar when no total steps are known', () => {
    render(<ProgressLog events={[]} currentStep={0} totalSteps={0} />);
    expect(screen.queryByTestId('progress-bar')).not.toBeInTheDocument();
  });
});
