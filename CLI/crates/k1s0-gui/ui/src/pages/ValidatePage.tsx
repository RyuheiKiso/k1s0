import { useState } from 'react';
import { executeValidateConfigSchema, executeValidateNavigation } from '../lib/tauri-commands';

type ValidateTarget = 'config-schema' | 'navigation';

export default function ValidatePage() {
  const [validateTarget, setValidateTarget] = useState<ValidateTarget>('config-schema');
  const [filePath, setFilePath] = useState('config/config-schema.yaml');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorCount, setErrorCount] = useState(0);
  const [errorMessage, setErrorMessage] = useState('');

  function handleTargetChange(nextTarget: ValidateTarget) {
    setValidateTarget(nextTarget);
    setFilePath(
      nextTarget === 'config-schema' ? 'config/config-schema.yaml' : 'config/navigation.yaml',
    );
    setStatus('idle');
    setErrorMessage('');
  }

  async function handleValidate() {
    setStatus('loading');
    setErrorMessage('');

    try {
      const count =
        validateTarget === 'config-schema'
          ? await executeValidateConfigSchema(filePath)
          : await executeValidateNavigation(filePath);
      setErrorCount(count);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-2xl p-6" data-testid="validate-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Quality</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Validate contracts</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Validate configuration and navigation files before moving into build, test, or deploy.
      </p>

      <div className="mt-6 space-y-5">
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
            className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
            data-testid="input-file-path"
          />
        </div>

        <button
          type="button"
          onClick={() => {
            void handleValidate();
          }}
          disabled={status === 'loading' || !filePath}
          className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
          data-testid="btn-validate"
        >
          {status === 'loading' ? 'Validating...' : 'Validate'}
        </button>

        {status === 'success' && (
          <div className="rounded-2xl border border-white/10 bg-white/5 p-4" data-testid="validate-result">
            {errorCount === 0 ? (
              <p className="text-sm text-emerald-300">Validation completed with no errors.</p>
            ) : (
              <p className="text-sm text-rose-300">
                Validation found {errorCount} error(s). Review the console output for details.
              </p>
            )}
          </div>
        )}

        {status === 'error' && (
          <p className="text-sm text-rose-300" data-testid="error-message">
            {errorMessage}
          </p>
        )}
      </div>
    </div>
  );
}
