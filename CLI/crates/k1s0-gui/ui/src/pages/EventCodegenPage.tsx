import { useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { executeEventCodegen, previewEventCodegen } from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

export default function EventCodegenPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [eventsPath, setEventsPath] = useState('events.yaml');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [preview, setPreview] = useState('');
  const [generatedFiles, setGeneratedFiles] = useState<string[]>([]);

  async function handlePreview() {
    setStatus('loading');
    setErrorMessage('');

    try {
      const summary = await previewEventCodegen(eventsPath, activeWorkspaceRoot);
      setPreview(summary);
      setGeneratedFiles([]);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  async function handleGenerate() {
    setStatus('loading');
    setErrorMessage('');

    try {
      const files = await executeEventCodegen(eventsPath, activeWorkspaceRoot);
      setGeneratedFiles(files);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-5xl p-6" data-testid="event-codegen-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Events</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Generate event assets</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Preview event generation and write producers, consumers, proto files, and outbox SQL from
        `events.yaml`.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before running event generation.
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <div>
            <label className="block text-sm font-medium text-slate-200/82">Events file</label>
            <input
              value={eventsPath}
              onChange={(event) => setEventsPath(event.target.value)}
              className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
              data-testid="input-events-path"
            />
          </div>

          <div className="mt-6 flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => {
                void handlePreview();
              }}
              disabled={workspaceUnavailable || !eventsPath || status === 'loading' || actionsLocked}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
              data-testid="btn-preview-event"
            >
              Preview
            </button>
            <button
              type="button"
              onClick={() => {
                void handleGenerate();
              }}
              disabled={workspaceUnavailable || !eventsPath || status === 'loading' || actionsLocked}
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
              data-testid="btn-generate-event"
            >
              Generate
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
          {generatedFiles.length > 0 ? (
            <div className="mt-4 space-y-2">
              {generatedFiles.map((file) => (
                <p key={file} className="text-sm text-slate-100">
                  {toDisplayPath(activeWorkspaceRoot, file)}
                </p>
              ))}
            </div>
          ) : (
            <pre className="mt-4 min-h-72 overflow-auto whitespace-pre-wrap rounded-2xl border border-white/10 bg-slate-950/35 p-4 text-xs text-slate-100">
              {preview || 'Preview the events file to inspect the planned generation output.'}
            </pre>
          )}
        </section>
      </div>
    </div>
  );
}
