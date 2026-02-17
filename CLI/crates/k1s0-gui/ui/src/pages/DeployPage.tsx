import { useState, useEffect, useCallback } from 'react';
import {
  executeDeployWithProgress,
  scanDeployableTargets,
  type Environment,
  type ProgressEvent,
} from '../lib/tauri-commands';
import ProgressLog from '../components/ProgressLog';
import * as RadioGroup from '@radix-ui/react-radio-group';
import * as Checkbox from '@radix-ui/react-checkbox';

export default function DeployPage() {
  const [targets, setTargets] = useState<string[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [environment, setEnvironment] = useState<Environment>('Dev');
  const [prodConfirm, setProdConfirm] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    scanDeployableTargets('.').then(setTargets).catch(() => {});
  }, []);

  const toggleTarget = (t: string) => {
    setSelected((prev) => prev.includes(t) ? prev.filter((x) => x !== t) : [...prev, t]);
  };

  const handleProgress = useCallback((event: ProgressEvent) => {
    setEvents((prev) => [...prev, event]);
    if (event.kind === 'StepStarted') {
      setCurrentStep(event.step);
      setTotalSteps(event.total);
    } else if (event.kind === 'StepCompleted') {
      setCurrentStep(event.step);
    }
  }, []);

  const handleDeploy = async () => {
    if (environment === 'Prod' && prodConfirm !== 'deploy') {
      setErrorMessage('"deploy" と入力してください。');
      setStatus('error');
      return;
    }
    setStatus('loading');
    setEvents([]);
    setCurrentStep(0);
    setTotalSteps(selected.length);
    try {
      await executeDeployWithProgress({ environment, targets: selected }, handleProgress);
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  return (
    <div className="max-w-lg" data-testid="deploy-page">
      <h1 className="text-2xl font-bold mb-6">デプロイ</h1>

      <div className="mb-4">
        <h2 className="font-semibold mb-2">デプロイ先環境</h2>
        <RadioGroup.Root value={environment} onValueChange={(v) => setEnvironment(v as Environment)}>
          {(['Dev', 'Staging', 'Prod'] as Environment[]).map((env) => (
            <div key={env} className="flex items-center gap-2 mb-1">
              <RadioGroup.Item value={env} id={`env-${env}`}>
                <RadioGroup.Indicator />
              </RadioGroup.Item>
              <label htmlFor={`env-${env}`}>{env.toLowerCase()}</label>
            </div>
          ))}
        </RadioGroup.Root>
      </div>

      <div className="mb-4">
        <h2 className="font-semibold mb-2">デプロイ対象</h2>
        {targets.length === 0 ? (
          <p className="text-gray-500 text-sm">デプロイ対象が見つかりません。</p>
        ) : (
          targets.map((t) => (
            <div key={t} className="flex items-center gap-2 mb-1">
              <Checkbox.Root
                checked={selected.includes(t)}
                onCheckedChange={() => toggleTarget(t)}
              >
                <Checkbox.Indicator><span>✓</span></Checkbox.Indicator>
              </Checkbox.Root>
              <label className="text-sm">{t}</label>
            </div>
          ))
        )}
      </div>

      {environment === 'Prod' && (
        <div className="mb-4 p-3 bg-yellow-50 border border-yellow-200 rounded" data-testid="prod-confirm">
          <p className="text-sm text-yellow-800 mb-2">本番環境へのデプロイです。"deploy" と入力してください。</p>
          <input
            value={prodConfirm}
            onChange={(e) => setProdConfirm(e.target.value)}
            className="w-full border rounded px-3 py-2"
            placeholder="deploy"
            data-testid="input-prod-confirm"
          />
        </div>
      )}

      <button
        onClick={handleDeploy}
        disabled={status === 'loading' || selected.length === 0}
        className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
        data-testid="btn-deploy"
      >
        {status === 'loading' ? 'デプロイ中...' : 'デプロイ'}
      </button>

      {status === 'success' && <p className="text-green-600 mt-2" data-testid="success-message">デプロイが完了しました。</p>}
      {status === 'error' && <p className="text-red-500 mt-2" data-testid="error-message">{errorMessage}</p>}

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
