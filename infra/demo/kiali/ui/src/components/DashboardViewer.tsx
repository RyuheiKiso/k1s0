import { useState, useEffect } from "react";
import type { RecommendedTab } from "./ScenarioPanel";

interface DashboardViewerProps {
  recommendedTab: RecommendedTab;
}

const tabs = [
  {
    id: "kiali" as const,
    label: "Kiali",
    url: "http://localhost:20001/kiali/console/graph/namespaces/?namespaces=k1s0-system,k1s0-business,k1s0-service&graphType=versionedApp&duration=60&refresh=15000&animation=true",
    color: "text-blue-400 border-blue-400",
  },
  {
    id: "jaeger" as const,
    label: "Jaeger",
    url: "http://localhost:16686/search",
    color: "text-cyan-400 border-cyan-400",
  },
  {
    id: "grafana" as const,
    label: "Grafana",
    url: "http://localhost:3000/d/k1s0-mesh-overview?orgId=1&refresh=10s",
    color: "text-orange-400 border-orange-400",
  },
  {
    id: "topology" as const,
    label: "Topology",
    url: "http://localhost:3000/d/k1s0-service-topology?orgId=1&refresh=10s",
    color: "text-purple-400 border-purple-400",
  },
];

export default function DashboardViewer({
  recommendedTab,
}: DashboardViewerProps) {
  const [activeTab, setActiveTab] = useState<string>("kiali");

  useEffect(() => {
    setActiveTab(recommendedTab);
  }, [recommendedTab]);

  const currentTab = tabs.find((t) => t.id === activeTab) ?? tabs[0];

  return (
    <div className="flex flex-col h-full">
      <div className="flex border-b border-slate-700">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium transition-all border-b-2 ${
              activeTab === tab.id
                ? `${tab.color} bg-slate-800/50`
                : "text-slate-500 border-transparent hover:text-slate-300"
            }`}
          >
            {tab.label}
          </button>
        ))}
        <a
          href={currentTab.url}
          target="_blank"
          rel="noopener noreferrer"
          className="ml-auto px-3 py-2 text-xs text-slate-500 hover:text-slate-300 transition-colors self-center"
        >
          Open in new tab
        </a>
      </div>
      <div className="flex-1 relative">
        {tabs.map((tab) => (
          <iframe
            key={tab.id}
            src={tab.url}
            title={tab.label}
            className={`absolute inset-0 w-full h-full border-0 ${
              activeTab === tab.id ? "block" : "hidden"
            }`}
          />
        ))}
      </div>
    </div>
  );
}
