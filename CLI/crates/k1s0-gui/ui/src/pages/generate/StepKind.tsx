/**
 * Step 0: モジュール種別選択コンポーネント
 * Server / Client / Library / Database の4種別から選択する
 */

import HelpButton from '../../components/HelpButton';
import type { Kind } from '../../lib/tauri-commands';

/** StepKindコンポーネントのprops型定義 */
export interface StepKindProps {
  /** 現在選択されている種別 */
  kind: Kind;
  /** 種別変更ハンドラー */
  onKindChange: (nextKind: Kind) => void;
  /** 次へ進むハンドラー */
  onNext: () => void;
}

/** 選択可能な種別の一覧 */
const KIND_OPTIONS: Kind[] = ['Server', 'Client', 'Library', 'Database'];

/** 種別選択ステップのUIコンポーネント */
export default function StepKind({ kind, onKindChange, onNext }: StepKindProps) {
  return (
    <section
      className="p3-expand-in mt-6 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5"
      data-testid="step-kind"
    >
      <div className="flex items-center gap-2">
        <h2 className="text-lg font-semibold text-white">モジュール種別を選択</h2>
        <HelpButton helpKey="generate.kind" />
      </div>
      {/* 4種別のラジオボタングリッド */}
      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        {KIND_OPTIONS.map((value) => (
          <label
            key={value}
            className="flex items-center gap-3 border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-4 py-3 text-sm text-slate-100"
          >
            <input
              type="radio"
              checked={kind === value}
              onChange={() => onKindChange(value)}
              name="generate-kind"
            />
            {value}
          </label>
        ))}
      </div>
      {/* 次へボタン */}
      <button
        type="button"
        onClick={onNext}
        className="mt-5 bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500"
        data-testid="btn-next"
      >
        次へ
      </button>
    </section>
  );
}
