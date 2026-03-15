/**
 * Step 5: 確認・実行コンポーネント
 * 選択した設定の最終確認と生成実行を行う
 */

import {
  BFF_GENERATE_LABEL,
  BFF_GENERATE_NO,
  BFF_GENERATE_UNAVAILABLE,
  BFF_GENERATE_YES,
  BFF_LANGUAGE_LABEL,
  BFF_LANGUAGE_NONE,
} from '../../constants/messages';
import type { ServerDatabaseMode } from '../../lib/generate-wizard';
import type { DetailConfig, Kind, LangFw, Rdbms, ScaffoldDatabaseInfo, Tier } from '../../lib/tauri-commands';

/** StepConfirmコンポーネントのprops型定義 */
export interface StepConfirmProps {
  /** 現在の種別 */
  kind: Kind;
  /** 現在のティア */
  tier: Tier;
  /** 配置ステップを表示するか */
  showPlacementStep: boolean;
  /** 現在の配置名 */
  placement: string;
  /** ランタイム情報（言語/フレームワーク/DB） */
  currentRuntime: LangFw;
  /** 解決済みの詳細名 */
  resolvedDetailName: string;
  /** 詳細設定 */
  detail: DetailConfig;
  /** 解決済みのサーバーデータベース設定 */
  resolvedServerDatabase: { name: string; rdbms: Rdbms } | null;
  /** データベースエンジン（Database種別用） */
  databaseEngine: Rdbms;
  /** データベース名（Database種別用） */
  databaseName: string;
  /** サーバーデータベースモード */
  serverDatabaseMode: ServerDatabaseMode;
  /** 選択中の既存データベース情報 */
  selectedExistingDatabase: ScaffoldDatabaseInfo | null;
  /** BFFコントロールを表示するか */
  showBffControls: boolean;
  /** BFF生成フラグ */
  generateBff: boolean;
  /** 選択中のBFF言語 */
  selectedBffLanguage: string | null;
  /** 生成ステータス */
  status: 'idle' | 'loading' | 'success' | 'error';
  /** エラーメッセージ */
  errorMessage: string;
  /** 利用可否エラーメッセージ */
  availabilityErrorMessage: string;
  /** ワークスペース利用不可フラグ */
  workspaceUnavailable: boolean;
  /** 認証ロック状態 */
  actionsLocked: boolean;
  /** 生成実行ハンドラー */
  onGenerate: () => void;
  /** 戻るハンドラー */
  onBack: () => void;
}

/** 確認・実行ステップのUIコンポーネント */
export default function StepConfirm({
  kind,
  tier,
  showPlacementStep,
  placement,
  currentRuntime,
  resolvedDetailName,
  detail,
  resolvedServerDatabase,
  databaseEngine,
  databaseName,
  serverDatabaseMode,
  selectedExistingDatabase,
  showBffControls,
  generateBff,
  selectedBffLanguage,
  status,
  errorMessage,
  availabilityErrorMessage,
  workspaceUnavailable,
  actionsLocked,
  onGenerate,
  onBack,
}: StepConfirmProps) {
  return (
    <section
      className="mt-6 rounded-2xl border border-white/10 bg-white/5 p-5"
      data-testid="step-confirm"
    >
      <h2 className="text-lg font-semibold text-white">生成の確認</h2>
      {/* 設定サマリー */}
      <div className="mt-4 space-y-3 text-sm text-slate-200/82">
        <p>種別: {kind}</p>
        <p>ティア: {tier}</p>
        <p>配置: {showPlacementStep ? placement || '未設定' : '不要'}</p>
        {/* ランタイム情報の表示（ユニオン型の分岐） */}
        <p>
          ランタイム:{' '}
          {'Framework' in currentRuntime
            ? currentRuntime.Framework
            : 'Database' in currentRuntime
              ? `${currentRuntime.Database.rdbms} (${currentRuntime.Database.name})`
              : currentRuntime.Language}
        </p>
        <p>名前: {resolvedDetailName}</p>
        {/* Server種別の追加情報 */}
        {kind === 'Server' && (
          <>
            <p>APIスタイル: {detail.api_styles.length > 0 ? detail.api_styles.join(', ') : 'なし'}</p>
            <p>
              データベース:{' '}
              {resolvedServerDatabase
                ? `${resolvedServerDatabase.name} (${resolvedServerDatabase.rdbms})`
                : 'なし'}
            </p>
            <p>Kafka: {detail.kafka ? '有効' : '無効'}</p>
            <p>Redis: {detail.redis ? '有効' : '無効'}</p>
            <p>{BFF_GENERATE_LABEL} {showBffControls ? (generateBff ? BFF_GENERATE_YES : BFF_GENERATE_NO) : BFF_GENERATE_UNAVAILABLE}</p>
            <p>{BFF_LANGUAGE_LABEL} {selectedBffLanguage ?? BFF_LANGUAGE_NONE}</p>
          </>
        )}
        {/* Database種別の追加情報 */}
        {kind === 'Database' && (
          <p>
            RDBMS: {databaseEngine} ({databaseName})
          </p>
        )}
        {/* 既存DB使用時のパス表示 */}
        {serverDatabaseMode === 'existing' && selectedExistingDatabase && (
          <p className="text-slate-300/60">
            既存DBパス: {selectedExistingDatabase.path}
          </p>
        )}
      </div>

      {/* ナビゲーション・生成ボタン */}
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
          onClick={onGenerate}
          disabled={status === 'loading' || workspaceUnavailable || actionsLocked}
          className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
          data-testid="btn-generate"
        >
          {status === 'loading' ? '生成中...' : '生成'}
        </button>
      </div>

      {/* 成功メッセージ */}
      {status === 'success' && (
        <p className="mt-4 text-sm text-emerald-300" data-testid="success-message">
          生成が正常に完了しました。
        </p>
      )}
      {/* エラーメッセージ */}
      {status === 'error' && (
        <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
          {errorMessage}
        </p>
      )}
      {/* 利用可否エラー表示 */}
      {availabilityErrorMessage && (
        <p className="mt-4 text-sm text-rose-300" data-testid="availability-error">
          {availabilityErrorMessage}
        </p>
      )}
    </section>
  );
}
