/**
 * Step 2: 配置選択コンポーネント
 * 既存の配置から選択するか、新しい配置名を入力する
 */

/** StepPlacementコンポーネントのprops型定義 */
export interface StepPlacementProps {
  /** 現在の配置名 */
  placement: string;
  /** 配置名変更ハンドラー */
  onPlacementChange: (value: string) => void;
  /** 既存の配置一覧 */
  existingPlacements: string[];
  /** 新規配置フラグ */
  isNewPlacement: boolean;
  /** 新規配置フラグ変更ハンドラー */
  onIsNewPlacementChange: (value: boolean) => void;
  /** バリデーションエラーメッセージ */
  placementError: string;
  /** 配置名のバリデーション関数 */
  onValidatePlacement: (value: string) => Promise<boolean>;
  /** 次へ進むハンドラー */
  onNext: () => void;
  /** 戻るハンドラー */
  onBack: () => void;
  /** 利用可否エラーメッセージ */
  availabilityErrorMessage: string;
  /** react-hook-formのsetValue（配置値の同期用） */
  onSetFormValue: (value: string) => void;
  /** react-hook-formのclearErrors（配置エラーのクリア用） */
  onClearPlacementError: () => void;
}

/** 配置選択ステップのUIコンポーネント */
export default function StepPlacement({
  placement,
  onPlacementChange,
  existingPlacements,
  isNewPlacement,
  onIsNewPlacementChange,
  placementError,
  onValidatePlacement,
  onNext,
  onBack,
  availabilityErrorMessage,
  onSetFormValue,
  onClearPlacementError,
}: StepPlacementProps) {
  return (
    <section
      className="mt-6 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5"
      data-testid="step-placement"
    >
      <h2 className="text-lg font-semibold text-white">配置を選択</h2>

      {/* 既存配置がある場合のセレクトボックス */}
      {existingPlacements.length > 0 && (
        <div className="mt-4">
          <label className="block text-sm font-medium text-slate-200/82">
            既存の配置
          </label>
          <select
            className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(5,8,15,0.35)] px-3 py-2 text-white"
            value={isNewPlacement ? '__new__' : placement}
            onChange={(event) => {
              if (event.target.value === '__new__') {
                onIsNewPlacementChange(true);
                onPlacementChange('');
                onSetFormValue('');
              } else {
                onIsNewPlacementChange(false);
                onPlacementChange(event.target.value);
                onSetFormValue(event.target.value);
                onClearPlacementError();
              }
            }}
            data-testid="select-placement"
          >
            <option value="__new__">新しい配置を作成</option>
            {existingPlacements.map((value) => (
              <option key={value} value={value}>
                {value}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* 新規配置名の入力フィールド */}
      {(isNewPlacement || existingPlacements.length === 0) && (
        <div className="mt-4">
          <label className="block text-sm font-medium text-slate-200/82">配置名</label>
          <input
            value={placement}
            onChange={(event) => {
              onPlacementChange(event.target.value);
              onSetFormValue(event.target.value);
              onClearPlacementError();
            }}
            onBlur={() => {
              void onValidatePlacement(placement);
            }}
            placeholder="placement-name"
            className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
            data-testid={
              existingPlacements.length > 0 ? 'input-new-placement' : 'input-placement'
            }
          />
          {placementError && (
            <p className="mt-2 text-sm text-rose-300" data-testid="error-placement">
              {placementError}
            </p>
          )}
        </div>
      )}

      {/* ナビゲーションボタン */}
      <div className="mt-5 flex gap-3">
        <button
          type="button"
          onClick={onBack}
          className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)]"
          data-testid="btn-back"
        >
          戻る
        </button>
        <button
          type="button"
          onClick={onNext}
          className="bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500"
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
