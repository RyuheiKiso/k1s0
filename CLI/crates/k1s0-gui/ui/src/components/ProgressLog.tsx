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
    <section className="mt-5" data-testid="progress-log">
      {totalSteps > 0 && (
        <div className="mb-3">
          <div className="mb-1 flex items-center justify-between text-sm text-slate-200/72">
            <span data-testid="progress-label">
              Step {currentStep} / {totalSteps}
            </span>
            <span data-testid="progress-percent">{percentage}%</span>
          </div>
          <Progress.Root value={percentage} max={100} data-testid="progress-bar-bg">
            <Progress.Indicator style={{ width: `${percentage}%` }} data-testid="progress-bar" />
          </Progress.Root>
        </div>
      )}

      <div
        className="glass-subtle max-h-72 overflow-y-auto rounded-2xl border border-white/10 p-3 font-mono text-xs"
        data-testid="log-viewer"
      >
        {events.length === 0 ? (
          <p className="text-slate-200/35">No logs yet.</p>
        ) : (
          events.map((event, index) => (
            <div key={`${event.kind}-${index}`} className={getEventClassName(event)}>
              {formatEvent(event)}
            </div>
          ))
        )}
      </div>
    </section>
  );
}

function getEventClassName(event: ProgressEvent): string {
  switch (event.kind) {
    case 'StepStarted':
      return 'text-sky-200';
    case 'StepCompleted':
      return 'text-emerald-200';
    case 'Log':
      return 'text-slate-200/65';
    case 'Warning':
      return 'text-amber-200';
    case 'Error':
      return 'text-rose-300';
    case 'Finished':
      return event.success ? 'font-semibold text-emerald-300' : 'font-semibold text-rose-300';
  }
}

function formatEvent(event: ProgressEvent): string {
  switch (event.kind) {
    case 'StepStarted':
      return `[${event.step}/${event.total}] start ${event.message}`;
    case 'StepCompleted':
      return `[${event.step}/${event.total}] done ${event.message}`;
    case 'Log':
      return `log ${event.message}`;
    case 'Warning':
      return `warn ${event.message}`;
    case 'Error':
      return `error ${event.message}`;
    case 'Finished':
      return event.success ? `success ${event.message}` : `failure ${event.message}`;
  }
}
