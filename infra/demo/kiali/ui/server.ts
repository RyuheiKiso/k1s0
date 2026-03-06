import { Hono } from "hono";
import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { spawn, type ChildProcess } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const KIALI_DIR = resolve(__dirname, "..");

const app = new Hono();
const { injectWebSocket, upgradeWebSocket } = createNodeWebSocket({ app });

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
    const proc = spawn(command, args, { shell: true });
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
  { apply: string | null; traffic: string; description: string }
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
    apply: "scenarios/kafka-flow.yaml",
    traffic: "normal",
    description: "Kafka messaging - async event flow",
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
    await runCommand(
      "kubectl",
      ["delete", "vs", "kafka-flow", "-n", "messaging", "--ignore-not-found"],
      true
    );
  } catch {
    // Ignore errors from non-existent resources
  }
  broadcast("All scenario VirtualServices removed.");
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
  const args = [resolve(KIALI_DIR, "traffic-gen.sh"), "2", "3600"];
  if (mode === "canary") args.push("canary");

  trafficProcess = spawn("bash", args, { shell: true });
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

const PROMETHEUS_URL = "http://localhost:9090";

app.get("/api/topology", async (c) => {
  try {
    // Query request rates between workloads
    const rateQuery = encodeURIComponent(
      'sum(rate(istio_requests_total{reporter="destination"}[5m])) by (source_workload, source_workload_namespace, destination_workload, destination_workload_namespace) > 0'
    );
    const errorQuery = encodeURIComponent(
      'sum(rate(istio_requests_total{reporter="destination",response_code!~"2.."}[5m])) by (source_workload, destination_workload) > 0'
    );

    const [rateRes, errorRes] = await Promise.all([
      fetch(`${PROMETHEUS_URL}/api/v1/query?query=${rateQuery}`),
      fetch(`${PROMETHEUS_URL}/api/v1/query?query=${errorQuery}`),
    ]);

    const rateData = await rateRes.json();
    const errorData = await errorRes.json();

    // Build error map
    const errorMap = new Map<string, number>();
    if (errorData.data?.result) {
      for (const r of errorData.data.result) {
        const key = `${r.metric.source_workload}->${r.metric.destination_workload}`;
        errorMap.set(key, parseFloat(r.value[1]));
      }
    }

    // Build nodes and edges
    const nodeSet = new Map<string, { id: string; namespace: string; rate: number }>();
    const edges: Array<{
      source: string;
      target: string;
      rate: number;
      errorRate: number;
    }> = [];

    if (rateData.data?.result) {
      for (const r of rateData.data.result) {
        const src = r.metric.source_workload;
        const srcNs = r.metric.source_workload_namespace;
        const dst = r.metric.destination_workload;
        const dstNs = r.metric.destination_workload_namespace;
        const rate = parseFloat(r.value[1]);

        // Accumulate node traffic
        const srcNode = nodeSet.get(src);
        if (srcNode) srcNode.rate += rate;
        else nodeSet.set(src, { id: src, namespace: srcNs, rate });

        const dstNode = nodeSet.get(dst);
        if (dstNode) dstNode.rate += rate;
        else nodeSet.set(dst, { id: dst, namespace: dstNs, rate });

        const errorKey = `${src}->${dst}`;
        const errRate = errorMap.get(errorKey) || 0;

        edges.push({ source: src, target: dst, rate, errorRate: errRate });
      }
    }

    const nodes = Array.from(nodeSet.values());

    return c.json({ nodes, edges });
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error);
    return c.json({ error: msg }, 500);
  }
});

// --- Jaeger Dependencies: inject synthetic multi-service traces ---

const JAEGER_OTLP_URL = "http://localhost:4318";

async function injectTraces() {
  const servicePairs = [
    ["order-bff.k1s0-service", "order-server.k1s0-service"],
    ["order-bff.k1s0-service", "graphql-gateway.k1s0-system"],
    ["order-server.k1s0-service", "auth-server.k1s0-system"],
    ["order-server.k1s0-service", "config-server.k1s0-system"],
    ["order-server.k1s0-service", "saga-server.k1s0-system"],
    ["order-server.k1s0-service", "accounting-server.k1s0-business"],
    ["accounting-server.k1s0-business", "auth-server.k1s0-system"],
    ["accounting-server.k1s0-business", "config-server.k1s0-system"],
    ["graphql-gateway.k1s0-system", "auth-server.k1s0-system"],
    ["graphql-gateway.k1s0-system", "config-server.k1s0-system"],
    ["saga-server.k1s0-system", "auth-server.k1s0-system"],
    ["saga-server.k1s0-system", "config-server.k1s0-system"],
    ["auth-server.k1s0-system", "config-server.k1s0-system"],
  ];

  const hexBytes = (n: number) =>
    Array.from({ length: n }, () =>
      Math.floor(Math.random() * 256).toString(16).padStart(2, "0")
    ).join("");

  const now = Date.now() * 1_000_000; // nanoseconds

  for (const [parent, child] of servicePairs) {
    const traceId = hexBytes(16);
    const parentSpanId = hexBytes(8);
    const childSpanId = hexBytes(8);

    const payload = {
      resourceSpans: [
        {
          resource: {
            attributes: [
              { key: "service.name", value: { stringValue: parent } },
            ],
          },
          scopeSpans: [
            {
              spans: [
                {
                  traceId,
                  spanId: parentSpanId,
                  name: `${parent} -> ${child}`,
                  kind: 3, // CLIENT
                  startTimeUnixNano: String(now - 50_000_000),
                  endTimeUnixNano: String(now),
                  attributes: [
                    {
                      key: "http.method",
                      value: { stringValue: "GET" },
                    },
                    {
                      key: "http.status_code",
                      value: { intValue: "200" },
                    },
                  ],
                },
              ],
            },
          ],
        },
        {
          resource: {
            attributes: [
              { key: "service.name", value: { stringValue: child } },
            ],
          },
          scopeSpans: [
            {
              spans: [
                {
                  traceId,
                  spanId: childSpanId,
                  parentSpanId: parentSpanId,
                  name: `${child} handler`,
                  kind: 2, // SERVER
                  startTimeUnixNano: String(now - 40_000_000),
                  endTimeUnixNano: String(now - 5_000_000),
                  attributes: [
                    {
                      key: "http.method",
                      value: { stringValue: "GET" },
                    },
                    {
                      key: "http.status_code",
                      value: { intValue: "200" },
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    };

    try {
      await fetch(`${JAEGER_OTLP_URL}/v1/traces`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
    } catch {
      // Jaeger OTLP may not be port-forwarded yet
    }
  }
}

// Inject traces periodically (every 30s)
setInterval(injectTraces, 30_000);
// Initial injection after 3s
setTimeout(injectTraces, 3_000);

// --- Grafana Node Graph endpoints (for Infinity datasource) ---

// Known service architecture (from traffic-gen.sh)
const SERVICE_TOPOLOGY = {
  nodes: [
    { id: "order-bff", ns: "k1s0-service" },
    { id: "order-server", ns: "k1s0-service" },
    { id: "graphql-gateway", ns: "k1s0-system" },
    { id: "auth-server", ns: "k1s0-system" },
    { id: "config-server", ns: "k1s0-system" },
    { id: "saga-server", ns: "k1s0-system" },
    { id: "accounting-server", ns: "k1s0-business" },
  ],
  edges: [
    { source: "order-bff", target: "order-server" },
    { source: "order-bff", target: "graphql-gateway" },
    { source: "order-server", target: "auth-server" },
    { source: "order-server", target: "config-server" },
    { source: "order-server", target: "saga-server" },
    { source: "order-server", target: "accounting-server" },
    { source: "accounting-server", target: "auth-server" },
    { source: "accounting-server", target: "config-server" },
    { source: "graphql-gateway", target: "auth-server" },
    { source: "graphql-gateway", target: "config-server" },
    { source: "saga-server", target: "auth-server" },
    { source: "saga-server", target: "config-server" },
    { source: "auth-server", target: "config-server" },
  ],
};

app.get("/api/grafana/nodes", async (c) => {
  try {
    const query = encodeURIComponent(
      'sum(rate(istio_requests_total{reporter="destination"}[5m])) by (destination_workload, destination_workload_namespace)'
    );
    const res = await fetch(`${PROMETHEUS_URL}/api/v1/query?query=${query}`);
    const data = await res.json();

    // Build rate map from actual metrics
    const rateMap = new Map<string, number>();
    for (const r of data.data?.result || []) {
      const id = r.metric.destination_workload;
      if (id === "unknown") continue;
      rateMap.set(id, (rateMap.get(id) || 0) + parseFloat(r.value[1]));
    }

    const nodes = SERVICE_TOPOLOGY.nodes.map((n) => ({
      id: n.id,
      title: n.id,
      subtitle: n.ns,
      mainStat: rateMap.get(n.id) || 0,
    }));

    c.header("Access-Control-Allow-Origin", "*");
    return c.json(nodes);
  } catch {
    return c.json([], 200);
  }
});

app.get("/api/grafana/edges", async (c) => {
  try {
    const query = encodeURIComponent(
      'sum(rate(istio_requests_total{reporter="destination"}[5m])) by (destination_workload)'
    );
    const res = await fetch(`${PROMETHEUS_URL}/api/v1/query?query=${query}`);
    const data = await res.json();

    // Build destination rate map
    const dstRate = new Map<string, number>();
    for (const r of data.data?.result || []) {
      const id = r.metric.destination_workload;
      if (id === "unknown") continue;
      dstRate.set(id, parseFloat(r.value[1]));
    }

    // Distribute rate across known source edges proportionally
    const edgesByTarget = new Map<string, typeof SERVICE_TOPOLOGY.edges>();
    for (const e of SERVICE_TOPOLOGY.edges) {
      if (!edgesByTarget.has(e.target)) edgesByTarget.set(e.target, []);
      edgesByTarget.get(e.target)!.push(e);
    }

    const edges = SERVICE_TOPOLOGY.edges.map((e, i) => {
      const totalRate = dstRate.get(e.target) || 0;
      const numSources = edgesByTarget.get(e.target)?.length || 1;
      return {
        id: String(i),
        source: e.source,
        target: e.target,
        mainStat: Math.round((totalRate / numSources) * 1000) / 1000,
      };
    });

    c.header("Access-Control-Allow-Origin", "*");
    return c.json(edges);
  } catch {
    return c.json([], 200);
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
