import { Hono } from "hono";
import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const KIALI_DIR = resolve(__dirname, "..");
const BASELINE_VIRTUAL_SERVICES = resolve(
  KIALI_DIR,
  "manifests/02-virtualservices.yaml"
);

const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });
const PROMETHEUS_URL =
  process.env.PROMETHEUS_URL || "http://localhost:9090";

type MetricUnit = "reqps" | "bytesps" | "mixed";
type ProtocolKind = "http" | "tcp";

function resolveBashExecutable() {
  const configured = process.env.GIT_BASH;
  if (configured && existsSync(configured)) return configured;

  if (process.platform === "win32") {
    const candidates = [
      "C:/Program Files/Git/bin/bash.exe",
      "C:/Program Files/Git/usr/bin/bash.exe",
      "C:/Program Files (x86)/Git/bin/bash.exe",
      "C:/Program Files (x86)/Git/usr/bin/bash.exe",
    ];

    for (const candidate of candidates) {
      if (existsSync(candidate)) return candidate;
    }
  }

  return "bash";
}

const BASH_EXECUTABLE = resolveBashExecutable();

// Active processes
let trafficProcess: ChildProcess | null = null;
let activeScenario: string | null = null;

// WebSocket clients
const wsClients = new Set<{
  send: (data: string) => void;
  close: () => void;
}>();

function broadcast(message: string) {
  const data = JSON.stringify({
    timestamp: new Date().toISOString(),
    message,
  });
  for (const client of wsClients) {
    try {
      client.send(data);
    } catch {
      wsClients.delete(client);
    }
  }
}

function runCommand(
  command: string,
  args: string[],
  stream = false
): Promise<string> {
  return new Promise((resolve, reject) => {
    const proc = spawn(command, args, { shell: false, windowsHide: true });
    let stdout = "";
    let stderr = "";

    proc.stdout.on("data", (data: Buffer) => {
      const text = data.toString();
      stdout += text;
      if (stream) broadcast(text.trimEnd());
    });

    proc.stderr.on("data", (data: Buffer) => {
      const text = data.toString();
      stderr += text;
      if (stream) broadcast(text.trimEnd());
    });

    proc.on("close", (code: number | null) => {
      if (code === 0) resolve(stdout);
      else reject(new Error(stderr || `Exit code ${code}`));
    });
  });
}

// Scenario definitions
const scenarios: Record<
  string,
  {
    apply: string | null;
    traffic: "normal" | "canary" | "stop";
    description: string;
  }
> = {
  normal: {
    apply: null,
    traffic: "normal",
    description: "Normal traffic - all services healthy",
  },
  canary: {
    apply: "scenarios/canary.yaml",
    traffic: "normal",
    description: "Canary release - 90:10 weight split",
  },
  header: {
    apply: "scenarios/canary.yaml",
    traffic: "canary",
    description: "Header routing - x-canary:true to canary",
  },
  mirror: {
    apply: "scenarios/mirror.yaml",
    traffic: "normal",
    description: "Traffic mirroring - 10% copy to canary",
  },
  fault: {
    apply: "scenarios/fault-abort.yaml",
    traffic: "normal",
    description: "Fault injection - 500ms delay + 503 abort",
  },
  tracing: {
    apply: null,
    traffic: "normal",
    description: "Distributed tracing - view in Jaeger",
  },
  logs: {
    apply: null,
    traffic: "normal",
    description: "Log aggregation - Grafana + Loki",
  },
  kafka: {
    apply: null,
    traffic: "stop",
    description: "Kafka producer/consumer flow",
  },
};

// Reset all scenarios
async function resetScenarios() {
  broadcast("Resetting all scenarios...");
  try {
    await runCommand(
      "kubectl",
      [
        "delete",
        "vs",
        "order-server-canary",
        "order-server-mirror",
        "order-server-fault",
        "-n",
        "k1s0-service",
        "--ignore-not-found",
      ],
      true
    );
  } catch {
    // Ignore errors from non-existent resources
  }

  await runCommand(
    "kubectl",
    ["apply", "-f", BASELINE_VIRTUAL_SERVICES],
    true
  );
  broadcast("Baseline VirtualServices restored.");
}

// Stop traffic generator
function stopTraffic() {
  if (trafficProcess) {
    trafficProcess.kill("SIGTERM");
    trafficProcess = null;
    broadcast("Traffic generator stopped.");
  }
}

// Start traffic generator
function startTraffic(mode: string) {
  stopTraffic();
  if (mode === "stop") {
    broadcast("Traffic generator remains stopped for this scenario.");
    return;
  }
  const args = [resolve(KIALI_DIR, "traffic-gen.sh"), "2", "3600"];
  if (mode === "canary") args.push("canary");

  trafficProcess = spawn(BASH_EXECUTABLE, args, {
    shell: false,
    windowsHide: true,
  });
  broadcast(`Traffic generator started (mode: ${mode})`);

  trafficProcess.stdout?.on("data", (data: Buffer) => {
    broadcast(data.toString().trimEnd());
  });

  trafficProcess.stderr?.on("data", (data: Buffer) => {
    broadcast(data.toString().trimEnd());
  });

  trafficProcess.on("close", (code: number | null) => {
    broadcast(`Traffic generator exited (code: ${code})`);
    trafficProcess = null;
  });
}

// --- REST API ---

app.post("/api/scenario/:name", async (c) => {
  const name = c.req.param("name");
  const scenario = scenarios[name];

  if (!scenario) {
    return c.json({ error: `Unknown scenario: ${name}` }, 400);
  }

  try {
    await resetScenarios();

    if (scenario.apply) {
      const yamlPath = resolve(KIALI_DIR, scenario.apply);
      broadcast(`Applying ${scenario.apply}...`);
      await runCommand("kubectl", ["apply", "-f", yamlPath], true);
    }

    startTraffic(scenario.traffic);
    activeScenario = name;

    return c.json({
      scenario: name,
      description: scenario.description,
      status: "applied",
    });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    broadcast(`Error: ${msg}`);
    return c.json({ error: msg }, 500);
  }
});

app.delete("/api/scenario", async (c) => {
  try {
    await resetScenarios();
    startTraffic("normal");
    activeScenario = null;
    return c.json({ status: "reset" });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    return c.json({ error: msg }, 500);
  }
});

app.get("/api/status", async (c) => {
  try {
    const pods = await runCommand("kubectl", [
      "get",
      "pods",
      "-A",
      "-l",
      "app",
      "--no-headers",
      "-o",
      "custom-columns=NS:.metadata.namespace,NAME:.metadata.name,READY:.status.conditions[?(@.type=='Ready')].status,STATUS:.status.phase",
    ]);

    const lines = pods
      .trim()
      .split("\n")
      .filter(Boolean);
    const podList = lines.map((line) => {
      const parts = line.trim().split(/\s+/);
      return {
        namespace: parts[0],
        name: parts[1],
        ready: parts[2] === "True",
        status: parts[3],
      };
    });

    const readyCount = podList.filter((p) => p.ready).length;

    return c.json({
      activeScenario,
      trafficRunning: trafficProcess !== null,
      pods: {
        ready: readyCount,
        total: podList.length,
        list: podList,
      },
    });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    return c.json({ error: msg }, 500);
  }
});

app.post("/api/traffic/start", (c) => {
  const mode = activeScenario === "header" ? "canary" : "normal";
  startTraffic(mode);
  return c.json({ status: "started", mode });
});

app.post("/api/traffic/stop", (c) => {
  stopTraffic();
  return c.json({ status: "stopped" });
});

// --- Topology API (query Prometheus) ---

app.get("/api/topology", async (c) => {
  try {
    const httpRateQuery = encodeURIComponent(
      'sum(rate(istio_requests_total{reporter="destination"}[1m])) by (source_workload, source_workload_namespace, destination_workload, destination_workload_namespace) > 0'
    );
    const httpErrorQuery = encodeURIComponent(
      'sum(rate(istio_requests_total{response_code!~"2.."}[1m])) by (reporter, source_workload, destination_workload, destination_service_name) > 0'
    );
    const tcpRateQuery = encodeURIComponent(
      'sum(rate(istio_tcp_sent_bytes_total{reporter="source"}[1m])) by (source_workload, source_workload_namespace, destination_workload, destination_workload_namespace) > 0'
    );

    const [httpRateRes, httpErrorRes, tcpRateRes] = await Promise.all([
      fetch(`${PROMETHEUS_URL}/api/v1/query?query=${httpRateQuery}`),
      fetch(`${PROMETHEUS_URL}/api/v1/query?query=${httpErrorQuery}`),
      fetch(`${PROMETHEUS_URL}/api/v1/query?query=${tcpRateQuery}`),
    ]);

    const httpRateData = await httpRateRes.json();
    const httpErrorData = await httpErrorRes.json();
    const tcpRateData = await tcpRateRes.json();

    const errorMap = new Map<string, number>();
    if (httpErrorData.data?.result) {
      for (const r of httpErrorData.data.result) {
        const destination =
          r.metric.destination_workload ||
          r.metric.destination_service_name?.split(".")[0] ||
          "unknown";
        const key = `${r.metric.source_workload}->${destination}`;
        const value = parseFloat(r.value[1]);
        errorMap.set(key, Math.max(errorMap.get(key) || 0, value));
      }
    }

    const nodeSet = new Map<
      string,
      {
        id: string;
        namespace: string;
        rate: number;
        unit: MetricUnit;
      }
    >();
    const edges: Array<{
      source: string;
      target: string;
      rate: number;
      errorRate: number;
      protocol: ProtocolKind;
      unit: Exclude<MetricUnit, "mixed">;
    }> = [];

    const recordNode = (
      id: string,
      namespace: string,
      rate: number,
      unit: Exclude<MetricUnit, "mixed">
    ) => {
      const existing = nodeSet.get(id);

      if (!existing) {
        nodeSet.set(id, { id, namespace, rate, unit });
        return;
      }

      existing.rate += rate;
      existing.unit = existing.unit === unit ? unit : "mixed";
    };

    if (httpRateData.data?.result) {
      for (const r of httpRateData.data.result) {
        const src = r.metric.source_workload;
        const srcNs = r.metric.source_workload_namespace;
        const dst = r.metric.destination_workload;
        const dstNs = r.metric.destination_workload_namespace;
        const rate = parseFloat(r.value[1]);

        recordNode(src, srcNs, rate, "reqps");
        recordNode(dst, dstNs, rate, "reqps");

        const errorKey = `${src}->${dst}`;
        const errRate = errorMap.get(errorKey) || 0;

        edges.push({
          source: src,
          target: dst,
          rate,
          errorRate: errRate,
          protocol: "http",
          unit: "reqps",
        });
      }
    }

    if (tcpRateData.data?.result) {
      for (const r of tcpRateData.data.result) {
        const src = r.metric.source_workload;
        const srcNs = r.metric.source_workload_namespace;
        const dst = r.metric.destination_workload;
        const dstNs = r.metric.destination_workload_namespace;
        const rate = parseFloat(r.value[1]);

        recordNode(src, srcNs, rate, "bytesps");
        recordNode(dst, dstNs, rate, "bytesps");

        edges.push({
          source: src,
          target: dst,
          rate,
          errorRate: 0,
          protocol: "tcp",
          unit: "bytesps",
        });
      }
    }

    const nodes = Array.from(nodeSet.values());

    return c.json({ nodes, edges });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    return c.json({ error: msg }, 500);
  }
});

// --- WebSocket ---

app.get(
  "/ws/logs",
  upgradeWebSocket(() => ({
    onOpen(_event, ws) {
      const client = {
        send: (data: string) => ws.send(data),
        close: () => ws.close(),
      };
      wsClients.add(client);
      ws.send(
        JSON.stringify({
          timestamp: new Date().toISOString(),
          message: "Connected to log stream",
        })
      );

      // Store reference for cleanup
      (ws as unknown as Record<string, unknown>).__client = client;
    },
    onClose(_event, ws) {
      const client = (ws as unknown as Record<string, unknown>).__client as
        | (typeof wsClients extends Set<infer T> ? T : never)
        | undefined;
      if (client) wsClients.delete(client);
    },
  }))
);

// --- Start server ---

const port = parseInt(process.env.BACKEND_PORT || "3100", 10);
const server = serve({ fetch: app.fetch, port }, (info) => {
  console.log(`Backend running on http://localhost:${info.port}`);
});

injectWebSocket(server);

// Cleanup on exit
process.on("SIGINT", () => {
  stopTraffic();
  process.exit(0);
});

process.on("SIGTERM", () => {
  stopTraffic();
  process.exit(0);
});
