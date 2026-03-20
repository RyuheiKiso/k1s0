import { useEffect, useState } from 'react';
import ProtectedActionNotice from '../components/ProtectedActionNotice';
import { useAuth } from '../lib/auth';
import { executeDeps, scanServices, type DepsConfig, type DepsResult, type ServiceInfo } from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

type ScopeMode = 'all' | 'tier' | 'services';
type OutputMode = 'terminal' | 'mermaid' | 'both';

export default function DepsPage() {
  const auth = useAuth();
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;
  const actionsLocked = auth.loading || !auth.isAuthenticated;

  const [services, setServices] = useState<ServiceInfo[]>([]);
  const [scopeMode, setScopeMode] = useState<ScopeMode>('all');
  const [tier, setTier] = useState('system');
  const [selectedServices, setSelectedServices] = useState<string[]>([]);
  const [outputMode, setOutputMode] = useState<OutputMode>('terminal');
  const [mermaidPath, setMermaidPath] = useState('docs/diagrams/dependency-map.md');
  const [noCache, setNoCache] = useState(false);
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');
  const [result, setResult] = useState<DepsResult | null>(null);

  useEffect(() => {
    let cancelled = false;

    if (!workspace.ready || !workspace.workspaceRoot) {
      return;
    }

    scanServices(activeWorkspaceRoot)
      .then((nextServices) => {
        if (!cancelled) {
          setServices(nextServices);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setServices([]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceRoot, workspace.ready, workspace.workspaceRoot]);

  const availableServices = workspace.ready && workspace.workspaceRoot ? services : [];
  const selectedServiceNames = selectedServices.filter((name) =>
    availableServices.some((service) => service.name === name),
  );

  function toggleService(name: string) {
    setSelectedServices((current) =>
      current.includes(name)
        ? current.filter((value) => value !== name)
        : [...current, name],
    );
  }

  function buildConfig(): DepsConfig {
    const scope =
      scopeMode === 'all'
          ? 'All'
        : scopeMode === 'tier'
          ? { Tier: tier }
          : { Services: selectedServiceNames };

    const output =
      outputMode === 'terminal'
        ? 'Terminal'
        : outputMode === 'mermaid'
          ? { Mermaid: mermaidPath }
          : { Both: mermaidPath };

    return {
      scope,
      output,
      no_cache: noCache,
    };
  }

  async function handleRun() {
    setStatus('loading');
    setErrorMessage('');

    try {
      const nextResult = await executeDeps(buildConfig(), activeWorkspaceRoot);
      setResult(nextResult);
      setStatus('success');
    } catch (error) {
      setStatus('error');
      setErrorMessage(String(error));
    }
  }

  return (
    <div className="glass max-w-6xl p-6 p3-animate-in" data-testid="deps-page">
      <p className="text-xs uppercase tracking-[0.24em] text-cyan-100/55">アーキテクチャ</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">依存関係マップの検査</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        選択したワークスペースの依存関係スキャンを実行し、オプションでMermaid出力をエクスポートします。
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 border border-red-400/25 bg-red-400/10 px-4 py-3 text-sm text-red-100">
          依存関係スキャンを実行する前に有効なワークスペースルートを設定してください。
        </p>
      )}
      {actionsLocked && <ProtectedActionNotice loading={auth.loading} />}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <div className="space-y-5">
            <fieldset className="space-y-2">
              <legend className="text-sm font-medium text-slate-200/82">スコープ</legend>
              {(['all', 'tier', 'services'] as ScopeMode[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={scopeMode === value}
                    onChange={() => setScopeMode(value)}
                    name="deps-scope"
                  />
                  {value === 'all'
                    ? '全サービス'
                    : value === 'tier'
                      ? '単一ティア'
                      : '選択されたサービス'}
                </label>
              ))}
            </fieldset>

            {scopeMode === 'tier' && (
              <fieldset className="space-y-2">
                <legend className="text-sm font-medium text-slate-200/82">ティア</legend>
                {(['system', 'business', 'service'] as const).map((value) => (
                  <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                    <input
                      type="radio"
                      checked={tier === value}
                      onChange={() => setTier(value)}
                      name="deps-tier"
                    />
                    {value}
                  </label>
                ))}
              </fieldset>
            )}

            {scopeMode === 'services' && (
              <div>
                <p className="text-sm font-medium text-slate-200/82">サービス</p>
                <div className="mt-3 max-h-64 space-y-2 overflow-auto pr-1">
                  {availableServices.length === 0 ? (
                    <p className="text-sm text-slate-200/55">サービスが見つかりませんでした。</p>
                  ) : (
                    availableServices.map((service) => (
                      <label
                        key={service.path}
                        className="flex items-center gap-3 border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-2 text-sm text-slate-100"
                      >
                        <input
                          type="checkbox"
                          checked={selectedServiceNames.includes(service.name)}
                          onChange={() => toggleService(service.name)}
                        />
                        <span>{service.name}</span>
                        <span className="text-slate-400/80">{service.tier}</span>
                      </label>
                    ))
                  )}
                </div>
              </div>
            )}

            <fieldset className="space-y-2">
              <legend className="text-sm font-medium text-slate-200/82">出力</legend>
              {(['terminal', 'mermaid', 'both'] as OutputMode[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={outputMode === value}
                    onChange={() => setOutputMode(value)}
                    name="deps-output"
                  />
                  {value === 'terminal'
                    ? 'ターミナルサマリー'
                    : value === 'mermaid'
                      ? 'Mermaidファイル'
                      : 'ターミナル + Mermaid'}
                </label>
              ))}
            </fieldset>

            {outputMode !== 'terminal' && (
              <div>
                <label className="block text-sm font-medium text-slate-200/82">Mermaidパス</label>
                <input
                  value={mermaidPath}
                  onChange={(event) => setMermaidPath(event.target.value)}
                  className="mt-2 w-full border border-[rgba(0,200,255,0.15)] bg-[rgba(0,200,255,0.04)] px-3 py-2 text-white"
                  data-testid="input-mermaid-path"
                />
              </div>
            )}

            <label className="flex items-center gap-3 text-sm text-slate-200/82">
              <input
                type="checkbox"
                checked={noCache}
                onChange={(event) => setNoCache(event.target.checked)}
              />
              依存関係キャッシュを無効化
            </label>
          </div>

          <button
            type="button"
            onClick={() => {
              void handleRun();
            }}
            disabled={
              status === 'loading' ||
              workspaceUnavailable ||
              actionsLocked ||
              (scopeMode === 'services' && selectedServiceNames.length === 0) ||
              (outputMode !== 'terminal' && !mermaidPath)
            }
            className="mt-6 bg-cyan-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-cyan-500 disabled:opacity-50"
            data-testid="btn-run-deps"
          >
            {status === 'loading' ? 'スキャン中...' : '依存関係スキャンを実行'}
          </button>

          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="border border-[rgba(0,200,255,0.12)] bg-[rgba(0,200,255,0.03)] p-5">
          <h2 className="text-lg font-semibold text-white">結果</h2>
          {!result ? (
            <p className="mt-4 text-sm text-slate-200/55">
              スキャンを実行してサービス、依存関係、ルール違反を検査します。
            </p>
          ) : (
            <div className="mt-4 space-y-5">
              <div className="grid gap-3 sm:grid-cols-3">
                <SummaryCard label="サービス" value={String(result.services.length)} />
                <SummaryCard label="依存関係" value={String(result.dependencies.length)} />
                <SummaryCard label="違反" value={String(result.violations.length)} />
              </div>

              <div>
                <p className="text-sm font-medium text-slate-200/82">依存関係</p>
                <div className="mt-3 max-h-72 space-y-2 overflow-auto pr-1">
                  {result.dependencies.length === 0 ? (
                    <p className="text-sm text-slate-200/55">依存関係が見つかりませんでした。</p>
                  ) : (
                    result.dependencies.map((dependency) => (
                      <div
                        key={`${dependency.source}-${dependency.target}-${dependency.dep_type}-${dependency.locations.join(',')}`}
                        className="border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-3 text-sm text-slate-100"
                      >
                        <p>
                          {dependency.source} {'->'} {dependency.target} ({dependency.dep_type})
                        </p>
                        {dependency.detail && (
                          <p className="mt-1 text-slate-300/70">{dependency.detail}</p>
                        )}
                      </div>
                    ))
                  )}
                </div>
              </div>

              <div>
                <p className="text-sm font-medium text-slate-200/82">違反</p>
                <div className="mt-3 max-h-72 space-y-2 overflow-auto pr-1">
                  {result.violations.length === 0 ? (
                    <p className="text-sm text-cyan-300">違反は見つかりませんでした。</p>
                  ) : (
                    result.violations.map((violation) => (
                      <div
                        key={`${violation.source}-${violation.target}-${violation.message}`}
                        className="border border-[rgba(0,200,255,0.10)] bg-[rgba(5,8,15,0.20)] px-3 py-3 text-sm text-slate-100"
                      >
                        <p>
                          [{violation.severity}] {violation.source} {'->'} {violation.target}
                        </p>
                        <p className="mt-1 text-slate-300/80">{violation.message}</p>
                        <p className="mt-1 text-slate-400/70">{violation.recommendation}</p>
                      </div>
                    ))
                  )}
                </div>
              </div>

              {outputMode !== 'terminal' && (
                <p className="text-sm text-slate-300/70">
                  Mermaid出力の書き込み先: {toDisplayPath(activeWorkspaceRoot, mermaidPath)}.
                </p>
              )}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="border border-[rgba(0,200,255,0.12)] bg-[rgba(5,8,15,0.20)] p-4">
      <p className="text-xs uppercase tracking-[0.24em] text-slate-200/55">{label}</p>
      <p className="mt-3 text-2xl font-semibold text-white">{value}</p>
    </div>
  );
}
