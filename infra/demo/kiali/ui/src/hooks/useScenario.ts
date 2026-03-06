import { useState, useCallback } from "react";

interface StatusResponse {
  activeScenario: string | null;
  trafficRunning: boolean;
  pods: {
    ready: number;
    total: number;
  };
}

export function useScenario() {
  const [activeScenario, setActiveScenario] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const applyScenario = useCallback(async (name: string) => {
    setLoading(true);
    try {
      const res = await fetch(`/api/scenario/${name}`, { method: "POST" });
      if (res.ok) {
        setActiveScenario(name);
      }
    } finally {
      setLoading(false);
    }
  }, []);

  const resetScenario = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch("/api/scenario", { method: "DELETE" });
      if (res.ok) {
        setActiveScenario(null);
      }
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchStatus = useCallback(async (): Promise<StatusResponse | null> => {
    try {
      const res = await fetch("/api/status");
      if (res.ok) {
        const data = await res.json();
        setActiveScenario(data.activeScenario);
        return data;
      }
    } catch {
      // ignore
    }
    return null;
  }, []);

  return { activeScenario, loading, applyScenario, resetScenario, fetchStatus };
}
