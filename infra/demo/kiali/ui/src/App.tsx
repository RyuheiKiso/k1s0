import { useState } from "react";
import ScenarioPanel, { getRecommendedTab } from "./components/ScenarioPanel";
import type { RecommendedTab } from "./components/ScenarioPanel";
import DashboardViewer from "./components/DashboardViewer";
import LiveLog from "./components/LiveLog";
import StatusBar from "./components/StatusBar";
import { useWebSocket } from "./hooks/useWebSocket";
import { useScenario } from "./hooks/useScenario";

const WS_URL = `${window.location.protocol === "https:" ? "wss:" : "ws:"}//${window.location.host}/ws/logs`;

export default function App() {
  const { logs, connected, clearLogs } = useWebSocket(WS_URL);
  const { activeScenario, loading, applyScenario, resetScenario } =
    useScenario();
  const [recommendedTab, setRecommendedTab] =
    useState<RecommendedTab>("kiali");

  const handleSelect = (name: string) => {
    setRecommendedTab(getRecommendedTab(name));
    applyScenario(name);
  };

  const handleReset = () => {
    setRecommendedTab("kiali");
    resetScenario();
  };

  return (
    <div className="h-screen flex flex-col">
      {/* Header */}
      <header className="flex items-center gap-3 px-4 py-2.5 bg-slate-900 border-b border-slate-700">
        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-400 flex items-center justify-center text-white font-bold text-sm">
          k1
        </div>
        <h1 className="text-base font-semibold text-slate-200">
          k1s0 Service Mesh Demo
        </h1>
        <span className="text-xs text-slate-500 ml-2">
          Istio + Kiali + Jaeger + Grafana
        </span>
      </header>

      {/* Main content */}
      <div className="flex-1 flex min-h-0">
        {/* Left panel: Scenarios + Logs */}
        <div className="w-72 flex flex-col border-r border-slate-700 bg-slate-900/50">
          <div className="flex-1 overflow-y-auto p-3">
            <ScenarioPanel
              activeScenario={activeScenario}
              loading={loading}
              onSelect={handleSelect}
              onReset={handleReset}
            />
          </div>
          <div className="h-56 border-t border-slate-700">
            <LiveLog logs={logs} connected={connected} onClear={clearLogs} />
          </div>
        </div>

        {/* Right panel: Dashboard viewer */}
        <div className="flex-1 min-w-0">
          <DashboardViewer recommendedTab={recommendedTab} />
        </div>
      </div>

      {/* Status bar */}
      <StatusBar activeScenario={activeScenario} />
    </div>
  );
}
