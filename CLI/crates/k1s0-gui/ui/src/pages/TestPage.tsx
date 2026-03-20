import { useEffect, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import ProgressLog from '../components/ProgressLog';
import { useAuth } from '../lib/auth';
import { toDisplayPath } from '../lib/paths';
import {
  executeTestWithProgressAt,
  scanTestableTargets,
  type ProgressEvent,
  type TestKind,
} from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

export default function TestPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [targets, setTargets] = useState<string[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [kind, setKind] = useState<TestKind>('Unit');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    let cancelled = false;

    if (!workspace.ready || !workspace.workspaceRoot || kind === 'All') {
      return;
    }

    scanTestableTargets(activeWorkspaceRoot)
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
  }, [activeWorkspaceRoot, kind, workspace.ready, workspace.workspaceRoot]);

  const availableTargets =
    workspace.ready && workspace.workspaceRoot && kind !== 'All' ? targets : [];
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

  async function handleTest() {
    setStatus('loading');
    setErrorMessage('');
    setEvents([]);
    setCurrentStep(0);
    setTotalSteps(kind === 'All' ? 0 : selectedTargets.length);

    let finished = false;

    try {
      await executeTestWithProgressAt(
        { kind, targets: kind === 'All' ? [] : selectedTargets },
        activeWorkspaceRoot,
        (event) => {
          if (event.kind === 'Finished') {
            finished = true;
          }
          handleProgress(event);
        },
      );

      if (!finished) {
        setStatus('error');
        setErrorMessage('Test execution completed without a terminal progress event.');
      }
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  const allSelected =
    availableTargets.length > 0 && selectedTargets.length === availableTargets.length;

  return (
    <div className="glass max-w-4xl p-6 p3-animate-in" data-testid="test-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">Quality</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Run test suites</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Execute unit, integration, or full coverage tests for the current workspace.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          Configure a valid workspace root before scanning test targets.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.85fr_1.15fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">Test kind</h2>
          <div className="mt-4 space-y-2">
            {(['Unit', 'Integration', 'All'] as TestKind[]).map((value) => (
              <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                <input
                  type="radio"
                  checked={kind === value}
                  onChange={() => {
                    setKind(value);
                    setSelected([]);
                  }}
                  name="test-kind"
                />
                {value}
              </label>
            ))}
          </div>

          <button
            type="button"
            onClick={() => {
              void handleTest();
            }}
            disabled={
              status === 'loading' ||
              workspaceUnavailable ||
              actionsLocked ||
              (kind !== 'All' && selectedTargets.length === 0)
            }
            className="mt-6 bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
            data-testid="btn-test"
          >
            {status === 'loading' ? 'Running...' : 'Run tests'}
          </button>

          {status === 'success' && (
            <p className="mt-4 text-sm text-cyan-300" data-testid="success-message">
              Test execution completed successfully.
            </p>
          )}
          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        {kind !== 'All' && (
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
                <p className="text-sm text-slate-200/55">No testable targets were found.</p>
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
        )}
      </div>

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
