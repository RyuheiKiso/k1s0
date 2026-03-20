import { useEffect, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import {
  executeDevDown,
  executeDevLogs,
  executeDevStatus,
  executeDevUp,
  previewDevUp,
  scanDevTargets,
  type AuthMode,
  type CleanupLevel,
  type DevUpPreview,
  type DevTarget,
} from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

export default function DevPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [targets, setTargets] = useState<DevTarget[]>([]);
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [authMode, setAuthMode] = useState<AuthMode>('Skip');
  const [cleanup, setCleanup] = useState<CleanupLevel>('ContainersOnly');
  const [logService, setLogService] = useState<string>('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [output, setOutput] = useState('');
  const [preview, setPreview] = useState<DevUpPreview | null>(null);

  useEffect(() => {
    let cancelled = false;

    if (!workspace.ready || !workspace.workspaceRoot) {
      return;
    }

    scanDevTargets(activeWorkspaceRoot)
      .then((nextTargets) => {
        if (!cancelled) {
          setTargets(nextTargets);
          setSelectedPaths((current) =>
            current.filter((path) => nextTargets.some(([, nextPath]) => nextPath === path)),
          );
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
  }, [activeWorkspaceRoot, workspace.ready, workspace.workspaceRoot]);

  const availableTargets = workspace.ready && workspace.workspaceRoot ? targets : [];
  const selectedTargetPaths = selectedPaths.filter((path) =>
    availableTargets.some(([, targetPath]) => targetPath === path),
  );
  const selectedServices = availableTargets.filter(([, path]) => selectedTargetPaths.includes(path));

  function toggleTarget(path: string) {
    setPreview(null);
    setSelectedPaths((current) =>
      current.includes(path)
        ? current.filter((value) => value !== path)
        : [...current, path],
    );
  }

  function getPreviewPorts() {
    if (!preview) {
      return [];
    }

    const ports: Array<{ name: string; value: string }> = [];

    if (preview.dependencies.databases.length > 0) {
      ports.push({ name: 'PostgreSQL', value: `localhost:${preview.ports.postgres}` });
      ports.push({ name: 'pgAdmin', value: preview.additional_services.find((service) => service.name === 'pgAdmin')?.url ?? `http://localhost:${preview.ports.pgadmin}` });
    }

    if (preview.dependencies.has_kafka) {
      ports.push({ name: 'Kafka', value: `localhost:${preview.ports.kafka}` });
      ports.push({ name: 'Kafka UI', value: preview.additional_services.find((service) => service.name === 'Kafka UI')?.url ?? `http://localhost:${preview.ports.kafka_ui}` });
    }

    if (preview.dependencies.has_redis) {
      ports.push({ name: 'Redis', value: `localhost:${preview.ports.redis}` });
    }

    if (preview.dependencies.has_redis_session) {
      ports.push({ name: 'Redis session', value: `localhost:${preview.ports.redis_session}` });
    }

    if (authMode === 'Keycloak') {
      ports.push({ name: 'Keycloak', value: `http://localhost:${preview.ports.keycloak}` });
    }

    return ports;
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

  async function handleReviewStart() {
    setStatus('loading');
    setErrorMessage('');

    try {
      const nextPreview = await previewDevUp(
        { services: selectedTargetPaths, auth_mode: authMode },
        activeWorkspaceRoot,
      );
      setPreview(nextPreview);
      setStatus('idle');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  async function handleConfirmStart() {
    setPreview(null);
    await runAction(() =>
      executeDevUp({ services: selectedTargetPaths, auth_mode: authMode }, activeWorkspaceRoot),
    );
  }

  return (
    <div className="glass max-w-6xl p-6 p3-animate-in" data-testid="dev-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">Operations</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Manage local development</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Start and stop local dependencies, inspect state, and collect logs from the selected
        workspace.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          Configure a valid workspace root before managing local development services.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div>
            <p className="text-sm font-medium text-slate-200/82">Services</p>
            <div className="mt-3 max-h-72 space-y-2 overflow-auto pr-1">
              {availableTargets.length === 0 ? (
                <p className="text-sm text-slate-200/55">No runnable services were found.</p>
              ) : (
                availableTargets.map(([label, path]) => (
                  <label
                    key={path}
                    className="flex items-center gap-3 border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-2 text-sm text-slate-100"
                  >
                    <input
                      type="checkbox"
                      checked={selectedTargetPaths.includes(path)}
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
                  onChange={() => {
                    setAuthMode(value);
                    setPreview(null);
                  }}
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
              className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
              data-testid="input-log-service"
            />
          </div>

          <div className="mt-6 flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => {
                void handleReviewStart();
              }}
              disabled={
                workspaceUnavailable ||
                selectedTargetPaths.length === 0 ||
                status === 'loading' ||
                actionsLocked
              }
              className="bg-cyan-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
              data-testid="btn-dev-up"
            >
              {status === 'loading' ? 'Preparing...' : 'Review start'}
            </button>
            <button
              type="button"
              onClick={() => {
                void runAction(() => executeDevDown({ cleanup }, activeWorkspaceRoot));
              }}
              disabled={workspaceUnavailable || status === 'loading' || actionsLocked}
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-dev-down"
            >
              Stop
            </button>
            <button
              type="button"
              onClick={() => {
                void runAction(() => executeDevStatus(activeWorkspaceRoot));
              }}
              disabled={workspaceUnavailable || status === 'loading' || actionsLocked}
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-dev-status"
            >
              Status
            </button>
            <button
              type="button"
              onClick={() => {
                void runAction(() => executeDevLogs(logService || null, activeWorkspaceRoot));
              }}
              disabled={workspaceUnavailable || status === 'loading' || actionsLocked}
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-dev-logs"
            >
              Logs
            </button>
          </div>

          {preview && (
            <div
              className="mt-6 border border-cyan-400/20 bg-cyan-400/10 p-4"
              data-testid="dev-up-preview"
            >
              <p className="text-sm font-medium text-cyan-100">Review local start plan</p>

              <div className="mt-4 space-y-4 text-sm text-cyan-50/90">
                <div>
                  <p className="font-medium">Selected services</p>
                  <div className="mt-2 space-y-1">
                    {selectedServices.map(([label, path]) => (
                      <p key={path}>
                        {label}: {toDisplayPath(activeWorkspaceRoot, path)}
                      </p>
                    ))}
                  </div>
                </div>

                <div>
                  <p className="font-medium">Detected dependencies</p>
                  <div className="mt-2 space-y-1">
                    <p>
                      Databases:{' '}
                      {preview.dependencies.databases.length > 0
                        ? preview.dependencies.databases
                            .map((database) => `${database.name} (${database.service})`)
                            .join(', ')
                        : 'none'}
                    </p>
                    <p>
                      Kafka topics:{' '}
                      {preview.dependencies.kafka_topics.length > 0
                        ? preview.dependencies.kafka_topics.join(', ')
                        : 'none'}
                    </p>
                    <p>Redis: {preview.dependencies.has_redis ? 'enabled' : 'disabled'}</p>
                    <p>
                      Redis session:{' '}
                      {preview.dependencies.has_redis_session ? 'enabled' : 'disabled'}
                    </p>
                    <p>Auth mode: {authMode}</p>
                  </div>
                </div>

                <div>
                  <p className="font-medium">Additional services</p>
                  <div className="mt-2 space-y-1">
                    {preview.additional_services.length === 0 ? (
                      <p>none</p>
                    ) : (
                      preview.additional_services.map((service) => (
                        <p key={service.name}>
                          {service.name}: {service.url}
                        </p>
                      ))
                    )}
                  </div>
                </div>

                <div>
                  <p className="font-medium">Resolved endpoints</p>
                  <div className="mt-2 space-y-1">
                    {getPreviewPorts().map((port) => (
                      <p key={port.name}>
                        {port.name}: {port.value}
                      </p>
                    ))}
                  </div>
                </div>
              </div>

              <div className="mt-4 flex flex-wrap gap-3">
                <button
                  type="button"
                  onClick={() => setPreview(null)}
                  className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)]"
                >
                  Back
                </button>
                <button
                  type="button"
                  onClick={() => {
                    void handleConfirmStart();
                  }}
                  className="bg-cyan-500/85 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500"
                  data-testid="btn-confirm-dev-up"
                >
                  Confirm start
                </button>
              </div>
            </div>
          )}

          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">Output</h2>
          <div className="mt-4 border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.35)] p-4">
            <pre className="min-h-72 overflow-auto whitespace-pre-wrap text-xs text-slate-100">
              {output || 'Run status or logs to inspect the local development environment.'}
            </pre>
          </div>
        </section>
      </div>
    </div>
  );
}
