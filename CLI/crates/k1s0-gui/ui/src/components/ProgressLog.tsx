import * as Progress from '@radix-ui/react-progress';
import type { ProgressEvent } from '../lib/tauri-commands';

interface ProgressLogProps {
  events: ProgressEvent[];
  currentStep: number;
  totalSteps: number;
}

export default function ProgressLog({ events, currentStep, totalSteps }: ProgressLogProps) {
  const percentage = totalSteps > 0 ? Math.round((currentStep / totalSteps) * 100) : 0;

  return (
    <div data-testid="progress-log" className="mt-4">
      {totalSteps > 0 && (
        <div className="mb-3">
          <div className="flex justify-between text-sm mb-1 text-white/70">
            <span data-testid="progress-label">
              ステップ {currentStep} / {totalSteps}
            </span>
            <span data-testid="progress-percent">{percentage}%</span>
          </div>
          <Progress.Root value={percentage} max={100} data-testid="progress-bar-bg">
            <Progress.Indicator
              style={{ width: `${percentage}%` }}
              data-testid="progress-bar"
            />
          </Progress.Root>
        </div>
      )}

      <div
        className="glass-subtle p-3 font-mono text-xs max-h-60 overflow-y-auto"
        data-testid="log-viewer"
      >
        {events.length === 0 ? (
          <p className="text-white/30">ログはありません。</p>
        ) : (
          events.map((event, i) => (
            <div key={i} className={getEventClassName(event)}>
              {formatEvent(event)}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function getEventClassName(event: ProgressEvent): string {
  switch (event.kind) {
    case 'StepStarted':
      return 'text-indigo-300';
    case 'StepCompleted':
      return 'text-emerald-300';
    case 'Log':
      return 'text-white/60';
    case 'Warning':
      return 'text-amber-300';
    case 'Error':
      return 'text-rose-400';
    case 'Finished':
      return event.success ? 'text-emerald-400 font-bold' : 'text-rose-400 font-bold';
  }
}

function formatEvent(event: ProgressEvent): string {
  switch (event.kind) {
    case 'StepStarted':
      return `[${event.step}/${event.total}] ${event.message} ...`;
    case 'StepCompleted':
      return `[${event.step}/${event.total}] \u2713 ${event.message}`;
    case 'Log':
      return `  ${event.message}`;
    case 'Warning':
      return `  \u26a0 ${event.message}`;
    case 'Error':
      return `  \u2717 ${event.message}`;
    case 'Finished':
      return event.success ? `\u2713 ${event.message}` : `\u2717 ${event.message}`;
  }
}
