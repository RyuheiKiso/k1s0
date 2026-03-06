import { useEffect, useState, useCallback } from "react";

interface StatusBarProps {
  activeScenario: string | null;
}

interface ClusterStatus {
  podsReady: number;
  podsTotal: number;
  trafficRunning: boolean;
}

export default function StatusBar({ activeScenario }: StatusBarProps) {
  const [status, setStatus] = useState<ClusterStatus | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      const res = await fetch("/api/status");
      if (res.ok) {
        const data = await res.json();
        setStatus({
          podsReady: data.pods.ready,
          podsTotal: data.pods.total,
          trafficRunning: data.trafficRunning,
        });
      }
    } catch {
      setStatus(null);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 10000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  return (
    <div className="flex items-center gap-4 px-4 py-2 bg-slate-900 border-t border-slate-700 text-xs">
      <div className="flex items-center gap-1.5">
        <span className="text-slate-500">Scenario:</span>
        <span className={activeScenario ? "text-blue-400 font-medium" : "text-slate-400"}>
          {activeScenario ?? "None"}
        </span>
      </div>

      {status && (
        <>
          <div className="flex items-center gap-1.5">
            <span className="text-slate-500">Pods:</span>
            <span
              className={
                status.podsReady === status.podsTotal
                  ? "text-green-400"
                  : "text-yellow-400"
              }
            >
              {status.podsReady}/{status.podsTotal} ready
            </span>
          </div>

          <div className="flex items-center gap-1.5">
            <span className="text-slate-500">Traffic:</span>
            <span
              className={
                status.trafficRunning ? "text-green-400" : "text-slate-400"
              }
            >
              {status.trafficRunning ? "ON" : "OFF"}
            </span>
          </div>
        </>
      )}

      {!status && (
        <span className="text-slate-600">Connecting to cluster...</span>
      )}

      <div className="ml-auto text-slate-600">k1s0 Demo Dashboard</div>
    </div>
  );
}
