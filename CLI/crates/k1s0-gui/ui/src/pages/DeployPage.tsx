import { useEffect, useState } from 'react';
import HelpButton from '../components/HelpButton';
import ProgressLog from '../components/ProgressLog';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { toDisplayPath } from '../lib/paths';
import {
  executeDeployRollback,
  executeDeployWithProgress,
  getFailedProdRollbackTarget,
  scanDeployableTargets,
  type Environment,
  type ProgressEvent,
} from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

export default function DeployPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [targets, setTargets] = useState<string[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [environment, setEnvironment] = useState<Environment>('Dev');
  const [prodConfirm, setProdConfirm] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);
  const [rollbackStatus, setRollbackStatus] = useState<'idle' | 'loading' | 'success' | 'error'>(
    'idle',
  );
  const [rollbackMessage, setRollbackMessage] = useState('');
  const [failedRollbackTarget, setFailedRollbackTarget] = useState<string | null>(null);

  async function refreshRollbackTarget() {
    try {
      const nextTarget = await getFailedProdRollbackTarget();
      setFailedRollbackTarget(nextTarget ?? null);
    } catch {
      setFailedRollbackTarget(null);
    }
  }

  useEffect(() => {
    let cancelled = false;

    setSelected([]);
    setTargets([]);

    if (!workspace.ready && workspace.workspaceRoot === '') {
      return;
    }

    if (workspace.ready && !workspace.workspaceRoot) {
      return;
    }

    scanDeployableTargets(activeWorkspaceRoot)
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
  }, [activeWorkspaceRoot, workspace.ready, workspace.workspaceRoot]);

  useEffect(() => {
    void refreshRollbackTarget();
  }, []);

  function toggleTarget(target: string) {
    setSelected((current) =>
      current.includes(target)
        ? current.filter((value) => value !== target)
        : [...current, target],
    );
  }

  function handleToggleAll(checked: boolean) {
    setSelected(checked ? [...targets] : []);
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

  async function handleDeploy() {
    if (environment === 'Prod' && prodConfirm !== 'deploy') {
      setStatus('error');
      setErrorMessage('本番デプロイを確認するには「deploy」と入力してください。');
      return;
    }

    setStatus('loading');
    setErrorMessage('');
    setEvents([]);
    setCurrentStep(0);
    setTotalSteps(selected.length);
    setRollbackStatus('idle');
    setRollbackMessage('');
    setFailedRollbackTarget(null);

    let finished = false;

    try {
      await executeDeployWithProgress({ environment, targets: selected }, (event) => {
        if (event.kind === 'Finished') {
          finished = true;
        }
        handleProgress(event);
      });

      if (!finished) {
        setStatus('error');
        setErrorMessage('終了イベントなしでデプロイが完了しました。');
      }
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    } finally {
      await refreshRollbackTarget();
    }
  }

  async function handleRollback() {
    if (!failedRollbackTarget) {
      return;
    }

    setRollbackStatus('loading');
    setRollbackMessage('');

    try {
      const message = await executeDeployRollback(failedRollbackTarget);
      setRollbackStatus('success');
      setRollbackMessage(message);
    } catch (error) {
      setRollbackStatus('error');
      setRollbackMessage(String(error));
    } finally {
      await refreshRollbackTarget();
    }
  }

  const allSelected = targets.length > 0 && selected.length === targets.length;
  const canRollback = status === 'error' && Boolean(failedRollbackTarget);
  const showRollbackPanel = canRollback || rollbackMessage !== '';

  return (
    <div className="glass max-w-5xl p-6 p3-animate-in" data-testid="deploy-page">
      {/* ページヘッダーとヘルプボタン */}
      <div className="flex items-center gap-3">
        <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55 p3-eyebrow-reveal">デリバリー</p>
        <HelpButton helpKey="deploy" size="md" />
      </div>
      <h1 className="mt-2 text-3xl font-semibold text-white p3-heading-glitch">サービスのデプロイ</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        デプロイフローはDockerビルド、プッシュ、Cosign署名、Helmデプロイを一つのパイプラインとして実行します。
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100 p3-warning-flicker">
          デプロイターゲットをスキャンする前に有効なワークスペースルートを設定してください。
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div className="flex items-center gap-2">
            <h2 className="text-lg font-semibold text-white p3-heading-glow">環境</h2>
            <HelpButton helpKey="deploy.environment" />
          </div>
          <div className="mt-4 space-y-2">
            {(['Dev', 'Staging', 'Prod'] as Environment[]).map((value) => (
              <label
                key={value}
                className="flex items-center gap-3 text-sm text-slate-200/82"
                htmlFor={`env-${value}`}
              >
                <input
                  id={`env-${value}`}
                  type="radio"
                  checked={environment === value}
                  onChange={() => setEnvironment(value)}
                  name="deploy-environment"
                />
                {value.toLowerCase()}
              </label>
            ))}
          </div>

          {environment === 'Prod' && (
            <div
              className="mt-5 border border-red-400/25 bg-red-400/10 p-4 p3-warning-flicker"
              data-testid="prod-confirm"
            >
              <p className="flex items-center gap-2 text-sm text-red-100">
                本番デプロイには明示的な確認トークンが必要です。
                <HelpButton helpKey="deploy.prodConfirm" />
              </p>
              <input
                value={prodConfirm}
                onChange={(event) => setProdConfirm(event.target.value)}
                placeholder="deploy"
                className="mt-3 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                data-testid="input-prod-confirm"
              />
            </div>
          )}

          <button
            type="button"
            onClick={() => {
              void handleDeploy();
            }}
            disabled={
              status === 'loading' || selected.length === 0 || workspaceUnavailable || actionsLocked
            }
            className="mt-6 bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
            data-testid="btn-deploy"
          >
            {status === 'loading' ? 'デプロイ中...' : 'デプロイ'}
          </button>

          {status === 'success' && (
            <p className="mt-4 text-sm text-cyan-300" data-testid="success-message">
              デプロイが正常に完了しました。
            </p>
          )}
          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}

          {showRollbackPanel && (
            <div className="mt-5 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-4 p3-expand-in">
              {failedRollbackTarget && (
                <>
                  <p className="text-sm text-slate-200/76">
                    最後に失敗した本番デプロイは安全にロールバックできます。
                  </p>
                  <p className="mt-2 text-sm text-slate-100">
                    ターゲット: {toDisplayPath(activeWorkspaceRoot, failedRollbackTarget)}
                  </p>
                  <button
                    type="button"
                    onClick={() => {
                      void handleRollback();
                    }}
                    disabled={rollbackStatus === 'loading' || actionsLocked}
                    className="mt-4 border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-4 py-2 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
                    data-testid="btn-rollback"
                  >
                    {rollbackStatus === 'loading' ? 'ロールバック中...' : 'ロールバック'}
                  </button>
                </>
              )}
              {rollbackMessage && (
                <p
                  className={`mt-3 text-sm ${
                    rollbackStatus === 'success' ? 'text-cyan-300' : 'text-rose-300'
                  }`}
                >
                  {rollbackMessage}
                </p>
              )}
            </div>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div className="flex items-center justify-between gap-3">
            <span className="flex items-center gap-2">
              <h2 className="text-lg font-semibold text-white p3-heading-glow">ターゲット</h2>
              <HelpButton helpKey="deploy.targets" />
            </span>
            {targets.length > 0 && (
              <label className="flex items-center gap-2 text-sm text-slate-200/72">
                <input
                  type="checkbox"
                  checked={allSelected}
                  onChange={(event) => handleToggleAll(event.target.checked)}
                />
                全ターゲット
              </label>
            )}
          </div>

          <div className="mt-4 space-y-2">
            {targets.length === 0 ? (
              <p className="text-sm text-slate-200/55">デプロイ可能なターゲットが見つかりませんでした。</p>
            ) : (
              targets.map((target) => (
                <label
                  key={target}
                  className="flex items-center gap-3 border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-2 text-sm text-slate-100"
                >
                  <input
                    type="checkbox"
                    checked={selected.includes(target)}
                    onChange={() => toggleTarget(target)}
                  />
                  {toDisplayPath(activeWorkspaceRoot, target)}
                </label>
              ))
            )}
          </div>
        </section>
      </div>

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
