import { useState, useEffect } from "react";
import type { RecommendedTab } from "./ScenarioPanel";
import TopologyView from "./TopologyView";

interface DashboardViewerProps {
  recommendedTab: RecommendedTab;
}

const iframeTabs = [
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
    url: "http://localhost:3000/d/k1s0-mesh-overview/k1s0-service-mesh-overview?orgId=1&refresh=10s",
    color: "text-orange-400 border-orange-400",
  },
];

const allTabs = [
  ...iframeTabs,
  {
    id: "topology" as const,
    label: "Topology",
    url: "",
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

  const currentTab = allTabs.find((t) => t.id === activeTab) ?? allTabs[0];
  const isTopology = activeTab === "topology";

  return (
    <div className="flex flex-col h-full">
      <div className="flex border-b border-slate-700">
        {allTabs.map((tab) => (
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
        {!isTopology && currentTab.url && (
          <a
            href={currentTab.url}
            target="_blank"
            rel="noopener noreferrer"
            className="ml-auto px-3 py-2 text-xs text-slate-500 hover:text-slate-300 transition-colors self-center"
          >
            Open in new tab
          </a>
        )}
      </div>
      <div className="flex-1 relative">
        {/* Iframe tabs */}
        {iframeTabs.map((tab) => (
          <iframe
            key={tab.id}
            src={tab.url}
            title={tab.label}
            className={`absolute inset-0 w-full h-full border-0 ${
              activeTab === tab.id ? "block" : "hidden"
            }`}
          />
        ))}
        {/* Topology tab (React component) */}
        {isTopology && <TopologyView />}
      </div>
    </div>
  );
}
