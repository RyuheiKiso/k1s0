/**
 * Step 3: 言語/フレームワーク選択コンポーネント
 * 種別に応じてフレームワーク、データベース設定、または言語を選択する
 */

import { getLanguageOptions } from '../../lib/generate-wizard';
import type { Framework, Kind, Language, Rdbms } from '../../lib/tauri-commands';

/** StepLangFwコンポーネントのprops型定義 */
export interface StepLangFwProps {
  /** 現在の種別 */
  kind: Kind;
  /** 選択中の言語 */
  language: Language;
  /** 言語変更ハンドラー */
  onLanguageChange: (value: Language) => void;
  /** 選択中のフレームワーク */
  framework: Framework;
  /** フレームワーク変更ハンドラー */
  onFrameworkChange: (value: Framework) => void;
  /** データベース名 */
  databaseName: string;
  /** データベース名変更ハンドラー */
  onDatabaseNameChange: (value: string) => void;
  /** データベースエンジン */
  databaseEngine: Rdbms;
  /** データベースエンジン変更ハンドラー */
  onDatabaseEngineChange: (value: Rdbms) => void;
  /** 名前のバリデーションエラー */
  nameError: string;
  /** データベース名のバリデーション関数 */
  onValidateDatabaseName: (
    field: 'placement' | 'detailName' | 'databaseName' | 'newDatabaseName',
    value: string,
  ) => Promise<boolean>;
  /** 次へ進むハンドラー */
  onNext: () => void;
  /** 戻るハンドラー */
  onBack: () => void;
  /** 利用可否エラーメッセージ */
  availabilityErrorMessage: string;
  /** react-hook-formのsetValue */
  onSetDatabaseNameFormValue: (value: string) => void;
  /** react-hook-formのclearErrors */
  onClearDatabaseNameError: () => void;
}

/** 言語/フレームワーク選択ステップのUIコンポーネント */
export default function StepLangFw({
  kind,
  language,
  onLanguageChange,
  framework,
  onFrameworkChange,
  databaseName,
  onDatabaseNameChange,
  databaseEngine,
  onDatabaseEngineChange,
  nameError,
  onValidateDatabaseName,
  onNext,
  onBack,
  availabilityErrorMessage,
  onSetDatabaseNameFormValue,
  onClearDatabaseNameError,
}: StepLangFwProps) {
  return (
    <section
      className="mt-6 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5"
      data-testid="step-langfw"
    >
      <h2 className="text-lg font-semibold text-white">言語またはフレームワーク</h2>

      {/* Client種別の場合はフレームワーク選択 */}
      {kind === 'Client' && (
        <div className="mt-4 space-y-2">
          {(['React', 'Flutter'] as Framework[]).map((value) => (
            <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={framework === value}
                onChange={() => onFrameworkChange(value)}
                name="client-framework"
              />
              {value}
            </label>
          ))}
        </div>
      )}

      {/* Database種別の場合はDB名とエンジン選択 */}
      {kind === 'Database' && (
        <div className="mt-4 space-y-5">
          <div>
            <label className="block text-sm font-medium text-slate-200/82">データベース名</label>
            <input
              value={databaseName}
              onChange={(event) => {
                onDatabaseNameChange(event.target.value);
                onSetDatabaseNameFormValue(event.target.value);
                onClearDatabaseNameError();
              }}
              onBlur={() => {
                void onValidateDatabaseName('databaseName', databaseName);
              }}
              placeholder="main"
              className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
              data-testid="input-db-name"
            />
            {nameError && (
              <p className="mt-2 text-sm text-rose-300" data-testid="error-name">
                {nameError}
              </p>
            )}
          </div>
          {/* RDBMSエンジン選択 */}
          <div className="space-y-2">
            {(['PostgreSQL', 'MySQL', 'SQLite'] as Rdbms[]).map((value) => (
              <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={databaseEngine === value}
                  onChange={() => onDatabaseEngineChange(value)}
                  name="database-engine"
                />
                {value}
              </label>
            ))}
          </div>
        </div>
      )}

      {/* Server/Library種別の場合は言語選択 */}
      {kind !== 'Client' && kind !== 'Database' && (
        <div className="mt-4 space-y-2">
          {getLanguageOptions(kind).map((value) => (
            <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={language === value}
                onChange={() => onLanguageChange(value)}
                name="module-language"
              />
              {value}
            </label>
          ))}
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
