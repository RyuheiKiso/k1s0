import { useState, useEffect, useCallback } from 'react';
import {
  executeTestWithProgress,
  scanTestableTargets,
  scanE2eSuites,
  type TestKind,
  type ProgressEvent,
} from '../lib/tauri-commands';
import ProgressLog from '../components/ProgressLog';
import * as RadioGroup from '@radix-ui/react-radio-group';
import * as Checkbox from '@radix-ui/react-checkbox';

export default function TestPage() {
  const [targets, setTargets] = useState<string[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [kind, setKind] = useState<TestKind>('Unit');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [events, setEvents] = useState<ProgressEvent[]>([]);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    setSelected([]);
    if (kind === 'All') {
      setTargets([]);
      return;
    }
    if (kind === 'E2e') {
      scanE2eSuites('.').then(setTargets).catch(() => {});
    } else {
      scanTestableTargets('.').then(setTargets).catch(() => {});
    }
  }, [kind]);

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

  const handleTest = async () => {
    setStatus('loading');
    setEvents([]);
    setCurrentStep(0);
    setTotalSteps(kind === 'All' ? 0 : selected.length);
    try {
      await executeTestWithProgress({ kind, targets: kind === 'All' ? [] : selected }, handleProgress);
      setStatus('success');
    } catch (e) {
      setStatus('error');
      setErrorMessage(String(e));
    }
  };

  return (
    <div className="max-w-lg" data-testid="test-page">
      <h1 className="text-2xl font-bold mb-6">テスト実行</h1>

      <div className="mb-4">
        <h2 className="font-semibold mb-2">テスト種別</h2>
        <RadioGroup.Root value={kind} onValueChange={(v) => setKind(v as TestKind)}>
          {(['Unit', 'Integration', 'E2e', 'All'] as TestKind[]).map((k) => (
            <div key={k} className="flex items-center gap-2 mb-1">
              <RadioGroup.Item value={k}>
                <RadioGroup.Indicator />
              </RadioGroup.Item>
              <label>{k}</label>
            </div>
          ))}
        </RadioGroup.Root>
      </div>

      {kind !== 'All' && (
        <div className="mb-4">
          <h2 className="font-semibold mb-2">テスト対象</h2>
          {targets.length === 0 ? (
            <p className="text-gray-500 text-sm">テスト対象が見つかりません。</p>
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
      )}

      <button
        onClick={handleTest}
        disabled={status === 'loading' || (kind !== 'All' && selected.length === 0)}
        className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
        data-testid="btn-test"
      >
        {status === 'loading' ? 'テスト中...' : 'テスト実行'}
      </button>

      {status === 'success' && <p className="text-green-600 mt-2" data-testid="success-message">テスト実行が完了しました。</p>}
      {status === 'error' && <p className="text-red-500 mt-2" data-testid="error-message">{errorMessage}</p>}

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
