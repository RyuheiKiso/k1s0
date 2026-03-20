import { useEffect, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import ProgressLog from '../components/ProgressLog';
import { useAuth } from '../lib/auth';
import { toDisplayPath } from '../lib/paths';
import {
  executeBuildWithProgress,
  scanBuildableTargets,
  type BuildMode,
  type ProgressEvent,
} from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

export default function BuildPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [targets, setTargets] = useState<string[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [mode, setMode] = useState<BuildMode>('Development');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    let cancelled = false;

    if (!workspace.ready || !workspace.workspaceRoot) {
      return;
    }

    scanBuildableTargets(activeWorkspaceRoot)
      .then((nextTargets) => {
        if (!cancelled) {
          setTargets(nextTargets);
          setSelected((current) => current.filter((target) => nextTargets.includes(target)));
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
  const selectedTargets = selected.filter((target) => availableTargets.includes(target));

  function toggleTarget(target: string) {
    setSelected((current) =>
      current.includes(target)
        ? current.filter((value) => value !== target)
        : [...current, target],
    );
  }

  function handleToggleAll(checked: boolean) {
    setSelected(checked ? [...availableTargets] : []);
  }

  function handleProgress(event: ProgressEvent) {
    setEvents((current) => [...current, event]);

    switch (event.kind) {
      case 'StepStarted':
        setCurrentStep(event.step);
        setTotalSteps(event.total);
        break;
      case 'StepCompleted':
        setCurrentStep(event.step);
        break;
      case 'Finished':
        setStatus(event.success ? 'success' : 'error');
        if (!event.success) {
          setErrorMessage(event.message);
        }
        break;
      case 'Error':
        setErrorMessage(event.message);
        break;
      default:
        break;
    }
  }

  async function handleBuild() {
    setStatus('loading');
    setErrorMessage('');
    setEvents([]);
    setCurrentStep(0);
    setTotalSteps(selectedTargets.length);

    let finished = false;

    try {
      await executeBuildWithProgress({ targets: selectedTargets, mode }, (event) => {
        if (event.kind === 'Finished') {
          finished = true;
        }
        handleProgress(event);
      });

      if (!finished) {
        setStatus('error');
        setErrorMessage('Build completed without a terminal progress event.');
      }
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  const allSelected =
    availableTargets.length > 0 && selectedTargets.length === availableTargets.length;

  return (
    <div className="glass max-w-4xl p-6 p3-animate-in" data-testid="build-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">Delivery</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Build release artifacts</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Run builds only against verified targets. A build is marked successful only after the
        backend emits a successful terminal event.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          Configure a valid workspace root before scanning build targets.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[1.1fr_0.9fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div className="flex items-center justify-between gap-3">
            <h2 className="text-lg font-semibold text-white">Targets</h2>
            {availableTargets.length > 0 && (
              <label className="flex items-center gap-2 text-sm text-slate-200/72">
                <input
                  type="checkbox"
                  checked={allSelected}
                  onChange={(event) => handleToggleAll(event.target.checked)}
                />
                All targets
              </label>
            )}
          </div>

          <div className="mt-4 space-y-2">
            {availableTargets.length === 0 ? (
              <p className="text-sm text-slate-200/55">No buildable targets were found.</p>
            ) : (
              availableTargets.map((target) => (
                <label
                  key={target}
                  className="flex items-center gap-3 border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-2 text-sm text-slate-100"
                >
                  <input
                    type="checkbox"
                    checked={selectedTargets.includes(target)}
                    onChange={() => toggleTarget(target)}
                  />
                  {toDisplayPath(activeWorkspaceRoot, target)}
                </label>
              ))
            )}
          </div>
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">Mode</h2>
          <div className="mt-4 space-y-2">
            {(['Development', 'Production'] as BuildMode[]).map((value) => (
              <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={mode === value}
                  onChange={() => setMode(value)}
                  name="build-mode"
                />
                {value.toLowerCase()}
              </label>
            ))}
          </div>

          <button
            type="button"
            onClick={() => {
              void handleBuild();
            }}
            disabled={
              status === 'loading' ||
              workspaceUnavailable ||
              actionsLocked ||
              selectedTargets.length === 0
            }
            className="mt-6 bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
            data-testid="btn-build"
          >
            {status === 'loading' ? 'Building...' : 'Build'}
          </button>

          {status === 'success' && (
            <p className="mt-4 text-sm text-cyan-300" data-testid="success-message">
              Build completed successfully.
            </p>
          )}
          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>
      </div>

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
