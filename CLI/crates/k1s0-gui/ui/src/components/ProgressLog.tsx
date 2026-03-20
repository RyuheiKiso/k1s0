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
    <section className="mt-5 p3-expand-in" data-testid="progress-log">
      {totalSteps > 0 && (
        <div className="mb-3">
          {/* プログレスラベル — シアンテーマ */}
          <div className="mb-1 flex items-center justify-between text-sm text-slate-200/72">
            <span data-testid="progress-label" className="p3-heading-glow">
              ステップ {currentStep} / {totalSteps}
            </span>
            <span data-testid="progress-percent">{percentage}%</span>
          </div>
          {/* プログレスバー — スキャンラインエフェクト付き */}
          <Progress.Root value={percentage} max={100} data-testid="progress-bar-bg">
            <Progress.Indicator style={{ width: `${percentage}%` }} data-testid="progress-bar" />
          </Progress.Root>
        </div>
      )}

      {/* ログビューア — シアンボーダー */}
      <div
        className="glass-subtle p3-data-rain max-h-72 overflow-y-auto border border-[rgba(0,200,255,0.12)] p-3 font-mono text-xs"
        data-testid="log-viewer"
      >
        {events.length === 0 ? (
          <p className="text-slate-200/35">ログはまだありません。</p>
        ) : (
          events.map((event, index) => (
            <div
              key={`${event.kind}-${index}`}
              className={`p3-log-cascade ${getEventClassName(event)}`}
              style={{ '--p3-stagger': index } as React.CSSProperties}
            >
              {formatEvent(event)}
            </div>
          ))
        )}
      </div>
    </section>
  );
}

/* イベント種別に応じたシアン系カラー割り当て */
function getEventClassName(event: ProgressEvent): string {
  switch (event.kind) {
    case 'StepStarted':
      return 'text-cyan-200';
    case 'StepCompleted':
      return 'text-cyan-300';
    case 'Log':
      return 'text-slate-200/65';
    case 'Warning':
      return 'text-red-200';
    case 'Error':
      return 'text-rose-300';
    case 'Finished':
      return event.success ? 'font-semibold text-cyan-300' : 'font-semibold text-rose-300';
  }
}

/* フォーマット文字列はテスト互換のため変更なし */
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
