import { useEffect, useMemo, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { toDisplayPath } from '../lib/paths';
import {
  executeTemplateMigration,
  executeTemplateMigrationRollback,
  listTemplateMigrationBackups,
  previewTemplateMigration,
  scanTemplateMigrationTargets,
  type TemplateConflictHunk,
  type TemplateFileChange,
  type TemplateMergeResult,
  type TemplateMigrationPlan,
  type TemplateMigrationTarget,
} from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

type ConflictResolution = 'template' | 'user' | 'skip';

export default function TemplateMigratePage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [targets, setTargets] = useState<TemplateMigrationTarget[]>([]);
  const [selectedTargetPath, setSelectedTargetPath] = useState('');
  const [plan, setPlan] = useState<TemplateMigrationPlan | null>(null);
  const [backups, setBackups] = useState<string[]>([]);
  const [selectedBackup, setSelectedBackup] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  async function refreshTargets(preferredPath?: string) {
    const nextTargets = await scanTemplateMigrationTargets(activeWorkspaceRoot);
    setTargets(nextTargets);
    setSelectedTargetPath((current) => {
      const desiredPath = preferredPath || current;
      if (desiredPath && nextTargets.some((target) => target.path === desiredPath)) {
        return desiredPath;
      }
      return nextTargets[0]?.path || '';
    });
    return nextTargets;
  }

  async function refreshBackups(projectPath: string, preferredBackup?: string) {
    const nextBackups = await listTemplateMigrationBackups(projectPath);
    setBackups(nextBackups);
    setSelectedBackup((current) => {
      const desiredBackup = preferredBackup || current;
      if (desiredBackup && nextBackups.includes(desiredBackup)) {
        return desiredBackup;
      }
      return nextBackups[0] || '';
    });
    return nextBackups;
  }

  useEffect(() => {
    let cancelled = false;

    if (!workspace.ready || !workspace.workspaceRoot) {
      return;
    }

    scanTemplateMigrationTargets(activeWorkspaceRoot)
      .then((nextTargets) => {
        if (cancelled) {
          return;
        }

        setTargets(nextTargets);
        setSelectedTargetPath((current) => current || nextTargets[0]?.path || '');
      })
      .catch(() => {
        if (!cancelled) {
          setTargets([]);
          setSelectedTargetPath('');
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceRoot, workspace.ready, workspace.workspaceRoot]);

  useEffect(() => {
    let cancelled = false;
    if (!selectedTargetPath) {
      setBackups([]);
      setSelectedBackup('');
      return;
    }

    listTemplateMigrationBackups(selectedTargetPath)
      .then((nextBackups) => {
        if (cancelled) {
          return;
        }

        setBackups(nextBackups);
        setSelectedBackup((current) => current || nextBackups[0] || '');
      })
      .catch(() => {
        if (!cancelled) {
          setBackups([]);
          setSelectedBackup('');
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedTargetPath]);

  const selectedTarget =
    targets.find((target) => target.path === selectedTargetPath) ?? targets[0] ?? null;
  const unresolvedConflicts = useMemo(
    () =>
      plan?.changes.filter((change) => getConflictHunks(change.merge_result).length > 0).length ?? 0,
    [plan],
  );

  async function handlePreview() {
    if (!selectedTarget) {
      return;
    }

    setStatus('loading');
    setErrorMessage('');
    setSuccessMessage('');

    try {
      const nextPlan = await previewTemplateMigration(selectedTarget);
      setPlan(nextPlan);
      setStatus('success');
      setSuccessMessage(
        nextPlan.changes.length === 0
          ? 'No template changes were detected.'
          : 'Dry-run completed. Review the plan before applying it.',
      );
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  async function handleExecute() {
    if (!plan) {
      return;
    }

    setStatus('loading');
    setErrorMessage('');
    setSuccessMessage('');

    try {
      await executeTemplateMigration(plan);
      setPlan(null);

      let refreshNote = '';
      try {
        await refreshTargets(plan.target.path);
        await refreshBackups(plan.target.path);
      } catch (refreshError) {
        refreshNote = ` Migration applied, but the page could not refresh automatically: ${String(refreshError)}`;
      }

      setStatus('success');
      setSuccessMessage(`Template migration completed successfully.${refreshNote}`);
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  async function handleRollback() {
    if (!selectedTarget || !selectedBackup) {
      return;
    }

    setStatus('loading');
    setErrorMessage('');
    setSuccessMessage('');

    try {
      await executeTemplateMigrationRollback(selectedTarget.path, selectedBackup);
      setPlan(null);

      let refreshNote = '';
      try {
        await refreshTargets(selectedTarget.path);
        await refreshBackups(selectedTarget.path);
      } catch (refreshError) {
        refreshNote = ` Workspace state was restored, but the page could not refresh automatically: ${String(refreshError)}`;
      }

      setStatus('success');
      setSuccessMessage(`Rolled back to backup ${selectedBackup}.${refreshNote}`);
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  function applyResolution(change: TemplateFileChange, resolution: ConflictResolution): TemplateFileChange {
    const conflictHunks = getConflictHunks(change.merge_result);
    if (conflictHunks.length === 0) {
      return change;
    }

    const [firstHunk] = conflictHunks;
    const merge_result: TemplateMergeResult =
      resolution === 'template'
        ? { Clean: firstHunk.theirs }
        : resolution === 'user'
          ? { Clean: firstHunk.ours }
          : 'NoChange';

    return {
      ...change,
      merge_result,
    };
  }

  function handleConflictResolution(changePath: string, resolution: ConflictResolution) {
    setPlan((current) => {
      if (!current) {
        return current;
      }

      return {
        ...current,
        changes: current.changes.map((change) =>
          change.path === changePath ? applyResolution(change, resolution) : change,
        ),
      };
    });
  }

  return (
    <div className="glass max-w-6xl p-6" data-testid="template-migrate-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Template</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Template migration</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Preview scaffold drift with a dry-run, resolve merge conflicts, and keep a rollback path
        for generated modules.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before running template migration.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.95fr_1.05fr]">
        <section className="space-y-6 rounded-2xl border border-white/10 bg-white/5 p-5">
          <div>
            <label className="block text-sm font-medium text-slate-200/82">Target</label>
            <select
              value={selectedTarget?.path ?? ''}
              onChange={(event) => {
                setSelectedTargetPath(event.target.value);
                setPlan(null);
                setSuccessMessage('');
                setErrorMessage('');
              }}
              className="mt-2 w-full rounded-xl border border-white/15 bg-slate-950/35 px-3 py-2 text-white"
              data-testid="select-template-target"
            >
              {targets.length === 0 ? (
                <option value="">No template-managed targets found</option>
              ) : (
                targets.map((target) => (
                  <option key={target.path} value={target.path}>
                    {toDisplayPath(activeWorkspaceRoot, target.path)} (v{target.manifest.spec.template.version}{' '}
                    → v{target.available_version})
                  </option>
                ))
              )}
            </select>
          </div>

          {selectedTarget && (
            <div className="rounded-2xl border border-white/10 bg-slate-950/20 p-4 text-sm text-slate-200/82">
              <p>Name: {selectedTarget.manifest.metadata.name}</p>
              <p>Type: {selectedTarget.manifest.spec.template.type}</p>
              <p>Language: {selectedTarget.manifest.spec.template.language}</p>
              <p>Version: v{selectedTarget.manifest.spec.template.version}</p>
              <p>Latest: v{selectedTarget.available_version}</p>
              <p>Path: {toDisplayPath(activeWorkspaceRoot, selectedTarget.path)}</p>
            </div>
          )}

          <button
            type="button"
            onClick={() => {
              void handlePreview();
            }}
            disabled={!selectedTarget || workspaceUnavailable || status === 'loading' || actionsLocked}
            className="rounded-xl bg-emerald-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
            data-testid="btn-template-preview"
          >
            Preview migration
          </button>

          <div className="rounded-2xl border border-white/10 bg-slate-950/20 p-4">
            <p className="text-sm font-medium text-slate-200/82">Rollback</p>
            <p className="mt-2 text-sm leading-6 text-slate-300/72">
              Restore the entire project tree from a captured backup if a migration outcome is not
              acceptable.
            </p>

            <select
              value={selectedBackup}
              onChange={(event) => setSelectedBackup(event.target.value)}
              className="mt-4 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
              data-testid="select-template-backup"
            >
              {backups.length === 0 ? (
                <option value="">No backups available</option>
              ) : (
                backups.map((backup) => (
                  <option key={backup} value={backup}>
                    {backup}
                  </option>
                ))
              )}
            </select>

            <button
              type="button"
              onClick={() => {
                void handleRollback();
              }}
              disabled={
                !selectedTarget ||
                !selectedBackup ||
                workspaceUnavailable ||
                status === 'loading' ||
                actionsLocked
              }
              className="mt-4 rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
              data-testid="btn-template-rollback"
            >
              Roll back
            </button>
          </div>

          {status === 'error' && (
            <p className="text-sm text-rose-300" data-testid="template-error-message">
              {errorMessage}
            </p>
          )}
          {status === 'success' && successMessage && (
            <p className="text-sm text-emerald-300" data-testid="template-success-message">
              {successMessage}
            </p>
          )}
        </section>

        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <div className="flex items-center justify-between gap-4">
            <div>
              <h2 className="text-lg font-semibold text-white">Plan</h2>
              <p className="mt-1 text-sm text-slate-300/72">
                Dry-run results include add, modify, delete, skip, and conflict outcomes.
              </p>
            </div>
            {plan && (
              <button
                type="button"
                onClick={() => {
                  void handleExecute();
                }}
                disabled={
                  unresolvedConflicts > 0 ||
                  plan.changes.length === 0 ||
                  workspaceUnavailable ||
                  status === 'loading' ||
                  actionsLocked
                }
                className="rounded-xl bg-emerald-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
                data-testid="btn-template-apply"
              >
                Apply migration
              </button>
            )}
          </div>

          {plan ? (
            <>
              <div className="mt-4 grid gap-3 sm:grid-cols-3">
                <Metric label="Changes" value={String(plan.changes.length)} />
                <Metric label="Conflicts" value={String(unresolvedConflicts)} />
                <Metric
                  label="Version"
                  value={`v${plan.target.manifest.spec.template.version} → v${plan.target.available_version}`}
                />
              </div>

              <div className="mt-5 space-y-3" data-testid="template-plan-list">
                {plan.changes.length === 0 ? (
                  <p className="text-sm text-slate-200/55">No migration changes were detected.</p>
                ) : (
                  plan.changes.map((change) => {
                    const conflictHunks = getConflictHunks(change.merge_result);
                    const cleanContent = getCleanContent(change.merge_result);
                    return (
                      <article
                        key={change.path}
                        className="rounded-2xl border border-white/10 bg-slate-950/20 p-4"
                        data-testid={`template-change-${change.path}`}
                      >
                        <div className="flex flex-wrap items-center gap-3">
                          <span className="rounded-full bg-emerald-400/12 px-3 py-1 text-xs uppercase tracking-[0.22em] text-emerald-100">
                            {change.change_type}
                          </span>
                          <span className="rounded-full border border-white/10 px-3 py-1 text-xs uppercase tracking-[0.22em] text-slate-200/72">
                            {change.merge_strategy}
                          </span>
                          <code className="text-sm text-white">{toDisplayPath(activeWorkspaceRoot, change.path)}</code>
                        </div>

                        {change.change_type === 'Skipped' && (
                          <p className="mt-3 text-sm text-slate-300/70">
                            This path matches an ignore rule and will be preserved.
                          </p>
                        )}

                        {cleanContent !== null && change.change_type !== 'Deleted' && (
                          <pre className="mt-3 overflow-x-auto rounded-xl border border-white/8 bg-slate-950/35 p-3 text-xs text-slate-200/82">
                            {previewText(cleanContent)}
                          </pre>
                        )}

                        {conflictHunks.length > 0 && (
                          <div className="mt-4 rounded-2xl border border-rose-400/20 bg-rose-400/10 p-4">
                            <p className="text-sm font-medium text-rose-100">Conflict resolution</p>
                            {conflictHunks.map((hunk, index) => (
                              <ConflictPreview key={index} index={index} hunk={hunk} />
                            ))}
                            <div className="mt-4 flex flex-wrap gap-3">
                              <button
                                type="button"
                                onClick={() => handleConflictResolution(change.path, 'template')}
                                className="rounded-xl bg-rose-500/80 px-3 py-2 text-sm font-medium text-white transition hover:bg-rose-500"
                              >
                                Use template
                              </button>
                              <button
                                type="button"
                                onClick={() => handleConflictResolution(change.path, 'user')}
                                className="rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-sm font-medium text-white/85 transition hover:bg-white/10"
                              >
                                Keep user changes
                              </button>
                              <button
                                type="button"
                                onClick={() => handleConflictResolution(change.path, 'skip')}
                                className="rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-sm font-medium text-white/85 transition hover:bg-white/10"
                              >
                                Skip file
                              </button>
                            </div>
                          </div>
                        )}
                      </article>
                    );
                  })
                )}
              </div>
            </>
          ) : (
            <p className="mt-5 text-sm text-slate-200/55">
              Run a dry-run preview to build a migration plan.
            </p>
          )}
        </section>
      </div>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-white/10 bg-slate-950/20 p-4">
      <p className="text-xs uppercase tracking-[0.24em] text-slate-200/55">{label}</p>
      <p className="mt-3 text-lg font-semibold text-white">{value}</p>
    </div>
  );
}

function ConflictPreview({
  index,
  hunk,
}: {
  index: number;
  hunk: TemplateConflictHunk;
}) {
  const templatePreview = hunk.theirs_preview ?? hunk.theirs;
  const userPreview = hunk.ours_preview ?? hunk.ours;
  return (
    <div className="mt-4 grid gap-3 lg:grid-cols-2">
      <div className="rounded-xl border border-white/8 bg-slate-950/35 p-3">
        <p className="text-xs uppercase tracking-[0.22em] text-rose-100/72">Template #{index + 1}</p>
        <pre className="mt-2 overflow-x-auto text-xs text-rose-50/90">{previewText(templatePreview)}</pre>
      </div>
      <div className="rounded-xl border border-white/8 bg-slate-950/35 p-3">
        <p className="text-xs uppercase tracking-[0.22em] text-slate-200/72">User #{index + 1}</p>
        <pre className="mt-2 overflow-x-auto text-xs text-slate-100/90">{previewText(userPreview)}</pre>
      </div>
    </div>
  );
}

function getConflictHunks(result: TemplateMergeResult): TemplateConflictHunk[] {
  if (typeof result === 'string') {
    return [];
  }
  return 'Conflict' in result ? result.Conflict : [];
}

function getCleanContent(result: TemplateMergeResult): string | null {
  if (typeof result === 'string') {
    return null;
  }
  return 'Clean' in result ? result.Clean : null;
}

function previewText(content: string): string {
  return content.split('\n').slice(0, 12).join('\n');
}
