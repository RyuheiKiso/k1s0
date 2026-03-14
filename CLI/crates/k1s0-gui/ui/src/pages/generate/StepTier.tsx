/**
 * Step 1: ティア選択コンポーネント
 * 種別に応じた利用可能なティア一覧から選択する
 */

import { getAvailableTiers } from '../../lib/generate-wizard';
import type { Kind, Tier } from '../../lib/tauri-commands';

/** StepTierコンポーネントのprops型定義 */
export interface StepTierProps {
  /** 現在の種別（利用可能なティア一覧の決定に使用） */
  kind: Kind;
  /** 現在選択されているティア */
  tier: Tier;
  /** ティア変更ハンドラー */
  onTierChange: (nextTier: Tier) => void;
  /** 次へ進むハンドラー */
  onNext: () => void;
  /** 戻るハンドラー */
  onBack: () => void;
  /** 利用可否エラーメッセージ */
  availabilityErrorMessage: string;
}

/** ティア選択ステップのUIコンポーネント */
export default function StepTier({
  kind,
  tier,
  onTierChange,
  onNext,
  onBack,
  availabilityErrorMessage,
}: StepTierProps) {
  return (
    <section
      className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
      data-testid="step-tier"
    >
      <h2 className="text-lg font-semibold text-white">ティアを選択</h2>
      {/* ティアのラジオボタン一覧 */}
      <div className="mt-4 space-y-2">
        {getAvailableTiers(kind).map((value) => (
          <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
            <input
              type="radio"
              checked={tier === value}
              onChange={() => onTierChange(value)}
              name="generate-tier"
            />
            {value.toLowerCase()}
          </label>
        ))}
      </div>
      {/* ナビゲーションボタン */}
      <div className="mt-5 flex gap-3">
        <button
          type="button"
          onClick={onBack}
          className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
          data-testid="btn-back"
        >
          戻る
        </button>
        <button
          type="button"
          onClick={onNext}
          className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
          data-testid="btn-next"
        >
          次へ
        </button>
      </div>
      {/* 利用可否エラー表示 */}
      {availabilityErrorMessage && (
        <p className="mt-4 text-sm text-rose-300" data-testid="availability-error">
          {availabilityErrorMessage}
        </p>
      )}
    </section>
  );
}
