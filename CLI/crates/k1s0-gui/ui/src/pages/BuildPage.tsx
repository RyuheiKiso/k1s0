import { useState, useEffect, useCallback } from 'react';
import {
  executeBuildWithProgress,
  scanBuildableTargets,
  type BuildMode,
  type ProgressEvent,
} from '../lib/tauri-commands';
import ProgressLog from '../components/ProgressLog';
import * as RadioGroup from '@radix-ui/react-radio-group';
import * as Checkbox from '@radix-ui/react-checkbox';

export default function BuildPage() {
  const [targets, setTargets] = useState<string[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [mode, setMode] = useState<BuildMode>('Development');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    scanBuildableTargets('.').then(setTargets).catch(() => {});
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

  const handleBuild = async () => {
    setStatus('loading');
    setEvents([]);
    setCurrentStep(0);
    setTotalSteps(selected.length);
    try {
      await executeBuildWithProgress({ targets: selected, mode }, handleProgress);
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  return (
    <div className="max-w-lg" data-testid="build-page">
      <h1 className="text-2xl font-bold mb-6">ビルド</h1>

      <div className="mb-4">
        <h2 className="font-semibold mb-2">ビルド対象</h2>
        {targets.length === 0 ? (
          <p className="text-gray-500 text-sm">ビルド対象が見つかりません。</p>
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

      <div className="mb-4">
        <h2 className="font-semibold mb-2">ビルドモード</h2>
        <RadioGroup.Root value={mode} onValueChange={(v) => setMode(v as BuildMode)}>
          {(['Development', 'Production'] as BuildMode[]).map((m) => (
            <div key={m} className="flex items-center gap-2 mb-1">
              <RadioGroup.Item value={m}>
                <RadioGroup.Indicator />
              </RadioGroup.Item>
              <label>{m.toLowerCase()}</label>
            </div>
          ))}
        </RadioGroup.Root>
      </div>

      <button
        onClick={handleBuild}
        disabled={status === 'loading' || selected.length === 0}
        className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
        data-testid="btn-build"
      >
        {status === 'loading' ? 'ビルド中...' : 'ビルド'}
      </button>

      {status === 'success' && <p className="text-green-600 mt-2" data-testid="success-message">ビルドが完了しました。</p>}
      {status === 'error' && <p className="text-red-500 mt-2" data-testid="error-message">{errorMessage}</p>}

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
