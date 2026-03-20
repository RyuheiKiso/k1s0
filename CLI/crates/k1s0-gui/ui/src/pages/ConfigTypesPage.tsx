import { useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import {
  executeGenerateConfigTypes,
  writeConfigTypes,
  type GenerateTarget,
  type GeneratedFileResult,
  type TypeOutputTarget,
} from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

type PreviewResult = { label: string; content: string };

function expandTargets(target: TypeOutputTarget): GenerateTarget[] {
  return target === 'both' ? ['typescript', 'dart'] : [target];
}

function formatLabel(target: GenerateTarget) {
  return target === 'typescript' ? 'TypeScript' : 'Dart';
}

export default function ConfigTypesPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [schemaPath, setSchemaPath] = useState('config/config-schema.yaml');
  const [outputDir, setOutputDir] = useState('src/config/__generated__');
  const [target, setTarget] = useState<TypeOutputTarget>('both');
  const [previewStatus, setPreviewStatus] = useState<'idle' | 'loading' | 'success' | 'error'>(
    'idle',
  );
  const [writeStatus, setWriteStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [previewResults, setPreviewResults] = useState<PreviewResult[]>([]);
  const [writtenFiles, setWrittenFiles] = useState<GeneratedFileResult[]>([]);
  const [errorMessage, setErrorMessage] = useState('');

  async function handlePreview() {
    setPreviewStatus('loading');
    setErrorMessage('');
    setPreviewResults([]);

    try {
      const previews = await Promise.all(
        expandTargets(target).map(async (currentTarget) => ({
          label: formatLabel(currentTarget),
          content: await executeGenerateConfigTypes(
            schemaPath,
            currentTarget,
            activeWorkspaceRoot,
          ),
        })),
      );
      setPreviewResults(previews);
      setPreviewStatus('success');
    } catch (error) {
      setPreviewStatus('error');
      setErrorMessage(String(error));
    }
  }

  async function handleWrite() {
    setWriteStatus('loading');
    setErrorMessage('');
    setWrittenFiles([]);

    try {
      const files = await writeConfigTypes(
        schemaPath,
        outputDir,
        expandTargets(target),
        activeWorkspaceRoot,
      );
      setWrittenFiles(files);
      setWriteStatus('success');
    } catch (error) {
      setWriteStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-4xl p-6 p3-animate-in" data-testid="config-types-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">型定義</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">設定コントラクトの生成</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        ワークスペーススキーマからTypeScriptとDartの設定型をプレビュー・書き出しします。
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          ファイルを生成する前に有効なワークスペースルートを設定してください。
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.95fr_1.05fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div className="space-y-5">
            <div>
              <label className="block text-sm font-medium text-slate-200/82">スキーマパス</label>
              <input
                type="text"
                value={schemaPath}
                onChange={(event) => setSchemaPath(event.target.value)}
                className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                data-testid="input-schema-path"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-200/82">出力ディレクトリ</label>
              <input
                type="text"
                value={outputDir}
                onChange={(event) => setOutputDir(event.target.value)}
                className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                data-testid="input-output-dir"
              />
            </div>

            <fieldset className="space-y-2">
              <legend className="text-sm font-medium text-slate-200/82">ターゲット</legend>
              {(['typescript', 'dart', 'both'] as TypeOutputTarget[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={target === value}
                    onChange={() => setTarget(value)}
                    name="config-types-target"
                  />
                  {value === 'both' ? 'TypeScript + Dart' : formatLabel(value)}
                </label>
              ))}
            </fieldset>
          </div>

          <div className="mt-6 flex flex-wrap gap-3">
            <button
              type="button"
              onClick={() => {
                void handlePreview();
              }}
              disabled={
                previewStatus === 'loading' || !schemaPath || workspaceUnavailable || actionsLocked
              }
              className="border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-[rgba(0,200,255,0.08)] disabled:opacity-50"
              data-testid="btn-preview"
            >
              {previewStatus === 'loading' ? 'プレビュー中...' : 'プレビュー'}
            </button>
            <button
              type="button"
              onClick={() => {
                void handleWrite();
              }}
              disabled={
                writeStatus === 'loading' ||
                !schemaPath ||
                !outputDir ||
                workspaceUnavailable ||
                actionsLocked
              }
              className="bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
              data-testid="btn-generate"
            >
              {writeStatus === 'loading' ? '書き出し中...' : 'ファイルを書き出し'}
            </button>
          </div>

          {(previewStatus === 'error' || writeStatus === 'error') && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}

          {writeStatus === 'success' && writtenFiles.length > 0 && (
            <div className="mt-5 border border-cyan-400/20 bg-cyan-400/10 p-4">
              <p className="text-sm font-medium text-cyan-100">生成されたファイル</p>
              <div className="mt-3 space-y-2 text-sm text-cyan-50/90">
                {writtenFiles.map((file) => (
                  <p key={file.path}>{toDisplayPath(activeWorkspaceRoot, file.path)}</p>
                ))}
              </div>
            </div>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">プレビュー</h2>
          <div className="mt-4 space-y-4">
            {previewResults.length === 0 && writtenFiles.length === 0 ? (
              <p className="text-sm text-slate-200/55">
                プレビューまたはファイル書き出しを実行して生成結果を確認します。
              </p>
            ) : (
              (previewResults.length > 0
                ? previewResults
                : writtenFiles.map((file) => ({
                    label: toDisplayPath(activeWorkspaceRoot, file.path),
                    content: file.preview,
                  }))
              ).map((result) => (
                <div
                  key={result.label}
                  className="border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.40)] p-4"
                >
                  <p className="mb-3 text-sm font-medium text-slate-100">{result.label}</p>
                  <pre className="overflow-auto whitespace-pre-wrap text-xs text-slate-100">
                    {result.content}
                  </pre>
                </div>
              ))
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
