import { useState, useEffect, useCallback } from 'react';
import {
  executeTestWithProgress,
  scanTestableTargets,
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
    scanTestableTargets('.').then(setTargets).catch(() => {});
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
    <div className="glass p-6 max-w-lg" data-testid="test-page">
      <h1 className="text-2xl font-bold mb-6 text-white">テスト実行</h1>

      <div className="mb-4">
        <h2 className="font-semibold mb-3 text-white/90">テスト種別</h2>
        <RadioGroup.Root value={kind} onValueChange={(v) => setKind(v as TestKind)}>
          {(['Unit', 'Integration', 'All'] as TestKind[]).map((k) => (
            <div key={k} className="flex items-center gap-2 mb-1">
              <RadioGroup.Item value={k}>
                <RadioGroup.Indicator />
              </RadioGroup.Item>
              <label className="text-sm text-white/80">{k}</label>
            </div>
          ))}
        </RadioGroup.Root>
      </div>

      {kind !== 'All' && (
        <div className="mb-4">
          <h2 className="font-semibold mb-3 text-white/90">テスト対象</h2>
          {targets.length === 0 ? (
            <p className="text-white/40 text-sm">テスト対象が見つかりません。</p>
          ) : (
            targets.map((t) => (
              <div key={t} className="flex items-center gap-2 mb-1">
                <Checkbox.Root
                  checked={selected.includes(t)}
                  onCheckedChange={() => toggleTarget(t)}
                >
                  <Checkbox.Indicator><span>✓</span></Checkbox.Indicator>
                </Checkbox.Root>
                <label className="text-sm text-white/80">{t}</label>
              </div>
            ))
          )}
        </div>
      )}

      <button
        onClick={handleTest}
        disabled={status === 'loading' || (kind !== 'All' && selected.length === 0)}
        className="bg-indigo-500/80 hover:bg-indigo-500 text-white px-5 py-2.5 rounded-xl transition-all duration-200 shadow-lg shadow-indigo-500/20 hover:shadow-indigo-500/30 disabled:opacity-40"
        data-testid="btn-test"
      >
        {status === 'loading' ? 'テスト中...' : 'テスト実行'}
      </button>

      {status === 'success' && <p className="text-emerald-400 mt-3" data-testid="success-message">テスト実行が完了しました。</p>}
      {status === 'error' && <p className="text-rose-400 mt-3" data-testid="error-message">{errorMessage}</p>}

      {(status === 'loading' || events.length > 0) && (
        <ProgressLog events={events} currentStep={currentStep} totalSteps={totalSteps} />
      )}
    </div>
  );
}
