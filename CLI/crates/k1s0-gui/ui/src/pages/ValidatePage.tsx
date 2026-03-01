import { useState } from 'react';
import { executeValidateConfigSchema, executeValidateNavigation } from '../lib/tauri-commands';
import * as RadioGroup from '@radix-ui/react-radio-group';

type ValidateTarget = 'config-schema' | 'navigation';

export default function ValidatePage() {
  const [validateTarget, setValidateTarget] = useState<ValidateTarget>('config-schema');
  const [filePath, setFilePath] = useState('config/config-schema.yaml');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorCount, setErrorCount] = useState(0);
  const [errorMessage, setErrorMessage] = useState('');

  const handleTargetChange = (v: ValidateTarget) => {
    setValidateTarget(v);
    setFilePath(v === 'config-schema' ? 'config/config-schema.yaml' : 'config/navigation.yaml');
    setStatus('idle');
  };

  const handleValidate = async () => {
    setStatus('loading');
    setErrorMessage('');
    try {
      const count = validateTarget === 'config-schema'
        ? await executeValidateConfigSchema(filePath)
        : await executeValidateNavigation(filePath);
      setErrorCount(count);
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  return (
    <div className="glass p-6 max-w-lg" data-testid="validate-page">
      <h1 className="text-2xl font-bold mb-6 text-white">バリデーション</h1>
      <p className="text-white/60 text-sm mb-6">
        設定スキーマまたはナビゲーション定義の整合性を検証します。
      </p>

      <div className="mb-4">
        <h2 className="font-semibold mb-3 text-white/90">検証対象</h2>
        <RadioGroup.Root
          value={validateTarget}
          onValueChange={(v) => handleTargetChange(v as ValidateTarget)}
          className="flex flex-col gap-2"
        >
          <div className="flex items-center gap-2">
            <RadioGroup.Item
              value="config-schema"
              className="w-4 h-4 rounded-full border border-white/40 data-[state=checked]:border-indigo-400 data-[state=checked]:bg-indigo-400/20"
            >
              <RadioGroup.Indicator className="flex items-center justify-center w-full h-full after:block after:w-2 after:h-2 after:rounded-full after:bg-indigo-400" />
            </RadioGroup.Item>
            <label className="text-sm text-white/80">設定スキーマ (config-schema.yaml)</label>
          </div>
          <div className="flex items-center gap-2">
            <RadioGroup.Item
              value="navigation"
              className="w-4 h-4 rounded-full border border-white/40 data-[state=checked]:border-indigo-400 data-[state=checked]:bg-indigo-400/20"
            >
              <RadioGroup.Indicator className="flex items-center justify-center w-full h-full after:block after:w-2 after:h-2 after:rounded-full after:bg-indigo-400" />
            </RadioGroup.Item>
            <label className="text-sm text-white/80">ナビゲーション定義 (navigation.yaml)</label>
          </div>
        </RadioGroup.Root>
      </div>

      <div className="mb-6">
        <label className="block text-sm font-medium text-white/90 mb-1">ファイルパス</label>
        <input
          type="text"
          value={filePath}
          onChange={(e) => setFilePath(e.target.value)}
          className="w-full bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-indigo-400"
          data-testid="input-file-path"
        />
      </div>

      <button
        onClick={handleValidate}
        disabled={status === 'loading' || !filePath}
        className="bg-indigo-500/80 hover:bg-indigo-500 text-white px-5 py-2.5 rounded-xl transition-all duration-200 shadow-lg shadow-indigo-500/20 disabled:opacity-40"
        data-testid="btn-validate"
      >
        {status === 'loading' ? '検証中...' : '検証を実行'}
      </button>

      {status === 'success' && (
        <div className="mt-4" data-testid="validate-result">
          {errorCount === 0 ? (
            <p className="text-emerald-400 text-sm">✓ バリデーション成功: エラーなし</p>
          ) : (
            <p className="text-rose-400 text-sm">✗ バリデーション失敗: {errorCount} 件のエラー（コンソールを確認してください）</p>
          )}
        </div>
      )}

      {status === 'error' && (
        <p className="text-rose-400 mt-3 text-sm" data-testid="error-message">{errorMessage}</p>
      )}
    </div>
  );
}
