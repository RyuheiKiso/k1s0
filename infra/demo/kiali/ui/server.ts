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
