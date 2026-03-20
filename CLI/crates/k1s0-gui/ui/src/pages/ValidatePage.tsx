import { useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import {
  executeValidateConfigSchema,
  executeValidateNavigation,
  type ValidationDiagnostic,
} from '../lib/tauri-commands';
import { useWorkspace } from '../lib/workspace';

type ValidateTarget = 'config-schema' | 'navigation';

function formatDiagnosticLocation(diagnostic: ValidationDiagnostic) {
  return diagnostic.line ? `${diagnostic.path} (line ${diagnostic.line})` : diagnostic.path;
}

export default function ValidatePage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [validateTarget, setValidateTarget] = useState<ValidateTarget>('config-schema');
  const [filePath, setFilePath] = useState('config/config-schema.yaml');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [diagnostics, setDiagnostics] = useState<ValidationDiagnostic[]>([]);
  const [errorMessage, setErrorMessage] = useState('');

  function handleTargetChange(nextTarget: ValidateTarget) {
    setValidateTarget(nextTarget);
    setFilePath(
      nextTarget === 'config-schema' ? 'config/config-schema.yaml' : 'config/navigation.yaml',
    );
    setStatus('idle');
    setErrorMessage('');
    setDiagnostics([]);
  }

  async function handleValidate() {
    setStatus('loading');
    setErrorMessage('');
    setDiagnostics([]);

    try {
      const nextDiagnostics =
        validateTarget === 'config-schema'
          ? await executeValidateConfigSchema(filePath, activeWorkspaceRoot)
          : await executeValidateNavigation(filePath, activeWorkspaceRoot);
      setDiagnostics(nextDiagnostics);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-4xl p-6 p3-animate-in" data-testid="validate-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">Quality</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Validate contracts</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Validate configuration and navigation files against the selected workspace root before
        build, test, or deploy.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          Configure a valid workspace root before running validation.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="space-y-5 border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <fieldset className="space-y-2">
            <legend className="text-sm font-medium text-slate-200/82">Target</legend>
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={validateTarget === 'config-schema'}
                onChange={() => handleTargetChange('config-schema')}
                name="validate-target"
              />
              Config schema
            </label>
            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={validateTarget === 'navigation'}
                onChange={() => handleTargetChange('navigation')}
                name="validate-target"
              />
              Navigation
            </label>
          </fieldset>

          <div>
            <label className="block text-sm font-medium text-slate-200/82">File path</label>
            <input
              type="text"
              value={filePath}
              onChange={(event) => setFilePath(event.target.value)}
              className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
              data-testid="input-file-path"
            />
          </div>

          <button
            type="button"
            onClick={() => {
              void handleValidate();
            }}
            disabled={status === 'loading' || !filePath || workspaceUnavailable || actionsLocked}
            className="bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
            data-testid="btn-validate"
          >
            {status === 'loading' ? 'Validating...' : 'Validate'}
          </button>

          {status === 'error' && (
            <p className="text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">Diagnostics</h2>

          {status !== 'success' ? (
            <p className="mt-4 text-sm text-slate-200/55">
              Run validation to inspect detailed schema or navigation diagnostics.
            </p>
          ) : diagnostics.length === 0 ? (
            <div
              className="mt-4 border border-cyan-400/20 bg-cyan-400/10 p-4"
              data-testid="validate-result"
            >
              <p className="text-sm text-cyan-300">Validation completed with no errors.</p>
            </div>
          ) : (
            <div
              className="mt-4 space-y-3 border border-rose-400/20 bg-rose-400/10 p-4"
              data-testid="validate-result"
            >
              <p className="text-sm font-medium text-rose-200">
                Validation found {diagnostics.length} error(s).
              </p>
              <div className="space-y-3">
                {diagnostics.map((diagnostic, index) => (
                  <div
                    key={`${diagnostic.rule}-${diagnostic.path}-${index}`}
                    className="border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.35)] p-3"
                  >
                    <p className="text-xs uppercase tracking-[0.18em] text-rose-200/75">
                      {diagnostic.rule}
                    </p>
                    <p className="mt-2 text-sm text-slate-100">
                      {formatDiagnosticLocation(diagnostic)}
                    </p>
                    <p className="mt-2 text-sm text-slate-200/80">{diagnostic.message}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
