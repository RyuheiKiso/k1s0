import { useState } from 'react';
import { executeGenerateNavigationTypes, type GenerateTarget } from '../lib/tauri-commands';

export default function NavigationTypesPage() {
  const [navPath, setNavPath] = useState('config/navigation.yaml');
  const [target, setTarget] = useState<GenerateTarget>('typescript');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [output, setOutput] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  async function handleGenerate() {
    setStatus('loading');
    setOutput('');
    setErrorMessage('');

    try {
      const result = await executeGenerateNavigationTypes(navPath, target);
      setOutput(result);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-3xl p-6" data-testid="navigation-types-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Types</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Generate navigation contracts</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Generate navigation route types from `config/navigation.yaml`.
      </p>

      <div className="mt-6 space-y-5">
        <div>
          <label className="block text-sm font-medium text-slate-200/82">Navigation file</label>
          <input
            type="text"
            value={navPath}
            onChange={(event) => setNavPath(event.target.value)}
            className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
            data-testid="input-nav-path"
          />
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium text-slate-200/82">Target</legend>
          {(['typescript', 'dart'] as GenerateTarget[]).map((value) => (
            <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="radio"
                checked={target === value}
                onChange={() => setTarget(value)}
                name="navigation-types-target"
              />
              {value}
            </label>
          ))}
        </fieldset>

        <button
          type="button"
          onClick={() => {
            void handleGenerate();
          }}
          disabled={status === 'loading' || !navPath}
          className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
          data-testid="btn-generate"
        >
          {status === 'loading' ? 'Generating...' : 'Generate'}
        </button>

        {status === 'error' && (
          <p className="text-sm text-rose-300" data-testid="error-message">
            {errorMessage}
          </p>
        )}

        {status === 'success' && output && (
          <div className="rounded-2xl border border-white/10 bg-slate-950/40 p-4" data-testid="output-area">
            <pre className="overflow-auto whitespace-pre-wrap text-xs text-slate-100">{output}</pre>
          </div>
        )}
      </div>
    </div>
  );
}
