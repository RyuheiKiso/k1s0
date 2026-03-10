import { useState } from 'react';
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
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;

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
    <div className="glass max-w-4xl p-6" data-testid="config-types-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Types</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Generate config contracts</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Preview and write TypeScript and Dart config types from the workspace schema.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before generating files.
        </p>
      )}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.95fr_1.05fr]">
        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <div className="space-y-5">
            <div>
              <label className="block text-sm font-medium text-slate-200/82">Schema path</label>
              <input
                type="text"
                value={schemaPath}
                onChange={(event) => setSchemaPath(event.target.value)}
                className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                data-testid="input-schema-path"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-200/82">Output directory</label>
              <input
                type="text"
                value={outputDir}
                onChange={(event) => setOutputDir(event.target.value)}
                className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
                data-testid="input-output-dir"
              />
            </div>

            <fieldset className="space-y-2">
              <legend className="text-sm font-medium text-slate-200/82">Target</legend>
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
              disabled={previewStatus === 'loading' || !schemaPath || workspaceUnavailable}
              className="rounded-xl border border-white/15 bg-white/6 px-5 py-2.5 text-sm font-medium text-white/85 transition hover:bg-white/10 disabled:opacity-50"
              data-testid="btn-preview"
            >
              {previewStatus === 'loading' ? 'Previewing...' : 'Preview'}
            </button>
            <button
              type="button"
              onClick={() => {
                void handleWrite();
              }}
              disabled={
                writeStatus === 'loading' || !schemaPath || !outputDir || workspaceUnavailable
              }
              className="rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
              data-testid="btn-generate"
            >
              {writeStatus === 'loading' ? 'Writing...' : 'Write files'}
            </button>
          </div>

          {(previewStatus === 'error' || writeStatus === 'error') && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}

          {writeStatus === 'success' && writtenFiles.length > 0 && (
            <div className="mt-5 rounded-2xl border border-emerald-400/20 bg-emerald-400/10 p-4">
              <p className="text-sm font-medium text-emerald-100">Generated files</p>
              <div className="mt-3 space-y-2 text-sm text-emerald-50/90">
                {writtenFiles.map((file) => (
                  <p key={file.path}>{toDisplayPath(activeWorkspaceRoot, file.path)}</p>
                ))}
              </div>
            </div>
          )}
        </section>

        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <h2 className="text-lg font-semibold text-white">Preview</h2>
          <div className="mt-4 space-y-4">
            {previewResults.length === 0 && writtenFiles.length === 0 ? (
              <p className="text-sm text-slate-200/55">
                Run preview or write files to inspect generated output.
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
                  className="rounded-2xl border border-white/10 bg-slate-950/40 p-4"
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
