import { useEffect, useState } from 'react';
import {
  executeDevDown,
  executeDevLogs,
  executeDevStatus,
  executeDevUp,
  scanDevTargets,
  type AuthMode,
  type CleanupLevel,
  type DevTarget,
} from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

export default function DevPage() {
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;

  const [targets, setTargets] = useState<DevTarget[]>([]);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [authMode, setAuthMode] = useState<AuthMode>('Skip');
  const [cleanup, setCleanup] = useState<CleanupLevel>('ContainersOnly');
  const [logService, setLogService] = useState<string>('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [output, setOutput] = useState('');

  useEffect(() => {
    let cancelled = false;

    if (workspaceUnavailable) {
      setTargets([]);
      return;
    }

    scanDevTargets(activeWorkspaceRoot)
      .then((nextTargets) => {
        if (!cancelled) {
          setTargets(nextTargets);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setTargets([]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceRoot, workspaceUnavailable]);

  function toggleTarget(path: string) {
    setSelectedPaths((current) =>
      current.includes(path)
        ? current.filter((value) => value !== path)
        : [...current, path],
    );
  }

  async function runAction(action: () => Promise<void | string>) {
    setStatus('loading');
    setErrorMessage('');

    try {
      const nextOutput = await action();
      if (typeof nextOutput === 'string') {
        setOutput(nextOutput);
      }
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-6xl p-6" data-testid="dev-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Operations</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Manage local development</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Start and stop local dependencies, inspect state, and collect logs from the selected
        workspace.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before managing local development services.
        </p>
      )}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <div>
            <p className="text-sm font-medium text-slate-200/82">Services</p>
            <div className="mt-3 max-h-72 space-y-2 overflow-auto pr-1">
              {targets.length === 0 ? (
                <p className="text-sm text-slate-200/55">No runnable services were found.</p>
              ) : (
                targets.map(([label, path]) => (
                  <label
                    key={path}
                    className="flex items-center gap-3 rounded-xl border border-white/8 bg-slate-950/20 px-3 py-2 text-sm text-slate-100"
                  >
                    <input
                      type="checkbox"
                      checked={selectedPaths.includes(path)}
                      onChange={() => toggleTarget(path)}
                    />
                    <span>{label}</span>
                    <span className="text-slate-400/80">
                      {toDisplayPath(activeWorkspaceRoot, path)}
                    </span>
                  </label>
                ))
              )}
            </div>
          </div>

          <fieldset className="mt-5 space-y-2">
            <legend className="text-sm font-medium text-slate-200/82">Auth mode</legend>
            {(['Skip', 'Keycloak'] as AuthMode[]).map((value) => (
              <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={authMode === value}
                  onChange={() => setAuthMode(value)}
                  name="dev-auth-mode"
                />
                {value}
              </label>
            ))}
          </fieldset>

          <fieldset className="mt-5 space-y-2">
            <legend className="text-sm font-medium text-slate-200/82">Cleanup on stop</legend>
            {(['ContainersOnly', 'ContainersAndVolumes'] as CleanupLevel[]).map((value) => (
              <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={cleanup === value}
                  onChange={() => setCleanup(value)}
                  name="dev-cleanup"
                />
                {value === 'ContainersOnly' ? 'Containers only' : 'Containers and volumes'}
              </label>
            ))}
          </fieldset>

          <div className="mt-5">
            <label className="block text-sm font-medium text-slate-200/82">
              Service filter for logs
            </label>
            <input
              value={logService}
              onChange={(event) => setLogService(event.target.value)}
              placeholder="leave empty for all services"
              className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
              data-testid="input-log-service"
            />
          </div>

          <div className="mt-6 flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => {
                void runAction(() =>
                  executeDevUp(
                    { services: selectedPaths, auth_mode: authMode },
                    activeWorkspaceRoot,
                  ),
                );
              }}
              disabled={workspaceUnavailable || selectedPaths.length === 0 || status === 'loading'}
              className="rounded-xl bg-emerald-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
              data-testid="btn-dev-up"
            >
              Start
            </button>
            <button
              type="button"
              onClick={() => {
                void runAction(() =>
                  executeDevDown({ cleanup }, activeWorkspaceRoot),
                );
              }}
              disabled={workspaceUnavailable || status === 'loading'}
              className="rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
              data-testid="btn-dev-down"
            >
              Stop
            </button>
            <button
              type="button"
              onClick={() => {
                void runAction(() => executeDevStatus(activeWorkspaceRoot));
              }}
              disabled={workspaceUnavailable || status === 'loading'}
              className="rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
              data-testid="btn-dev-status"
            >
              Status
            </button>
            <button
              type="button"
              onClick={() => {
                void runAction(() => executeDevLogs(logService || null, activeWorkspaceRoot));
              }}
              disabled={workspaceUnavailable || status === 'loading'}
              className="rounded-xl border border-white/15 bg-white/6 px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
              data-testid="btn-dev-logs"
            >
              Logs
            </button>
          </div>

          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <h2 className="text-lg font-semibold text-white">Output</h2>
          <div className="mt-4 rounded-2xl border border-white/10 bg-slate-950/35 p-4">
            <pre className="min-h-72 overflow-auto whitespace-pre-wrap text-xs text-slate-100">
              {output || 'Run status or logs to inspect the local development environment.'}
            </pre>
          </div>
        </section>
      </div>
    </div>
  );
}
