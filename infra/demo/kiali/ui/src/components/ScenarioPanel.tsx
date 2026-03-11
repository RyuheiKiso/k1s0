interface ScenarioPanelProps {
  activeScenario: string | null;
  loading: boolean;
  onSelect: (name: string) => void;
  onReset: () => void;
}

const scenarios = [
  {
    id: "normal",
    label: "Normal Traffic",
    desc: "7 services, healthy communication",
    group: "traffic",
    tab: "kiali",
  },
  {
    id: "canary",
    label: "Canary Release",
    desc: "order-server v1:v2 = 90:10",
    group: "traffic",
    tab: "kiali",
  },
  {
    id: "header",
    label: "Header Routing",
    desc: "x-canary:true routes to canary",
    group: "traffic",
    tab: "kiali",
  },
  {
    id: "mirror",
    label: "Traffic Mirroring",
    desc: "10% shadow copy to canary",
    group: "traffic",
    tab: "kiali",
  },
  {
    id: "fault",
    label: "Fault Injection",
    desc: "500ms delay + 503 abort",
    group: "traffic",
    tab: "grafana",
  },
  {
    id: "tracing",
    label: "Distributed Tracing",
    desc: "View request trace chain",
    group: "observability",
    tab: "jaeger",
  },
  {
    id: "logs",
    label: "Log Aggregation",
    desc: "Grafana + Loki structured logs",
    group: "observability",
    tab: "grafana",
  },
  {
    id: "kafka",
    label: "Kafka Messaging",
    desc: "producer -> kafka -> consumer flow",
    group: "observability",
    tab: "grafana",
  },
] as const;

export type RecommendedTab = "kiali" | "jaeger" | "grafana" | "topology";

export function getRecommendedTab(scenarioId: string): RecommendedTab {
  const scenario = scenarios.find((s) => s.id === scenarioId);
  return (scenario?.tab as RecommendedTab) ?? "kiali";
}

export default function ScenarioPanel({
  activeScenario,
  loading,
  onSelect,
  onReset,
}: ScenarioPanelProps) {
  const trafficScenarios = scenarios.filter((s) => s.group === "traffic");
  const observabilityScenarios = scenarios.filter(
    (s) => s.group === "observability"
  );

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">
          Traffic Control
        </h3>
        <div className="flex flex-col gap-1.5">
          {trafficScenarios.map((s) => (
            <button
              key={s.id}
              onClick={() => onSelect(s.id)}
              disabled={loading}
              className={`text-left px-3 py-2 rounded-lg transition-all text-sm ${
                activeScenario === s.id
                  ? "bg-blue-600 text-white shadow-lg shadow-blue-600/20"
                  : "bg-slate-800 text-slate-300 hover:bg-slate-700"
              } disabled:opacity-50`}
            >
              <div className="font-medium">{s.label}</div>
              <div
                className={`text-xs mt-0.5 ${activeScenario === s.id ? "text-blue-200" : "text-slate-500"}`}
              >
                {s.desc}
              </div>
            </button>
          ))}
        </div>
      </div>

      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">
          Observability
        </h3>
        <div className="flex flex-col gap-1.5">
          {observabilityScenarios.map((s) => (
            <button
              key={s.id}
              onClick={() => onSelect(s.id)}
              disabled={loading}
              className={`text-left px-3 py-2 rounded-lg transition-all text-sm ${
                activeScenario === s.id
                  ? "bg-emerald-600 text-white shadow-lg shadow-emerald-600/20"
                  : "bg-slate-800 text-slate-300 hover:bg-slate-700"
              } disabled:opacity-50`}
            >
              <div className="font-medium">{s.label}</div>
              <div
                className={`text-xs mt-0.5 ${activeScenario === s.id ? "text-emerald-200" : "text-slate-500"}`}
              >
                {s.desc}
              </div>
            </button>
          ))}
        </div>
      </div>

      <div className="pt-2 border-t border-slate-700">
        <button
          onClick={onReset}
          disabled={loading}
          className="w-full px-3 py-2 rounded-lg bg-slate-800 text-slate-400 hover:bg-red-900/50 hover:text-red-300 transition-all text-sm font-medium disabled:opacity-50"
        >
          Reset All
        </button>
      </div>
    </div>
  );
}
