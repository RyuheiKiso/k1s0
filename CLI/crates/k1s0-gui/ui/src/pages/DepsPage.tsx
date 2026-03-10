import { useEffect, useState } from 'react';
import { executeDeps, scanServices, type DepsConfig, type DepsResult, type ServiceInfo } from '../lib/tauri-commands';
import { toDisplayPath } from '../lib/paths';
import { useWorkspace } from '../lib/workspace';

type ScopeMode = 'all' | 'tier' | 'services';
type OutputMode = 'terminal' | 'mermaid' | 'both';

export default function DepsPage() {
  const workspace = useWorkspace();
  const activeWorkspaceRoot = workspace.workspaceRoot || '.';
  const workspaceUnavailable = workspace.ready && !workspace.workspaceRoot;

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

    if (workspaceUnavailable) {
      setServices([]);
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
  }, [activeWorkspaceRoot, workspaceUnavailable]);

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
          : { Services: selectedServices };

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
    <div className="glass max-w-6xl p-6" data-testid="deps-page">
      <p className="text-xs uppercase tracking-[0.24em] text-emerald-100/55">Architecture</p>
      <h1 className="mt-2 text-3xl font-semibold text-white">Inspect dependency map</h1>
      <p className="mt-3 text-sm leading-7 text-slate-200/76">
        Run the dependency scan for the selected workspace and optionally export Mermaid output.
      </p>

      {workspaceUnavailable && (
        <p className="mt-5 rounded-2xl border border-amber-400/25 bg-amber-400/10 px-4 py-3 text-sm text-amber-100">
          Configure a valid workspace root before running the dependency scan.
        </p>
      )}

      <div className="mt-6 grid gap-6 lg:grid-cols-[0.9fr_1.1fr]">
        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <div className="space-y-5">
            <fieldset className="space-y-2">
              <legend className="text-sm font-medium text-slate-200/82">Scope</legend>
              {(['all', 'tier', 'services'] as ScopeMode[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={scopeMode === value}
                    onChange={() => setScopeMode(value)}
                    name="deps-scope"
                  />
                  {value === 'all'
                    ? 'All services'
                    : value === 'tier'
                      ? 'Single tier'
                      : 'Selected services'}
                </label>
              ))}
            </fieldset>

            {scopeMode === 'tier' && (
              <fieldset className="space-y-2">
                <legend className="text-sm font-medium text-slate-200/82">Tier</legend>
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
                <p className="text-sm font-medium text-slate-200/82">Services</p>
                <div className="mt-3 max-h-64 space-y-2 overflow-auto pr-1">
                  {services.length === 0 ? (
                    <p className="text-sm text-slate-200/55">No services were found.</p>
                  ) : (
                    services.map((service) => (
                      <label
                        key={service.path}
                        className="flex items-center gap-3 rounded-xl border border-white/8 bg-slate-950/20 px-3 py-2 text-sm text-slate-100"
                      >
                        <input
                          type="checkbox"
                          checked={selectedServices.includes(service.name)}
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
              <legend className="text-sm font-medium text-slate-200/82">Output</legend>
              {(['terminal', 'mermaid', 'both'] as OutputMode[]).map((value) => (
                <label key={value} className="flex items-center gap-3 text-sm text-slate-200/82">
                  <input
                    type="radio"
                    checked={outputMode === value}
                    onChange={() => setOutputMode(value)}
                    name="deps-output"
                  />
                  {value === 'terminal'
                    ? 'Terminal summary'
                    : value === 'mermaid'
                      ? 'Mermaid file'
                      : 'Terminal + Mermaid'}
                </label>
              ))}
            </fieldset>

            {outputMode !== 'terminal' && (
              <div>
                <label className="block text-sm font-medium text-slate-200/82">Mermaid path</label>
                <input
                  value={mermaidPath}
                  onChange={(event) => setMermaidPath(event.target.value)}
                  className="mt-2 w-full rounded-xl border border-white/15 bg-white/6 px-3 py-2 text-white"
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
              Disable dependency cache
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
              (scopeMode === 'services' && selectedServices.length === 0) ||
              (outputMode !== 'terminal' && !mermaidPath)
            }
            className="mt-6 rounded-xl bg-emerald-500/85 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
            data-testid="btn-run-deps"
          >
            {status === 'loading' ? 'Scanning...' : 'Run dependency scan'}
          </button>

          {status === 'error' && (
            <p className="mt-4 text-sm text-rose-300" data-testid="error-message">
              {errorMessage}
            </p>
          )}
        </section>

        <section className="rounded-2xl border border-white/10 bg-white/5 p-5">
          <h2 className="text-lg font-semibold text-white">Result</h2>
          {!result ? (
            <p className="mt-4 text-sm text-slate-200/55">
              Run the scan to inspect services, dependencies, and rule violations.
            </p>
          ) : (
            <div className="mt-4 space-y-5">
              <div className="grid gap-3 sm:grid-cols-3">
                <SummaryCard label="Services" value={String(result.services.length)} />
                <SummaryCard label="Dependencies" value={String(result.dependencies.length)} />
                <SummaryCard label="Violations" value={String(result.violations.length)} />
              </div>

              <div>
                <p className="text-sm font-medium text-slate-200/82">Dependencies</p>
                <div className="mt-3 max-h-72 space-y-2 overflow-auto pr-1">
                  {result.dependencies.length === 0 ? (
                    <p className="text-sm text-slate-200/55">No dependencies were found.</p>
                  ) : (
                    result.dependencies.map((dependency) => (
                      <div
                        key={`${dependency.source}-${dependency.target}-${dependency.dep_type}-${dependency.locations.join(',')}`}
                        className="rounded-xl border border-white/8 bg-slate-950/20 px-3 py-3 text-sm text-slate-100"
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
                <p className="text-sm font-medium text-slate-200/82">Violations</p>
                <div className="mt-3 max-h-72 space-y-2 overflow-auto pr-1">
                  {result.violations.length === 0 ? (
                    <p className="text-sm text-emerald-300">No violations were found.</p>
                  ) : (
                    result.violations.map((violation) => (
                      <div
                        key={`${violation.source}-${violation.target}-${violation.message}`}
                        className="rounded-xl border border-white/8 bg-slate-950/20 px-3 py-3 text-sm text-slate-100"
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
                  Mermaid output written to {toDisplayPath(activeWorkspaceRoot, mermaidPath)}.
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
    <div className="rounded-2xl border border-white/10 bg-slate-950/20 p-4">
      <p className="text-xs uppercase tracking-[0.24em] text-slate-200/55">{label}</p>
      <p className="mt-3 text-2xl font-semibold text-white">{value}</p>
    </div>
  );
}
