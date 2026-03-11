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

  function buildConnection(): DbConnection {
    return connectionMode === 'local' ? 'LocalDev' : { Custom: customConnection };
  }

  function buildUpRange(mode: UpRangeMode, value: string): MigrateRange {
    return mode === 'all' ? 'All' : { UpTo: Number(value) };
  }

  function buildDownRange(mode: DownRangeMode, value: string): MigrateRange {
    if (mode === 'previous') {
      return { Steps: 1 };
    }
    return mode === 'all' ? 'All' : { UpTo: Number(value) };
  }

  function resetResult() {
    setCreatedFiles(null);
    setMigrationStatus([]);
    setPendingAction(null);
  }

  function describeRange(range: MigrateRange): string {
    if (range === 'All') {
      return 'all migrations';
    }
    if ('Steps' in range) {
      return `roll back ${range.Steps} migration`;
    }
    return `up to version ${range.UpTo}`;
  }

  function describeConnection(connection: DbConnection): string {
    return connection === 'LocalDev' ? 'local development database' : connection.Custom;
  }

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
    <div className="glass max-w-6xl p-6" data-testid="migrate-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Database</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Manage migrations</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Create, apply, roll back, inspect, and repair migrations from the same workspace-aware UI.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before managing migrations.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.95fr_1.05fr]">
        <section className="space-y-6 rounded-2xl border border-white/10 bg-white/5 p-5">
          <div>
            <label className="block text-sm font-medium text-slate-200/82">Target</label>
            <select
              value={activeTargetKey}
              onChange={(event) => {
                setSelectedTargetKey(event.target.value);
                resetResult();
              }}
              className="mt-2 w-full rounded-xl border border-white/15 bg-slate-950/35 px-3 py-2 text-white"
              data-testid="select-migrate-target"
            >
              {availableTargets.length === 0 ? (
                <option value="">No migration targets found</option>
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
            <legend className="text-sm font-medium text-slate-200/82">Connection</legend>
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
              Local development
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
              Custom connection string
            </label>
            {connectionMode === 'custom' && (
              <input
                value={customConnection}
                onChange={(event) => {
                  setCustomConnection(event.target.value);
                  setPendingAction(null);
                }}
                placeholder="postgres://user:pass@host:5432/db"
                className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                data-testid="input-custom-connection"
              />
            )}
          </fieldset>

          <div className="rounded-2xl border border-white/10 bg-slate-950/20 p-4">
            <p className="text-sm font-medium text-slate-200/82">Create migration</p>
            <input
              value={migrationName}
              onChange={(event) => setMigrationName(event.target.value)}
              placeholder="add_new_column"
              className="mt-3 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
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
              className="mt-4 rounded-xl bg-emerald-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
              data-testid="btn-migrate-create"
            >
              Create
            </button>
          </div>

          <ActionBlock
            title="Apply migrations"
            buttonLabel="Review migrate up"
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
            title="Roll back migrations"
            buttonLabel="Review migrate down"
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
            title="Repair migration state"
            buttonLabel="Repair"
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
                Clear dirty state
              </label>
              <label className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={repairMode === 'forceVersion'}
                  onChange={() => setRepairMode('forceVersion')}
                  name="repair-mode"
                />
                Force version
              </label>
            </fieldset>
            {repairMode === 'forceVersion' && (
              <input
                value={repairVersion}
                onChange={(event) => setRepairVersion(event.target.value)}
                className="mt-3 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
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
            className="rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
            data-testid="btn-migrate-status"
          >
            Load status
          </button>

          {pendingAction && selectedTarget && (
            <div
              className="rounded-2xl border border-emerald-400/20 bg-emerald-400/10 p-4"
              data-testid="migrate-confirmation"
            >
              <p className="text-sm font-medium text-emerald-100">Confirm migration action</p>
              <div className="mt-3 space-y-2 text-sm text-emerald-50/90">
                <p>Service: {selectedTarget.service_name}</p>
                <p>Database: {selectedTarget.db_name}</p>
                <p>Action: {pendingAction.kind === 'up' ? 'migrate up' : 'migrate down'}</p>
                <p>Range: {describeRange(pendingAction.config.range)}</p>
                <p>Connection: {describeConnection(pendingAction.config.connection)}</p>
              </div>
              <div className="mt-4 flex flex-wrap gap-3">
                <button
                  type="button"
                  onClick={() => setPendingAction(null)}
                  className="rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10"
                >
                  Back
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void handleConfirmAction();
                  }}
                  className="rounded-xl bg-emerald-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500"
                  data-testid="btn-confirm-migrate"
                >
                  Confirm
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

        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <h2 className="text-lg font-semibold text-white">Result</h2>

          {selectedTarget && (
            <div className="mt-4 rounded-2xl border border-white/10 bg-slate-950/20 p-4 text-sm text-slate-200/82">
              <p>Service: {selectedTarget.service_name}</p>
              <p>Tier: {selectedTarget.tier}</p>
              <p>Language: {selectedTarget.language}</p>
              <p>Database: {selectedTarget.db_name}</p>
              <p>
                Migrations dir:{' '}
                {toDisplayPath(activeWorkspaceRoot, selectedTarget.migrations_dir)}
              </p>
            </div>
          )}

          {createdFiles && (
            <div className="mt-5 rounded-2xl border border-emerald-400/20 bg-emerald-400/10 p-4">
              <p className="text-sm font-medium text-emerald-100">Created files</p>
              <p className="mt-3 text-sm text-emerald-50/90">
                {toDisplayPath(activeWorkspaceRoot, createdFiles[0])}
              </p>
              <p className="text-sm text-emerald-50/90">
                {toDisplayPath(activeWorkspaceRoot, createdFiles[1])}
              </p>
            </div>
          )}

          <div className="mt-5 space-y-2">
            {migrationStatus.length === 0 ? (
              <p className="text-sm text-slate-200/55">
                Load status to inspect applied and pending migrations.
              </p>
            ) : (
              migrationStatus.map((item) => (
                <div
                  key={`${item.number}-${item.description}`}
                  className="rounded-xl border border-white/8 bg-slate-950/20 px-3 py-3 text-sm text-slate-100"
                >
                  <p>
                    {String(item.number).padStart(3, '0')} {item.description}
                  </p>
                  <p className="mt-1 text-slate-300/75">
                    {item.applied ? `Applied at ${item.applied_at ?? 'unknown time'}` : 'Pending'}
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

function encodeTarget(target: MigrateTarget | undefined) {
  return target ? `${target.service_name}:${target.migrations_dir}` : '';
}

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
    <div className="rounded-2xl border border-white/10 bg-slate-950/20 p-4">
      <p className="text-sm font-medium text-slate-200/82">{title}</p>
      <div className="mt-3">{children}</div>
      <button
        type="button"
        onClick={onClick}
        disabled={disabled}
        className="mt-4 rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
        data-testid={buttonTestId}
      >
        {buttonLabel}
      </button>
    </div>
  );
}

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
        All pending migrations
      </label>
      <label className="mt-2 flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'upTo'}
          onChange={() => onModeChange('upTo')}
          name={name}
        />
        Up to version
      </label>
      {mode === 'upTo' && (
        <input
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          className="mt-3 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
        />
      )}
    </>
  );
}

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
        Roll back the previous migration
      </label>
      <label className="mt-2 flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'upTo'}
          onChange={() => onModeChange('upTo')}
          name={name}
        />
        Roll back to version
      </label>
      {mode === 'upTo' && (
        <input
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
          className="mt-3 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
        />
      )}
      <label className="mt-2 flex items-center gap-3 text-sm text-slate-200/82">
        <input
          type="radio"
          checked={mode === 'all'}
          onChange={() => onModeChange('all')}
          name={name}
        />
        Roll back all migrations
      </label>
    </>
  );
}
