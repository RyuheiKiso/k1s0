import { useState } from 'react';
import { executeGenerateConfigTypes, type GenerateTarget } from '../lib/tauri-commands';
import * as RadioGroup from '@radix-ui/react-radio-group';

export default function ConfigTypesPage() {
  const [schemaPath, setSchemaPath] = useState('config/config-schema.yaml');
  const [target, setTarget] = useState<GenerateTarget>('typescript');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [output, setOutput] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  const handleGenerate = async () => {
    setStatus('loading');
    setOutput('');
    setErrorMessage('');
    try {
      const result = await executeGenerateConfigTypes(schemaPath, target);
      setOutput(result);
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  return (
    <div className="glass p-6 max-w-2xl" data-testid="config-types-page">
      <h1 className="text-2xl font-bold mb-6 text-white">設定スキーマ型生成</h1>
      <p className="text-white/60 text-sm mb-6">
        config-schema.yaml から TypeScript / Dart の型定義を生成します。
      </p>

      <div className="mb-4">
        <label className="block text-sm font-medium text-white/90 mb-1">
          config-schema.yaml のパス
        </label>
        <input
          type="text"
          value={schemaPath}
          onChange={(e) => setSchemaPath(e.target.value)}
          className="w-full bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-indigo-400"
          data-testid="input-schema-path"
        />
      </div>

      <div className="mb-6">
        <h2 className="font-semibold mb-3 text-white/90">生成ターゲット</h2>
        <RadioGroup.Root
          value={target}
          onValueChange={(v) => setTarget(v as GenerateTarget)}
          className="flex gap-4"
        >
          {(['typescript', 'dart'] as GenerateTarget[]).map((t) => (
            <div key={t} className="flex items-center gap-2">
              <RadioGroup.Item
                value={t}
                className="w-4 h-4 rounded-full border border-white/40 data-[state=checked]:border-indigo-400 data-[state=checked]:bg-indigo-400/20"
              >
                <RadioGroup.Indicator className="flex items-center justify-center w-full h-full after:block after:w-2 after:h-2 after:rounded-full after:bg-indigo-400" />
              </RadioGroup.Item>
              <label className="text-sm text-white/80">{t}</label>
            </div>
          ))}
        </RadioGroup.Root>
      </div>

      <button
        onClick={handleGenerate}
        disabled={status === 'loading' || !schemaPath}
        className="bg-indigo-500/80 hover:bg-indigo-500 text-white px-5 py-2.5 rounded-xl transition-all duration-200 shadow-lg shadow-indigo-500/20 disabled:opacity-40"
        data-testid="btn-generate"
      >
        {status === 'loading' ? '生成中...' : '型定義を生成'}
      </button>

      {status === 'error' && (
        <p className="text-rose-400 mt-3 text-sm" data-testid="error-message">{errorMessage}</p>
      )}

      {status === 'success' && output && (
        <div className="mt-4" data-testid="output-area">
          <p className="text-emerald-400 text-sm mb-2">生成完了</p>
          <pre className="bg-black/30 rounded-lg p-3 text-xs text-white/80 overflow-auto max-h-96 whitespace-pre">
            {output}
          </pre>
        </div>
      )}
    </div>
  );
}
