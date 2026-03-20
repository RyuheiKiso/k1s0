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
    <div className="glass max-w-5xl p-6 p3-animate-in" data-testid="event-codegen-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">イベント</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">イベントアセットの生成</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        `events.yaml`からイベント生成をプレビューし、プロデューサー、コンシューマー、protoファイル、アウトボックスSQLを書き出します。
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          イベント生成を実行する前に有効なワークスペースルートを設定してください。
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div>
            <label className="block text-sm font-medium text-slate-200/82">イベントファイル</label>
            <input
              value={eventsPath}
              onChange={(event) => setEventsPath(event.target.value)}
              className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
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
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-preview-event"
            >
              プレビュー
            </button>
            <button
              type="button"
              onClick={() => {
                void handleGenerate();
              }}
              disabled={workspaceUnavailable || !eventsPath || status === 'loading' || actionsLocked}
              className="bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
              data-testid="btn-generate-event"
            >
              生成
            </button>
          </div>

          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">出力</h2>
          {generatedFiles.length > 0 ? (
            <div className="mt-4 space-y-2">
              {generatedFiles.map((file) => (
                <p key={file} className="text-sm text-slate-100">
                  {toDisplayPath(activeWorkspaceRoot, file)}
                </p>
              ))}
            </div>
          ) : (
            <pre className="mt-4 min-h-72 overflow-auto whitespace-pre-wrap border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.35)] p-4 text-xs text-slate-100">
              {preview || 'イベントファイルをプレビューして生成予定の出力を確認します。'}
            </pre>
          )}
        </section>
      </div>
    </div>
  );
}
