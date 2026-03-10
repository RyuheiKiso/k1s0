import { useEffect, useState, useCallback } from "react";

interface TopoNode {
  id: string;
  namespace: string;
  rate: number;
  unit: "reqps" | "bytesps" | "mixed";
}

interface TopoEdge {
  source: string;
  target: string;
  rate: number;
  errorRate: number;
  protocol: "http" | "tcp";
  unit: "reqps" | "bytesps";
}

interface TopologyData {
  nodes: TopoNode[];
  edges: TopoEdge[];
}

// Namespace color mapping
const NS_COLORS: Record<string, { bg: string; border: string; text: string }> = {
  "k1s0-system":   { bg: "#1e3a5f", border: "#3b82f6", text: "#93c5fd" },
  "k1s0-business": { bg: "#1a3f2e", border: "#22c55e", text: "#86efac" },
  "k1s0-service":  { bg: "#3f1a3f", border: "#a855f7", text: "#d8b4fe" },
  "messaging":     { bg: "#3f2a1a", border: "#f97316", text: "#fdba74" },
};

const DEFAULT_COLOR = { bg: "#1e293b", border: "#64748b", text: "#94a3b8" };

// Layout: tier-based positioning
const TIER_CONFIG: Record<string, { x: number; label: string }> = {
  "k1s0-service":  { x: 100, label: "Service" },
  "k1s0-business": { x: 380, label: "Business" },
  "k1s0-system":   { x: 660, label: "System" },
  "messaging":     { x: 660, label: "Messaging" },
};

function layoutNodes(nodes: TopoNode[]): Map<string, { x: number; y: number }> {
  const positions = new Map<string, { x: number; y: number }>();
  const byNs = new Map<string, TopoNode[]>();

  for (const n of nodes) {
    const ns = n.namespace;
    if (!byNs.has(ns)) byNs.set(ns, []);
    byNs.get(ns)!.push(n);
  }

  for (const [ns, nsNodes] of byNs) {
    const tier = TIER_CONFIG[ns] || { x: 400 };
    const startY = 80;
    const spacing = 90;
    // Sort by rate descending for consistent layout
    nsNodes.sort((a, b) => b.rate - a.rate);
    for (let i = 0; i < nsNodes.length; i++) {
      positions.set(nsNodes[i].id, { x: tier.x, y: startY + i * spacing });
    }
  }

  return positions;
}

function formatRate(rate: number, unit: TopoNode["unit"] | TopoEdge["unit"]): string {
  if (unit === "bytesps") {
    if (rate >= 1024 * 1024) return `${(rate / (1024 * 1024)).toFixed(1)} MiB/s`;
    if (rate >= 1024) return `${(rate / 1024).toFixed(1)} KiB/s`;
    return `${rate.toFixed(0)} B/s`;
  }

  if (unit === "mixed") return "mixed traffic";
  if (rate >= 1) return `${rate.toFixed(1)} req/s`;
  if (rate >= 0.01) return `${(rate * 1000).toFixed(0)} mreq/s`;
  return `${(rate * 1000).toFixed(1)} mreq/s`;
}

export default function TopologyView() {
  const [data, setData] = useState<TopologyData | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchTopology = useCallback(async () => {
    try {
      const res = await fetch("/api/topology");
      if (res.ok) {
        const json = await res.json();
        // Filter out "unknown" workloads
        json.nodes = json.nodes.filter((n: TopoNode) => n.id !== "unknown");
        json.edges = json.edges.filter(
          (e: TopoEdge) => e.source !== "unknown" && e.target !== "unknown"
        );
        setData(json);
        setError(null);
      } else {
        setError("Failed to fetch topology");
      }
    } catch {
      setError("Backend not reachable");
    }
  }, []);

  useEffect(() => {
    fetchTopology();
    const interval = setInterval(fetchTopology, 5000);
    return () => clearInterval(interval);
  }, [fetchTopology]);

  if (error) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500">
        {error}
      </div>
    );
  }

  if (!data || data.nodes.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-slate-500">
        Waiting for traffic data... Start a scenario to see the topology.
      </div>
    );
  }

  const positions = layoutNodes(data.nodes);
  const nodeRadius = 32;

  // Calculate SVG viewBox
  const allX = Array.from(positions.values()).map((p) => p.x);
  const allY = Array.from(positions.values()).map((p) => p.y);
  const minX = Math.min(...allX) - 80;
  const maxX = Math.max(...allX) + 80;
  const minY = Math.min(...allY) - 60;
  const maxY = Math.max(...allY) + 60;
  const svgWidth = maxX - minX;
  const svgHeight = maxY - minY;

  // Max rate for edge thickness scaling
  const maxRate = Math.max(...data.edges.map((e) => e.rate), 0.001);

  return (
    <div className="h-full w-full overflow-auto bg-slate-950 p-4">
      {/* Legend */}
      <div className="flex gap-4 mb-3 text-xs">
        {Object.entries(TIER_CONFIG).map(([ns, cfg]) => {
          const color = NS_COLORS[ns] || DEFAULT_COLOR;
          return (
            <div key={ns} className="flex items-center gap-1.5">
              <div
                className="w-3 h-3 rounded-full border"
                style={{ backgroundColor: color.bg, borderColor: color.border }}
              />
              <span style={{ color: color.text }}>{cfg.label}</span>
            </div>
          );
        })}
        <div className="ml-auto flex items-center gap-3 text-slate-500">
          <span>
            <span className="inline-block w-6 h-0.5 bg-green-500 mr-1 align-middle" />
            healthy
          </span>
          <span>
            <span className="inline-block w-6 h-0.5 bg-red-500 mr-1 align-middle" />
            errors
          </span>
          <span className="text-sky-400">TCP edges show throughput</span>
        </div>
      </div>

      <svg
        viewBox={`${minX} ${minY} ${svgWidth} ${svgHeight}`}
        className="w-full"
        style={{ maxHeight: "calc(100% - 32px)" }}
      >
        <defs>
          <marker
            id="arrowGreen"
            viewBox="0 0 10 6"
            refX="10"
            refY="3"
            markerWidth="8"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 3 L 0 6 z" fill="#22c55e" />
          </marker>
          <marker
            id="arrowRed"
            viewBox="0 0 10 6"
            refX="10"
            refY="3"
            markerWidth="8"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 3 L 0 6 z" fill="#ef4444" />
          </marker>
          <marker
            id="arrowYellow"
            viewBox="0 0 10 6"
            refX="10"
            refY="3"
            markerWidth="8"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <path d="M 0 0 L 10 3 L 0 6 z" fill="#eab308" />
          </marker>
        </defs>

        {/* Edges */}
        {data.edges.map((edge, i) => {
          const src = positions.get(edge.source);
          const dst = positions.get(edge.target);
          if (!src || !dst) return null;

          const hasError = edge.errorRate > 0;
          const errorPct = edge.rate > 0 ? (edge.errorRate / edge.rate) * 100 : 0;
          const isTcp = edge.protocol === "tcp";
          const strokeColor = isTcp
            ? "#38bdf8"
            : hasError
              ? errorPct > 5
                ? "#ef4444"
                : "#eab308"
              : "#22c55e";
          const markerEnd = isTcp
            ? "url(#arrowGreen)"
            : hasError
              ? errorPct > 5
                ? "url(#arrowRed)"
                : "url(#arrowYellow)"
              : "url(#arrowGreen)";
          const thickness = 1 + (edge.rate / maxRate) * 3;

          // Offset for line so it doesn't overlap node circle
          const dx = dst.x - src.x;
          const dy = dst.y - src.y;
          const dist = Math.sqrt(dx * dx + dy * dy) || 1;
          const offsetX = (dx / dist) * (nodeRadius + 6);
          const offsetY = (dy / dist) * (nodeRadius + 6);

          const x1 = src.x + offsetX;
          const y1 = src.y + offsetY;
          const x2 = dst.x - offsetX;
          const y2 = dst.y - offsetY;

          // Midpoint for label
          const mx = (x1 + x2) / 2;
          const my = (y1 + y2) / 2 - 6;

          return (
            <g key={i}>
              <line
                x1={x1}
                y1={y1}
                x2={x2}
                y2={y2}
                stroke={strokeColor}
                strokeWidth={thickness}
                strokeOpacity={0.7}
                markerEnd={markerEnd}
              />
              <text
                x={mx}
                y={my}
                textAnchor="middle"
                fontSize="9"
                fill={strokeColor}
                opacity={0.9}
              >
                {edge.protocol.toUpperCase()} {formatRate(edge.rate, edge.unit)}
              </text>
            </g>
          );
        })}

        {/* Nodes */}
        {data.nodes.map((node) => {
          const pos = positions.get(node.id);
          if (!pos) return null;
          const color = NS_COLORS[node.namespace] || DEFAULT_COLOR;

          return (
            <g key={node.id}>
              <circle
                cx={pos.x}
                cy={pos.y}
                r={nodeRadius}
                fill={color.bg}
                stroke={color.border}
                strokeWidth={2}
                opacity={0.9}
              />
              <text
                x={pos.x}
                y={pos.y - 4}
                textAnchor="middle"
                fontSize="9"
                fontWeight="600"
                fill={color.text}
              >
                {node.id.replace("-server", "").replace("-gateway", "-gw")}
              </text>
              <text
                x={pos.x}
                y={pos.y + 10}
                textAnchor="middle"
                fontSize="8"
                fill={color.text}
                opacity={0.6}
              >
                {formatRate(node.rate, node.unit)}
              </text>
            </g>
          );
        })}
      </svg>
    </div>
  );
}
