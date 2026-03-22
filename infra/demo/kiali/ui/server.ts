import { Hono } from "hono";
import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const KIALI_DIR = resolve(__dirname, "..");
const PROMETHEUS_URL = process.env.PROMETHEUS_URL || "http://localhost:9090";
const DEMO_NAMESPACES = new Set([
  "service-mesh",
  "observability",
  "messaging",
  "k1s0-system",
  "k1s0-business",
  "k1s0-service",
]);

type MetricUnit = "reqps" | "bytesps" | "mixed";
type ProtocolKind = "http" | "tcp";
type ScenarioTraffic = "normal" | "stop";

type ScenarioDefinition = {
  commands: Array<{ script: string; args?: string[] }>;
  traffic: ScenarioTraffic;
  description: string;
};

type CanaryResponse = {
  status?: {
    phase?: string;
    canaryWeight?: number;
    failedChecks?: number;
    conditions?: Array<{ message?: string }>;
  };
};

type JobResponse = {
  items?: Array<{
    metadata?: {
      name?: string;
      creationTimestamp?: string;
    };
    status?: {
      active?: number;
      succeeded?: number;
      failed?: number;
    };
  }>;
};

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

const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });

let trafficProcess: ChildProcess | null = null;
let activeScenario: string | null = null;

const wsClients = new Set<{
  send: (data: string) => void;
  close: () => void;
}>();

const scenarios: Record<string, ScenarioDefinition> = {
  normal: {
    commands: [{ script: "reset-demo-state.sh", args: ["flagger"] }],
    traffic: "normal",
    description: "Flagger baseline with healthy traffic",
  },
  canary: {
    commands: [
      { script: "reset-demo-state.sh", args: ["flagger"] },
      { script: "start-flagger-rollout.sh", args: ["promote"] },
    ],
    traffic: "normal",
    description: "Flagger automatic canary promotion in 20% steps",
  },
  rollback: {
    commands: [
      { script: "reset-demo-state.sh", args: ["flagger"] },
      { script: "start-flagger-rollout.sh", args: ["rollback"] },
    ],
    traffic: "normal",
    description: "Flagger automatic rollback on 503 and latency regression",
  },
  fault: {
    commands: [
      { script: "reset-demo-state.sh", args: ["manual"] },
      { script: "run-fault-cronjob.sh" },
    ],
    traffic: "normal",
    description: "CronJob-backed fault injection window with auto cleanup",
  },
  tracing: {
    commands: [{ script: "reset-demo-state.sh", args: ["flagger"] }],
    traffic: "normal",
    description: "Distributed tracing in Jaeger",
  },
  logs: {
    commands: [{ script: "reset-demo-state.sh", args: ["flagger"] }],
    traffic: "normal",
    description: "Structured logs in Grafana Loki",
  },
  kafka: {
    commands: [{ script: "reset-demo-state.sh", args: ["flagger"] }],
    traffic: "stop",
    description: "Kafka producer and consumer flow",
  },
};

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
  return new Promise((resolvePromise, reject) => {
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
      if (code === 0) resolvePromise(stdout);
      else reject(new Error(stderr || `Exit code ${code}`));
    });
  });
}

async function runScript(script: string, args: string[] = []) {
  const scriptPath = resolve(KIALI_DIR, script);
  broadcast(`Running ${script}${args.length > 0 ? ` ${args.join(" ")}` : ""}...`);
  return runCommand(BASH_EXECUTABLE, [scriptPath, ...args], true);
}

async function stopTraffic() {
  if (!trafficProcess?.pid) return;

  const pid = trafficProcess.pid;
  const runningProcess = trafficProcess;
  trafficProcess = null;

  try {
    if (process.platform === "win32") {
      await runCommand("taskkill", ["/PID", `${pid}`, "/T", "/F"]);
    } else {
      try {
        process.kill(-pid, "SIGTERM");
      } catch {
        runningProcess.kill("SIGTERM");
      }
    }
  } catch {
    runningProcess.kill("SIGTERM");
  }

  broadcast("Traffic generator stopped.");
}

async function startTraffic(mode: ScenarioTraffic) {
  await stopTraffic();

  if (mode === "stop") {
    broadcast("Traffic generator remains stopped for this scenario.");
    return;
  }

  const args = [resolve(KIALI_DIR, "traffic-gen.sh"), "2", "3600", mode];
  trafficProcess = spawn(BASH_EXECUTABLE, args, {
    shell: false,
    windowsHide: true,
    detached: process.platform !== "win32",
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

async function getCanaryStatus() {
  try {
    const raw = await runCommand("kubectl", [
      "get",
      "canary",
      "task-server",
      "-n",
      "k1s0-service",
      "-o",
      "json",
    ]);
    const data = JSON.parse(raw) as CanaryResponse;
    const lastCondition = data.status?.conditions?.at(-1);

    return {
      phase: data.status?.phase ?? "Unknown",
      weight: data.status?.canaryWeight ?? 0,
      failedChecks: data.status?.failedChecks ?? 0,
      message: lastCondition?.message ?? null,
    };
  } catch {
    return null;
  }
}

async function getFaultStatus() {
  try {
    const raw = await runCommand("kubectl", [
      "get",
      "job",
      "-n",
      "k1s0-service",
      "-l",
      "app.kubernetes.io/part-of=k1s0-demo,fault-injection-run=manual",
      "-o",
      "json",
    ]);
    const data = JSON.parse(raw) as JobResponse;
    const job = [...(data.items ?? [])]
      .sort((left, right) =>
        (right.metadata?.creationTimestamp ?? "").localeCompare(
          left.metadata?.creationTimestamp ?? ""
        )
      )
      .at(0);

    if (!job?.metadata?.name) {
      return null;
    }

    let windowActive = false;
    try {
      await runCommand("kubectl", [
        "get",
        "virtualservice",
        "task-server-fault-window",
        "-n",
        "k1s0-service",
        "-o",
        "name",
      ]);
      windowActive = true;
    } catch {
      windowActive = false;
    }

    const status = job.status ?? {};
    const phase = status.active
      ? "Running"
      : status.succeeded
        ? "Succeeded"
        : status.failed
          ? "Failed"
          : "Pending";

    return {
      name: job.metadata.name,
      phase,
      windowActive,
    };
  } catch {
    return null;
  }
}

// --- REST API ---

app.post("/api/scenario/:name", async (c) => {
  const name = c.req.param("name");
  const scenario = scenarios[name];

  if (!scenario) {
    return c.json({ error: `Unknown scenario: ${name}` }, 400);
  }

  try {
    for (const command of scenario.commands) {
      await runScript(command.script, command.args ?? []);
    }

    await startTraffic(scenario.traffic);
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
    await runScript("reset-demo-state.sh", ["flagger"]);
    await startTraffic("normal");
    activeScenario = null;
    return c.json({ status: "reset" });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    return c.json({ error: msg }, 500);
  }
});

app.get("/api/status", async (c) => {
  try {
    const [pods, canary, fault] = await Promise.all([
      runCommand("kubectl", [
        "get",
        "pods",
        "-A",
        "--no-headers",
        "-o",
        "custom-columns=NS:.metadata.namespace,NAME:.metadata.name,READY:.status.conditions[?(@.type=='Ready')].status,STATUS:.status.phase",
      ]),
      getCanaryStatus(),
      getFaultStatus(),
    ]);

    const lines = pods
      .trim()
      .split("\n")
      .filter(Boolean);

    const podList = lines
      .map((line) => {
        const parts = line.trim().split(/\s+/);
        return {
          namespace: parts[0],
          name: parts[1],
          ready: parts[2] === "True",
          status: parts[3],
        };
      })
      .filter((pod) => DEMO_NAMESPACES.has(pod.namespace));

    const readyCount = podList.filter((pod) => pod.ready).length;

    return c.json({
      activeScenario,
      trafficRunning: trafficProcess !== null,
      pods: {
        ready: readyCount,
        total: podList.length,
        list: podList,
      },
      canary,
      fault,
    });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    return c.json({ error: msg }, 500);
  }
});

app.post("/api/traffic/start", async (c) => {
  await startTraffic("normal");
  return c.json({ status: "started", mode: "normal" });
});

app.post("/api/traffic/stop", async (c) => {
  await stopTraffic();
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
      for (const result of httpErrorData.data.result) {
        const destination =
          result.metric.destination_workload ||
          result.metric.destination_service_name?.split(".")[0] ||
          "unknown";
        const key = `${result.metric.source_workload}->${destination}`;
        const value = parseFloat(result.value[1]);
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
      for (const result of httpRateData.data.result) {
        const src = result.metric.source_workload;
        const srcNs = result.metric.source_workload_namespace;
        const dst = result.metric.destination_workload;
        const dstNs = result.metric.destination_workload_namespace;
        const rate = parseFloat(result.value[1]);

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
      for (const result of tcpRateData.data.result) {
        const src = result.metric.source_workload;
        const srcNs = result.metric.source_workload_namespace;
        const dst = result.metric.destination_workload;
        const dstNs = result.metric.destination_workload_namespace;
        const rate = parseFloat(result.value[1]);

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

const port = parseInt(process.env.BACKEND_PORT || "3100", 10);
const server = serve({ fetch: app.fetch, port }, (info) => {
  console.log(`Backend running on http://localhost:${info.port}`);
});

injectWebSocket(server);

process.on("SIGINT", () => {
  void stopTraffic().finally(() => process.exit(0));
});

process.on("SIGTERM", () => {
  void stopTraffic().finally(() => process.exit(0));
});
