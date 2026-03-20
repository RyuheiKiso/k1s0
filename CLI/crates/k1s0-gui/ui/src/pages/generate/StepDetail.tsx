/**
 * Step 4: 詳細オプションコンポーネント
 * モジュール名、APIスタイル、データベース設定、Kafka/Redis連携、BFF設定を行う
 */

import HelpButton from '../../components/HelpButton';
import { BFF_OPT_IN_NO, BFF_OPT_IN_YES } from '../../constants/messages';
import { BFF_LANGUAGE_VALUES, getDefaultDetailName, type ServerDatabaseMode } from '../../lib/generate-wizard';
import type { ApiStyle, DetailConfig, Kind, Rdbms, ScaffoldDatabaseInfo, Tier } from '../../lib/tauri-commands';

/** StepDetailコンポーネントのprops型定義 */
export interface StepDetailProps {
  /** 現在の種別 */
  kind: Kind;
  /** 現在のティア */
  tier: Tier;
  /** 現在の配置名（Serviceティアで名前の導出に使用） */
  placement: string;
  /** 詳細設定オブジェクト */
  detail: DetailConfig;
  /** 詳細設定変更ハンドラー */
  onDetailChange: React.Dispatch<React.SetStateAction<DetailConfig>>;
  /** APIスタイルのトグルハンドラー */
  onToggleApiStyle: (style: ApiStyle) => void;
  /** 名前のバリデーションエラー */
  nameError: string;
  /** 詳細バリデーションエラー */
  detailError: string;
  /** 詳細名のバリデーション関数 */
  onValidateDetailName: (value: string) => Promise<boolean>;
  /** サーバーデータベースモード */
  serverDatabaseMode: ServerDatabaseMode;
  /** サーバーデータベースモード変更ハンドラー */
  onServerDatabaseModeChange: (mode: ServerDatabaseMode) => void;
  /** 利用可能なデータベース一覧 */
  availableDatabases: ScaffoldDatabaseInfo[];
  /** 選択中のデータベースパス */
  selectedDatabasePath: string;
  /** データベースパス変更ハンドラー */
  onSelectedDatabasePathChange: (path: string) => void;
  /** 新規データベース名 */
  newDatabaseName: string;
  /** 新規データベース名変更ハンドラー */
  onNewDatabaseNameChange: (value: string) => void;
  /** 新規データベースエンジン */
  newDatabaseEngine: Rdbms;
  /** 新規データベースエンジン変更ハンドラー */
  onNewDatabaseEngineChange: (value: Rdbms) => void;
  /** サーバーデータベースエラー */
  serverDatabaseError: string;
  /** サーバーデータベース名のバリデーション関数 */
  onValidateServerDatabaseName: (value: string) => Promise<boolean>;
  /** BFF生成フラグ */
  generateBff: boolean;
  /** BFF生成フラグ変更ハンドラー */
  onGenerateBffChange: (value: boolean) => void;
  /** BFFコントロールを表示するか */
  showBffControls: boolean;
  /** 次へ進むハンドラー */
  onNext: () => void;
  /** 戻るハンドラー */
  onBack: () => void;
  /** react-hook-formのsetValue（フォーム値の同期用） */
  onSetFormValue: (field: string, value: unknown) => void;
  /** react-hook-formのclearErrors */
  onClearErrors: (name?: string | string[]) => void;
}

/** 詳細オプションステップのUIコンポーネント */
export default function StepDetail({
  kind,
  tier,
  placement,
  detail,
  onDetailChange,
  onToggleApiStyle,
  nameError,
  detailError,
  onValidateDetailName,
  serverDatabaseMode,
  onServerDatabaseModeChange,
  availableDatabases,
  selectedDatabasePath,
  onSelectedDatabasePathChange,
  newDatabaseName,
  onNewDatabaseNameChange,
  newDatabaseEngine,
  onNewDatabaseEngineChange,
  serverDatabaseError,
  onValidateServerDatabaseName,
  generateBff,
  onGenerateBffChange,
  showBffControls,
  onNext,
  onBack,
  onSetFormValue,
  onClearErrors,
}: StepDetailProps) {
  return (
    <section
      className="p3-expand-in mt-6 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5"
      data-testid="step-detail"
    >
      <div className="flex items-center gap-2">
        <h2 className="text-lg font-semibold text-white">詳細オプション</h2>
        <HelpButton helpKey="generate.detail" />
      </div>

      {/* Serviceティア以外の場合はモジュール名入力 */}
      {tier !== 'Service' && (
        <div className="mt-4">
          <label className="block text-sm font-medium text-slate-200/82">モジュール名</label>
          <input
            value={detail.name ?? ''}
            onChange={(event) => {
              onDetailChange((current) => ({
                ...current,
                name: event.target.value,
              }));
              onSetFormValue('detailName', event.target.value);
              onClearErrors('detailName');
            }}
            onBlur={() => {
              void onValidateDetailName(detail.name ?? '');
            }}
            placeholder={getDefaultDetailName(kind)}
            className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
            data-testid="input-name"
          />
          {nameError && (
            <p className="mt-2 text-sm text-rose-300" data-testid="error-name">
              {nameError}
            </p>
          )}
        </div>
      )}

      {/* Serviceティアの場合は配置名から導出される旨を表示 */}
      {tier === 'Service' && (
        <div className="mt-4 border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.20)] p-4 text-sm text-slate-200/82">
          サービス名は配置から導出されます: <strong>{placement || '未設定'}</strong>
        </div>
      )}

      {/* Server種別の場合のオプション群 */}
      {kind === 'Server' && (
        <>
          {/* APIスタイル選択 */}
          <div className="mt-5">
            <p className="text-sm font-medium text-slate-200/82">APIスタイル</p>
            <div className="mt-3 space-y-2">
              {(['Rest', 'Grpc', 'GraphQL'] as ApiStyle[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="checkbox"
                    checked={detail.api_styles.includes(value)}
                    onChange={() => onToggleApiStyle(value)}
                  />
                  {value}
                </label>
              ))}
            </div>
          </div>

          {/* データベース設定 */}
          <div className="mt-5">
            <p className="text-sm font-medium text-slate-200/82">データベース</p>
            <div className="mt-3 space-y-3">
              {(['none', 'existing', 'new'] as ServerDatabaseMode[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                    <input
                      type="radio"
                      checked={serverDatabaseMode === value}
                      onChange={() => {
                        onServerDatabaseModeChange(value);
                        onClearErrors(['selectedDatabasePath', 'newDatabaseName']);
                      }}
                      name="server-database-mode"
                    />
                  {value === 'none'
                    ? 'データベースなし'
                    : value === 'existing'
                      ? '既存のデータベースを使用'
                      : '新しいデータベースを作成'}
                </label>
              ))}
            </div>

            {/* 既存データベース選択 */}
            {serverDatabaseMode === 'existing' && (
              <div className="mt-4">
                {availableDatabases.length === 0 ? (
                  <p className="text-sm text-slate-200/55">
                    このティアに既存のデータベースが見つかりませんでした。
                  </p>
                ) : (
                  <>
                    <label className="block text-sm font-medium text-slate-200/82">
                      既存のデータベース
                    </label>
                    <select
                      value={selectedDatabasePath}
                      onChange={(event) => {
                        onSelectedDatabasePathChange(event.target.value);
                        onSetFormValue('selectedDatabasePath', event.target.value);
                        onClearErrors('selectedDatabasePath');
                      }}
                      className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(5,8,15,0.35)] px-3 py-2 text-white"
                      data-testid="select-server-db"
                    >
                      {availableDatabases.map((database) => (
                        <option key={database.path} value={database.path}>
                          {database.name} ({database.rdbms})
                        </option>
                      ))}
                    </select>
                  </>
                )}
              </div>
            )}

            {/* 新規データベース作成 */}
            {serverDatabaseMode === 'new' && (
              <div className="mt-4 space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-200/82">
                    データベース名
                  </label>
                  <input
                    value={newDatabaseName}
                    onChange={(event) => {
                      onNewDatabaseNameChange(event.target.value);
                      onSetFormValue('newDatabaseName', event.target.value);
                      onClearErrors('newDatabaseName');
                    }}
                    onBlur={() => {
                      void onValidateServerDatabaseName(newDatabaseName);
                    }}
                    placeholder="service-db"
                    className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                    data-testid="input-server-db-name"
                  />
                </div>
                {/* 新規DBのRDBMSエンジン選択 */}
                <div className="space-y-2">
                  {(['PostgreSQL', 'MySQL', 'SQLite'] as Rdbms[]).map((value) => (
                    <label
                      key={value}
                      className="flex items-center gap-3 text-sm text-slate-200/82"
                    >
                      <input
                        type="radio"
                        checked={newDatabaseEngine === value}
                        onChange={() => onNewDatabaseEngineChange(value)}
                        name="server-database-engine"
                      />
                      {value}
                    </label>
                  ))}
                </div>
              </div>
            )}

            {/* サーバーデータベースエラー表示 */}
            {serverDatabaseError && (
              <p className="mt-3 text-sm text-rose-300">{serverDatabaseError}</p>
            )}
          </div>

          {/* Kafka/Redis連携オプション */}
          <div className="mt-5 space-y-2">
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="checkbox"
                checked={detail.kafka}
                onChange={(event) =>
                  onDetailChange((current) => ({
                    ...current,
                    kafka: event.target.checked,
                  }))
                }
              />
              Kafka連携を有効にする
            </label>
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="checkbox"
                checked={detail.redis}
                onChange={(event) =>
                  onDetailChange((current) => ({
                    ...current,
                    redis: event.target.checked,
                  }))
                }
              />
              Redis連携を有効にする
            </label>
          </div>

          {/* BFF生成オプション（GraphQL選択時のみ表示） */}
          {showBffControls && (
            <div className="mt-5">
              <p className="text-sm font-medium text-slate-200/82">
                GraphQL BFFを生成する
              </p>
              <div className="mt-3 space-y-2">
                {[
                  { label: BFF_OPT_IN_YES, enabled: true },
                  { label: BFF_OPT_IN_NO, enabled: false },
                ].map(({ label, enabled }) => (
                  <label key={label} className="flex items-center gap-3 text-sm text-slate-200/82">
                    <input
                      type="radio"
                      checked={generateBff === enabled}
                      onChange={() => {
                        onGenerateBffChange(enabled);
                        onSetFormValue('generateBff', enabled);
                        if (!enabled) {
                          onDetailChange((current) => ({
                            ...current,
                            bff_language: null,
                          }));
                          onSetFormValue('bffLanguage', null);
                          onClearErrors('bffLanguage');
                        }
                      }}
                      name="generate-bff"
                    />
                    {label}
                  </label>
                ))}
              </div>

              {/* BFF言語選択 */}
              {generateBff && (
                <div className="mt-4">
                  <p className="text-sm font-medium text-slate-200/82">BFF言語</p>
                  <div className="mt-3 space-y-2">
                    {BFF_LANGUAGE_VALUES.map((value) => (
                      <label
                        key={value}
                        className="flex items-center gap-3 text-sm text-slate-200/82"
                      >
                        <input
                          type="radio"
                          checked={detail.bff_language === value}
                          onChange={() => {
                            onDetailChange((current) => ({
                              ...current,
                              bff_language: value,
                            }));
                            onSetFormValue('bffLanguage', value);
                            onClearErrors('bffLanguage');
                          }}
                          name="bff-language"
                        />
                        {value}
                      </label>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </>
      )}

      {/* 詳細エラー表示 */}
      {detailError && (
        <p className="mt-4 text-sm text-rose-300" data-testid="detail-error">
          {detailError}
        </p>
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
    </section>
  );
}
