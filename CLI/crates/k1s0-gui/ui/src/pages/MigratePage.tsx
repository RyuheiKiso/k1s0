import { useEffect, useState, type ReactNode } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import {
  executeMigrateCreate,
  executeMigrateDown,
  executeMigrateRepair,
  executeMigrateStatus,
  executeMigrateUp,
  scanMigrateTargets,
  type DbConnection,
  type MigrateDownConfig,
  type MigrateRange,
  type MigrateTarget,
  type MigrateUpConfig,
  type MigrationStatus,
  type RepairOperation,
} from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

type ConnectionMode = 'local' | 'custom';
type UpRangeMode = 'all' | 'upTo';
type DownRangeMode = 'previous' | 'upTo' | 'all';
type RepairMode = 'clearDirty' | 'forceVersion';
type PendingMigrationAction =
  | { kind: 'up'; config: MigrateUpConfig }
  | { kind: 'down'; config: MigrateDownConfig };

export default function MigratePage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [targets, setTargets] = useState<MigrateTarget[]>([]);
  const [selectedTargetKey, setSelectedTargetKey] = useState('');
  const [connectionMode, setConnectionMode] = useState<ConnectionMode>('local');
  const [customConnection, setCustomConnection] = useState('');
  const [migrationName, setMigrationName] = useState('');
  const [upRangeMode, setUpRangeMode] = useState<UpRangeMode>('all');
  const [upRangeValue, setUpRangeValue] = useState('1');
  const [downRangeMode, setDownRangeMode] = useState<DownRangeMode>('previous');
  const [downRangeValue, setDownRangeValue] = useState('1');
  const [repairMode, setRepairMode] = useState<RepairMode>('clearDirty');
  const [repairVersion, setRepairVersion] = useState('1');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [createdFiles, setCreatedFiles] = useState<[string, string] | null>(null);
  const [migrationStatus, setMigrationStatus] = useState<MigrationStatus[]>([]);
  const [pendingAction, setPendingAction] = useState<PendingMigrationAction | null>(null);

  useEffect(() => {
    let cancelled = false;

    if (!workspace.ready || !workspace.workspaceRoot) {
      return;
    }

    scanMigrateTargets(activeWorkspaceRoot)
      .then((nextTargets) => {
        if (!cancelled) {
          setTargets(nextTargets);
          setSelectedTargetKey((current) => current || encodeTarget(nextTargets[0]));
        }
      })
      .catch(() => {
        if (!cancelled) {
          setTargets([]);
          setSelectedTargetKey('');
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceRoot, workspace.ready, workspace.workspaceRoot]);

  const availableTargets = workspace.ready && workspace.workspaceRoot ? targets : [];
  const activeTargetKey =
    selectedTargetKey && availableTargets.some((target) => encodeTarget(target) === selectedTargetKey)
      ? selectedTargetKey
      : encodeTarget(availableTargets[0]);
  const selectedTarget =
    availableTargets.find((target) => encodeTarget(target) === activeTargetKey) ?? null;

  // 接続情報を構築する
  function buildConnection(): DbConnection {
    return connectionMode === 'local' ? 'LocalDev' : { Custom: customConnection };
  }

  // マイグレーションアップの範囲を構築する
  function buildUpRange(mode: UpRangeMode, value: string): MigrateRange {
    return mode === 'all' ? 'All' : { UpTo: Number(value) };
  }

  // マイグレーションダウンの範囲を構築する
  function buildDownRange(mode: DownRangeMode, value: string): MigrateRange {
    if (mode === 'previous') {
      return { Steps: 1 };
    }
    return mode === 'all' ? 'All' : { UpTo: Number(value) };
  }

  // 結果表示をリセットする
  function resetResult() {
    setCreatedFiles(null);
    setMigrationStatus([]);
    setPendingAction(null);
  }

  // マイグレーション範囲を日本語で説明する
  function describeRange(range: MigrateRange): string {
    if (range === 'All') {
      return '全マイグレーション';
    }
    if ('Steps' in range) {
      return `${range.Steps}件のマイグレーションをロールバック`;
    }
    return `バージョン${range.UpTo}まで`;
  }

  // 接続先を日本語で説明する
  function describeConnection(connection: DbConnection): string {
    return connection === 'LocalDev' ? 'ローカル開発データベース' : connection.Custom;
  }

  // マイグレーションアクションを実行する
  async function runAction(action: () => Promise<void>) {
    setStatus('loading');
    setErrorMessage('');

    try {
      await action();
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  // マイグレーションファイルを作成する
  async function handleCreate() {
    if (!selectedTarget) {
      return;
    }

    setStatus('loading');
    setErrorMessage('');
    setMigrationStatus([]);

    try {
      const files = await executeMigrateCreate({
        target: selectedTarget,
        migration_name: migrationName,
      });
      setCreatedFiles(files);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  // マイグレーションステータスを取得する
  async function handleStatus() {
    if (!selectedTarget) {
      return;
    }

    setStatus('loading');
    setErrorMessage('');
    setCreatedFiles(null);

    try {
      const nextStatus = await executeMigrateStatus(
        selectedTarget,
        buildConnection(),
        activeWorkspaceRoot,
      );
      setMigrationStatus(nextStatus);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  // 確認済みのマイグレーションアクションを実行する
  async function handleConfirmAction() {
    if (!pendingAction) {
      return;
    }

    const action = pendingAction;
    setPendingAction(null);

    if (action.kind === 'up') {
      await runAction(() => executeMigrateUp(action.config, activeWorkspaceRoot));
      return;
    }

    await runAction(() => executeMigrateDown(action.config, activeWorkspaceRoot));
  }

  return (
    <div className="p3-animate-in glass max-w-6xl p-6" data-testid="migrate-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">データベース</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">マイグレーション管理</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        同一のワークスペース対応UIからマイグレーションの作成、適用、ロールバック、検査、修復を行います。
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          マイグレーションを管理する前に有効なワークスペースルートを設定してください。
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.95fr_1.05fr]">
        <section className="space-y-6 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div>
            <label className="block text-sm font-medium text-slate-200/82">ターゲット</label>
            <select
              value={activeTargetKey}
              onChange={(event) => {
                setSelectedTargetKey(event.target.value);
                resetResult();
              }}
              className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(5,8,15,0.35)] px-3 py-2 text-white"
              data-testid="select-migrate-target"
            >
              {availableTargets.length === 0 ? (
                <option value="">マイグレーションターゲットが見つかりません</option>
              ) : (
                availableTargets.map((target) => (
                  <option key={encodeTarget(target)} value={encodeTarget(target)}>
                    {target.service_name} ({target.tier}/{target.language}) [{target.db_name}]
                  </option>
                ))
              )}
            </select>
          </div>

          <fieldset className="space-y-2">
            <legend className="text-sm font-medium text-slate-200/82">接続</legend>
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={connectionMode === 'local'}
                onChange={() => {
                  setConnectionMode('local');
                  setPendingAction(null);
                }}
                name="migrate-connection"
              />
              ローカル開発
            </label>
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={connectionMode === 'custom'}
                onChange={() => {
                  setConnectionMode('custom');
                  setPendingAction(null);
                }}
                name="migrate-connection"
              />
              カスタム接続文字列
            </label>
            {connectionMode === 'custom' && (
              <input
                value={customConnection}
                onChange={(event) => {
                  setCustomConnection(event.target.value);
                  setPendingAction(null);
                }}
                placeholder="postgres://user:pass@host:5432/db"
                className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                data-testid="input-custom-connection"
              />
            )}
          </fieldset>

          <div className="border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.20)] p-4">
            <p className="text-sm font-medium text-slate-200/82">マイグレーションを作成</p>
            <input
              value={migrationName}
              onChange={(event) => setMigrationName(event.target.value)}
              placeholder="add_new_column"
              className="mt-3 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
              data-testid="input-migration-name"
            />
            <button
              type="button"
              onClick={() => {
                void handleCreate();
              }}
              disabled={
                !selectedTarget ||
                !migrationName ||
                workspaceUnavailable ||
                status === 'loading' ||
                actionsLocked
              }
              className="mt-4 bg-cyan-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
              data-testid="btn-migrate-create"
            >
              作成
            </button>
          </div>

          <ActionBlock
            title="マイグレーションを適用"
            buttonLabel="マイグレーションアップを確認"
            buttonTestId="btn-migrate-up"
            disabled={!selectedTarget || workspaceUnavailable || status === 'loading' || actionsLocked}
            onClick={() => {
              if (!selectedTarget) {
                return;
              }
              setPendingAction({
                kind: 'up',
                config: {
                  target: selectedTarget,
                  range: buildUpRange(upRangeMode, upRangeValue),
                  connection: buildConnection(),
                } satisfies MigrateUpConfig,
              });
            }}
          >
            <UpRangeSelector
              name="migrate-up-range"
              mode={upRangeMode}
              value={upRangeValue}
              onModeChange={(mode) => {
                setUpRangeMode(mode);
                setPendingAction(null);
              }}
              onValueChange={(value) => {
                setUpRangeValue(value);
                setPendingAction(null);
              }}
            />
          </ActionBlock>

          <ActionBlock
            title="マイグレーションをロールバック"
            buttonLabel="マイグレーションダウンを確認"
            buttonTestId="btn-migrate-down"
            disabled={!selectedTarget || workspaceUnavailable || status === 'loading' || actionsLocked}
            onClick={() => {
              if (!selectedTarget) {
                return;
              }
              setPendingAction({
                kind: 'down',
                config: {
                  target: selectedTarget,
                  range: buildDownRange(downRangeMode, downRangeValue),
                  connection: buildConnection(),
                } satisfies MigrateDownConfig,
              });
            }}
          >
            <DownRangeSelector
              name="migrate-down-range"
              mode={downRangeMode}
              value={downRangeValue}
              onModeChange={(mode) => {
                setDownRangeMode(mode);
                setPendingAction(null);
              }}
              onValueChange={(value) => {
                setDownRangeValue(value);
                setPendingAction(null);
              }}
            />
          </ActionBlock>

          <ActionBlock
            title="マイグレーション状態を修復"
            buttonLabel="修復"
            buttonTestId="btn-migrate-repair"
            disabled={!selectedTarget || workspaceUnavailable || status === 'loading' || actionsLocked}
            onClick={() => {
              if (!selectedTarget) {
                return;
              }
              const operation: RepairOperation =
                repairMode === 'clearDirty'
                  ? 'ClearDirty'
                  : { ForceVersion: Number(repairVersion) };
              void runAction(() =>
                executeMigrateRepair(
                  selectedTarget,
                  operation,
                  buildConnection(),
                  activeWorkspaceRoot,
                ),
              );
            }}
          >
            <fieldset className="space-y-2">
              <label className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={repairMode === 'clearDirty'}
                  onChange={() => setRepairMode('clearDirty')}
                  name="repair-mode"
                />
                ダーティ状態をクリア
              </label>
              <label className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={repairMode === 'forceVersion'}
                  onChange={() => setRepairMode('forceVersion')}
                  name="repair-mode"
                />
                バージョンを強制設定
              </label>
            </fieldset>
            {repairMode === 'forceVersion' && (
              <input
                value={repairVersion}
                onChange={(event) => setRepairVersion(event.target.value)}
                className="mt-3 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                data-testid="input-repair-version"
              />
            )}
          </ActionBlock>

          <button
            type="button"
            onClick={() => {
              void handleStatus();
            }}
            disabled={!selectedTarget || workspaceUnavailable || status === 'loading' || actionsLocked}
            className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
            data-testid="btn-migrate-status"
          >
            ステータスを読み込み
          </button>

          {pendingAction && selectedTarget && (
            <div
              className="border border-cyan-400/20 bg-cyan-400/10 p-4"
              data-testid="migrate-confirmation"
            >
              <p className="text-sm font-medium text-cyan-100">マイグレーションアクションの確認</p>
              <div className="mt-3 space-y-2 text-sm text-cyan-50/90">
                <p>サービス: {selectedTarget.service_name}</p>
                <p>データベース: {selectedTarget.db_name}</p>
                <p>アクション: {pendingAction.kind === 'up' ? 'マイグレーションアップ' : 'マイグレーションダウン'}</p>
                <p>範囲: {describeRange(pendingAction.config.range)}</p>
                <p>接続: {describeConnection(pendingAction.config.connection)}</p>
              </div>
              <div className="mt-4 flex flex-wrap gap-3">
                <button
                  type="button"
                  onClick={() => setPendingAction(null)}
                  className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)]"
                >
                  戻る
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void handleConfirmAction();
                  }}
                  className="bg-cyan-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500"
                  data-testid="btn-confirm-migrate"
                >
                  確定
                </button>
              </div>
            </div>
          )}

          {status === 'error' && (
            <p className="text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">結果</h2>

          {selectedTarget && (
            <div className="mt-4 border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.20)] p-4 text-sm text-slate-200/82">
              <p>サービス: {selectedTarget.service_name}</p>
              <p>ティア: {selectedTarget.tier}</p>
              <p>言語: {selectedTarget.language}</p>
              <p>データベース: {selectedTarget.db_name}</p>
              <p>
                マイグレーションディレクトリ:{' '}
                {toDisplayPath(activeWorkspaceRoot, selectedTarget.migrations_dir)}
              </p>
            </div>
          )}

          {createdFiles && (
            <div className="mt-5 border border-cyan-400/20 bg-cyan-400/10 p-4">
              <p className="text-sm font-medium text-cyan-100">作成されたファイル</p>
              <p className="mt-3 text-sm text-cyan-50/90">
                {toDisplayPath(activeWorkspaceRoot, createdFiles[0])}
              </p>
              <p className="text-sm text-cyan-50/90">
                {toDisplayPath(activeWorkspaceRoot, createdFiles[1])}
              </p>
            </div>
          )}

          <div className="mt-5 space-y-2">
            {migrationStatus.length === 0 ? (
              <p className="text-sm text-slate-200/55">
                適用済みおよび保留中のマイグレーションを確認するにはステータスを読み込みます。
              </p>
            ) : (
              migrationStatus.map((item) => (
                <div
                  key={`${item.number}-${item.description}`}
                  className="border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-3 text-sm text-slate-100"
                >
                  <p>
                    {String(item.number).padStart(3, '0')} {item.description}
                  </p>
                  <p className="mt-1 text-slate-300/75">
                    {item.applied ? `適用日時: ${item.applied_at ?? '不明'}` : '保留中'}
                  </p>
                </div>
              ))
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

// ターゲットを一意のキーにエンコードする
function encodeTarget(target: MigrateTarget | undefined) {
  return target ? `${target.service_name}:${target.migrations_dir}` : '';
}

// アクションブロックコンポーネント（タイトル、子要素、実行ボタンを含む共通レイアウト）
function ActionBlock({
  children,
  disabled,
  onClick,
  title,
  buttonLabel,
  buttonTestId,
}: {
  children: ReactNode;
  disabled: boolean;
  onClick: () => void;
  title: string;
  buttonLabel: string;
  buttonTestId: string;
}) {
  return (
    <div className="border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.20)] p-4">
      <p className="text-sm font-medium text-slate-200/82">{title}</p>
      <div className="mt-3">{children}</div>
      <button
        type="button"
        onClick={onClick}
        disabled={disabled}
        className="mt-4 border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
        data-testid={buttonTestId}
      >
        {buttonLabel}
      </button>
    </div>
  );
}

// マイグレーションアップの範囲選択コンポーネント
function UpRangeSelector({
  name,
  mode,
  value,
  onModeChange,
  onValueChange,
}: {
  name: string;
  mode: UpRangeMode;
  value: string;
  onModeChange: (mode: UpRangeMode) => void;
  onValueChange: (value: string) => void;
}) {
  return (
    <>
      <label className="flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'all'}
          onChange={() => onModeChange('all')}
          name={name}
        />
        保留中の全マイグレーション
      </label>
      <label className="mt-2 flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'upTo'}
          onChange={() => onModeChange('upTo')}
          name={name}
        />
        指定バージョンまで
      </label>
      {mode === 'upTo' && (
        <input
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          className="mt-3 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
        />
      )}
    </>
  );
}

// マイグレーションダウンの範囲選択コンポーネント
function DownRangeSelector({
  name,
  mode,
  value,
  onModeChange,
  onValueChange,
}: {
  name: string;
  mode: DownRangeMode;
  value: string;
  onModeChange: (mode: DownRangeMode) => void;
  onValueChange: (value: string) => void;
}) {
  return (
    <>
      <label className="flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'previous'}
          onChange={() => onModeChange('previous')}
          name={name}
        />
        前回のマイグレーションをロールバック
      </label>
      <label className="mt-2 flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'upTo'}
          onChange={() => onModeChange('upTo')}
          name={name}
        />
        指定バージョンまでロールバック
      </label>
      {mode === 'upTo' && (
        <input
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          className="mt-3 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
        />
      )}
      <label className="mt-2 flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'all'}
          onChange={() => onModeChange('all')}
          name={name}
        />
        全マイグレーションをロールバック
      </label>
    </>
  );
}
