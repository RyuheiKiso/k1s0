import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import ProgressLog from '../ProgressLog';
import type { ProgressEvent } from '../../lib/tauri-commands';

describe('ProgressLog', () => {
  it('should render empty log message when no events', () => {
    render(<ProgressLog events={[]} currentStep={0} totalSteps={0} />);
    expect(screen.getByTestId('log-viewer')).toHaveTextContent('ログはありません');
  });

  it('should render progress bar with correct percentage', () => {
    const events: ProgressEvent[] = [
      { kind: 'StepCompleted', step: 1, total: 4, message: '完了' },
    ];
    render(<ProgressLog events={events} currentStep={2} totalSteps={4} />);
    expect(screen.getByTestId('progress-label')).toHaveTextContent('ステップ 2 / 4');
    expect(screen.getByTestId('progress-percent')).toHaveTextContent('50%');
    expect(screen.getByTestId('progress-bar-bg')).toHaveAttribute('aria-valuenow', '50');
  });

  it('should render all event types correctly', () => {
    const events: ProgressEvent[] = [
      { kind: 'StepStarted', step: 1, total: 2, message: 'ビルド中' },
      { kind: 'Log', message: 'go build ./...' },
      { kind: 'Warning', message: 'ディレクトリが見つかりません' },
      { kind: 'StepCompleted', step: 1, total: 2, message: 'ビルド完了' },
      { kind: 'Error', message: 'コマンド失敗' },
      { kind: 'Finished', success: true, message: '全体完了' },
    ];
    render(<ProgressLog events={events} currentStep={2} totalSteps={2} />);

    const logViewer = screen.getByTestId('log-viewer');
    expect(logViewer).toHaveTextContent('ビルド中');
    expect(logViewer).toHaveTextContent('go build');
    expect(logViewer).toHaveTextContent('ディレクトリが見つかりません');
    expect(logViewer).toHaveTextContent('ビルド完了');
    expect(logViewer).toHaveTextContent('コマンド失敗');
    expect(logViewer).toHaveTextContent('全体完了');
  });

  it('should not show progress bar when totalSteps is 0', () => {
    render(<ProgressLog events={[]} currentStep={0} totalSteps={0} />);
    expect(screen.queryByTestId('progress-bar')).not.toBeInTheDocument();
  });

  it('should show 100% when all steps complete', () => {
    const events: ProgressEvent[] = [
      { kind: 'Finished', success: true, message: '完了' },
    ];
    render(<ProgressLog events={events} currentStep={3} totalSteps={3} />);
    expect(screen.getByTestId('progress-percent')).toHaveTextContent('100%');
    expect(screen.getByTestId('progress-bar-bg')).toHaveAttribute('aria-valuenow', '100');
  });

  it('should render failure finished event differently', () => {
    const events: ProgressEvent[] = [
      { kind: 'Finished', success: false, message: '失敗しました' },
    ];
    render(<ProgressLog events={events} currentStep={1} totalSteps={2} />);
    const logViewer = screen.getByTestId('log-viewer');
    expect(logViewer).toHaveTextContent('失敗しました');
  });
});
