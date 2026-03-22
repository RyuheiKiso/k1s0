#!/usr/bin/env python3
import json
import os
import random
import secrets
import threading
import time
import urllib.error
import urllib.request
from collections import defaultdict
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SERVICE_NAME = os.getenv("SERVICE_NAME", "demo-service")
NAMESPACE = os.getenv("NAMESPACE_NAME", "default")
VERSION = os.getenv("VERSION", "stable")
PORT = int(os.getenv("PORT", "8080"))
DOWNSTREAMS = [
    url.strip() for url in os.getenv("DOWNSTREAMS", "").split(",") if url.strip()
]
ENVIRONMENT = os.getenv("ENVIRONMENT_NAME", "dev")
RELEASE_TRACK = os.getenv("RELEASE_TRACK", VERSION)


def env_float(name: str, default: float) -> float:
    try:
        return float(os.getenv(name, str(default)))
    except ValueError:
        return default


def env_int(name: str, default: int) -> int:
    try:
        return int(os.getenv(name, str(default)))
    except ValueError:
        return default


FAILURE_RATE = max(0.0, min(1.0, env_float("FAILURE_RATE", 0.0)))
FIXED_DELAY_MS = max(0, env_int("FIXED_DELAY_MS", 0))
ERROR_STATUS = max(100, min(599, env_int("ERROR_STATUS", 503)))

HTTP_BUCKETS = (0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0)
DB_BUCKETS = (0.001, 0.003, 0.005, 0.01, 0.025, 0.05, 0.1)
DB_QUERY_PLANS = {
    "auth-server": [("users", "select_user_by_token", 0.002, 0.008)],
    "config-server": [("config_entries", "load_config_bundle", 0.001, 0.004)],
    "graphql-gateway": [("schema_registry", "load_resolvers", 0.001, 0.003)],
    "saga-server": [("saga_instances", "persist_step", 0.004, 0.015)],
    "project-master-server": [("tasks", "create_task", 0.003, 0.012)],
    "task-bff": [("sessions", "load_session", 0.001, 0.004)],
    "task-server": [
        ("orders", "persist_order", 0.005, 0.02),
        ("order_items", "insert_items", 0.003, 0.01),
    ],
}


def now_iso() -> str:
    return (
        datetime.now(timezone.utc)
        .isoformat(timespec="milliseconds")
        .replace("+00:00", "Z")
    )


def hex_id(bytes_len: int) -> str:
    return secrets.token_hex(bytes_len)


def infer_tier(namespace: str) -> str:
    if namespace.endswith("system"):
        return "system"
    if namespace.endswith("business"):
        return "business"
    if namespace.endswith("service"):
        return "service"
    return "infra"


TIER = os.getenv("TIER_NAME", infer_tier(NAMESPACE))
RNG = random.Random(f"{SERVICE_NAME}:{VERSION}:{NAMESPACE}")


def write_log(payload: dict) -> None:
    print(json.dumps(payload, separators=(",", ":"), ensure_ascii=True), flush=True)


class CounterMetric:
    def __init__(self, name: str, description: str, label_names: tuple[str, ...]):
        self.name = name
        self.description = description
        self.label_names = label_names
        self._values: defaultdict[tuple[str, ...], float] = defaultdict(float)

    def inc(self, labels: tuple[str, ...], amount: float = 1.0) -> None:
        self._values[labels] += amount

    def render(self) -> str:
        lines = [
            f"# HELP {self.name} {self.description}",
            f"# TYPE {self.name} counter",
        ]
        for labels, value in sorted(self._values.items()):
            lines.append(
                f"{self.name}{format_labels(self.label_names, labels)} {value:.6f}"
            )
        return "\n".join(lines)


class GaugeMetric:
    def __init__(self, name: str, description: str, label_names: tuple[str, ...]):
        self.name = name
        self.description = description
        self.label_names = label_names
        self._values: dict[tuple[str, ...], float] = {}

    def set(self, labels: tuple[str, ...], value: float) -> None:
        self._values[labels] = value

    def render(self) -> str:
        lines = [
            f"# HELP {self.name} {self.description}",
            f"# TYPE {self.name} gauge",
        ]
        for labels, value in sorted(self._values.items()):
            lines.append(
                f"{self.name}{format_labels(self.label_names, labels)} {value:.6f}"
            )
        return "\n".join(lines)


class HistogramMetric:
    def __init__(
        self,
        name: str,
        description: str,
        label_names: tuple[str, ...],
        buckets: tuple[float, ...],
    ):
        self.name = name
        self.description = description
        self.label_names = label_names
        self.buckets = buckets
        self._bucket_counts: defaultdict[tuple[str, ...], list[float]] = defaultdict(
            lambda: [0.0 for _ in self.buckets]
        )
        self._sum: defaultdict[tuple[str, ...], float] = defaultdict(float)
        self._count: defaultdict[tuple[str, ...], float] = defaultdict(float)

    def observe(self, labels: tuple[str, ...], value: float) -> None:
        for idx, upper_bound in enumerate(self.buckets):
            if value <= upper_bound:
                self._bucket_counts[labels][idx] += 1.0
        self._sum[labels] += value
        self._count[labels] += 1.0

    def render(self) -> str:
        lines = [
            f"# HELP {self.name} {self.description}",
            f"# TYPE {self.name} histogram",
        ]
        for labels in sorted(self._count):
            for idx, upper_bound in enumerate(self.buckets):
                bucket_labels = labels + (format_bucket(upper_bound),)
                lines.append(
                    f"{self.name}_bucket"
                    f"{format_labels(self.label_names + ('le',), bucket_labels)} "
                    f"{self._bucket_counts[labels][idx]:.6f}"
                )
            inf_labels = labels + ("+Inf",)
            lines.append(
                f"{self.name}_bucket"
                f"{format_labels(self.label_names + ('le',), inf_labels)} "
                f"{self._count[labels]:.6f}"
            )
            lines.append(
                f"{self.name}_sum{format_labels(self.label_names, labels)} "
                f"{self._sum[labels]:.6f}"
            )
            lines.append(
                f"{self.name}_count{format_labels(self.label_names, labels)} "
                f"{self._count[labels]:.6f}"
            )
        return "\n".join(lines)


def format_bucket(value: float) -> str:
    text = f"{value:.6f}".rstrip("0").rstrip(".")
    return text if text else "0"


def escape_label(value: str) -> str:
    return (
        value.replace("\\", "\\\\").replace("\n", "\\n").replace('"', '\\"')
    )


def format_labels(label_names: tuple[str, ...], values: tuple[str, ...]) -> str:
    if not label_names:
        return ""
    parts = [
        f'{label}="{escape_label(value)}"'
        for label, value in zip(label_names, values, strict=True)
    ]
    return "{" + ",".join(parts) + "}"


METRICS_LOCK = threading.Lock()
HTTP_REQUESTS = CounterMetric(
    "http_requests_total",
    "Total HTTP requests handled by the demo service.",
    ("service", "tier", "namespace", "environment", "method", "path", "status"),
)
HTTP_REQUEST_DURATION = HistogramMetric(
    "http_request_duration_seconds",
    "HTTP request duration for the demo service.",
    ("service", "tier", "namespace", "environment", "method", "path", "status"),
    HTTP_BUCKETS,
)
DB_QUERY_DURATION = HistogramMetric(
    "db_query_duration_seconds",
    "Simulated database query duration for the demo service.",
    ("service", "tier", "namespace", "environment", "table", "query_name"),
    DB_BUCKETS,
)
KAFKA_MESSAGES_PRODUCED = CounterMetric(
    "kafka_messages_produced_total",
    "Simulated Kafka messages produced by the demo service.",
    ("service", "topic"),
)
KAFKA_MESSAGES_CONSUMED = CounterMetric(
    "kafka_messages_consumed_total",
    "Simulated Kafka messages consumed by the demo service.",
    ("service", "topic", "consumer_group"),
)
SERVICE_INFO = GaugeMetric(
    "demo_service_info",
    "Static metadata for the demo service.",
    ("service", "namespace", "tier", "environment", "version"),
)
SERVICE_INFO.set((SERVICE_NAME, NAMESPACE, TIER, ENVIRONMENT, VERSION), 1.0)


def render_metrics() -> bytes:
    with METRICS_LOCK:
        sections = [
            SERVICE_INFO.render(),
            HTTP_REQUESTS.render(),
            HTTP_REQUEST_DURATION.render(),
            DB_QUERY_DURATION.render(),
            KAFKA_MESSAGES_PRODUCED.render(),
            KAFKA_MESSAGES_CONSUMED.render(),
        ]
    return ("\n\n".join(section for section in sections if section) + "\n").encode(
        "utf-8"
    )


def record_request_metrics(method: str, path: str, status: int, duration_ms: float) -> None:
    status_text = str(status)
    labels = (SERVICE_NAME, TIER, NAMESPACE, ENVIRONMENT, method, path, status_text)
    with METRICS_LOCK:
        HTTP_REQUESTS.inc(labels)
        HTTP_REQUEST_DURATION.observe(labels, duration_ms / 1000.0)


def record_db_metrics(response_status: int) -> None:
    plans = DB_QUERY_PLANS.get(SERVICE_NAME, [])
    with METRICS_LOCK:
        for table, query_name, low, high in plans:
            duration = RNG.uniform(low, high)
            if response_status >= 400:
                duration *= 1.25
            DB_QUERY_DURATION.observe(
                (SERVICE_NAME, TIER, NAMESPACE, ENVIRONMENT, table, query_name),
                duration,
            )


def record_kafka_metrics(response_status: int) -> None:
    if response_status >= 500:
        return
    with METRICS_LOCK:
        if SERVICE_NAME == "task-server":
            KAFKA_MESSAGES_PRODUCED.inc((SERVICE_NAME, "task-events"))
        elif SERVICE_NAME == "project-master-server":
            KAFKA_MESSAGES_CONSUMED.inc(
                (SERVICE_NAME, "task-events", "project-master-processor")
            )


class DemoHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, format: str, *args: object) -> None:
        return

    def send_json(self, status: int, payload: dict) -> None:
        body = json.dumps(payload, separators=(",", ":"), ensure_ascii=True).encode(
            "utf-8"
        )
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:
        if self.path == "/metrics":
            body = render_metrics()
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; version=0.0.4")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        if self.path in {"/healthz", "/readyz", "/livez"}:
            self.send_json(200, {"status": "ok", "service": SERVICE_NAME})
            return

        trace_id = self.headers.get("x-b3-traceid") or hex_id(16)
        span_id = self.headers.get("x-b3-spanid") or hex_id(8)
        sampled = self.headers.get("x-b3-sampled", "1")
        request_id = self.headers.get("x-request-id") or trace_id
        canary_header = self.headers.get("x-canary")
        user_id = self.headers.get("x-user-id")

        overall_status = 200
        downstream_results = []
        error_payload = None
        started = time.time()

        for url in DOWNSTREAMS:
            child_span_id = hex_id(8)
            headers = {
                "x-b3-traceid": trace_id,
                "x-b3-spanid": child_span_id,
                "x-b3-parentspanid": span_id,
                "x-b3-sampled": sampled,
                "x-request-id": request_id,
                "user-agent": f"{SERVICE_NAME}/{VERSION}",
            }
            if canary_header:
                headers["x-canary"] = canary_header

            req = urllib.request.Request(url, headers=headers, method="GET")
            child_status = 200

            try:
                with urllib.request.urlopen(req, timeout=3) as resp:
                    child_status = resp.getcode()
                    resp.read()
            except urllib.error.HTTPError as exc:
                child_status = exc.code
                overall_status = 502
                error_payload = {
                    "message": "downstream returned error",
                    "target": url,
                    "status": child_status,
                }
            except Exception as exc:  # noqa: BLE001
                child_status = 599
                overall_status = 502
                error_payload = {
                    "message": "downstream request failed",
                    "target": url,
                    "detail": str(exc),
                }
                write_log(
                    {
                        "timestamp": now_iso(),
                        "level": "error",
                        "message": "downstream request failed",
                        "service": SERVICE_NAME,
                        "version": VERSION,
                        "tier": TIER,
                        "environment": ENVIRONMENT,
                        "namespace": NAMESPACE,
                        "release_track": RELEASE_TRACK,
                        "trace_id": trace_id,
                        "span_id": child_span_id,
                        "request_id": request_id,
                        "target": url,
                        "error": error_payload,
                    }
                )

            if child_status >= 400:
                overall_status = 502

            downstream_results.append({"target": url, "status": child_status})

        duration_ms = round((time.time() - started) * 1000, 2)

        if FIXED_DELAY_MS > 0:
            time.sleep(FIXED_DELAY_MS / 1000.0)
            duration_ms = round((time.time() - started) * 1000, 2)

        if FAILURE_RATE > 0 and RNG.random() < FAILURE_RATE:
            overall_status = ERROR_STATUS
            error_payload = {
                "message": "demo canary failure injected",
                "failure_rate": FAILURE_RATE,
                "fixed_delay_ms": FIXED_DELAY_MS,
                "status": ERROR_STATUS,
            }

        record_request_metrics(self.command, self.path, overall_status, duration_ms)
        record_db_metrics(overall_status)
        record_kafka_metrics(overall_status)

        body = json.dumps(
            {
                "service": SERVICE_NAME,
                "namespace": NAMESPACE,
                "tier": TIER,
                "environment": ENVIRONMENT,
                "version": VERSION,
                "release_track": RELEASE_TRACK,
                "trace_id": trace_id,
                "request_id": request_id,
                "failure_rate": FAILURE_RATE,
                "fixed_delay_ms": FIXED_DELAY_MS,
                "downstreams": downstream_results,
            },
            separators=(",", ":"),
            ensure_ascii=True,
        ).encode("utf-8")

        write_log(
            {
                "timestamp": now_iso(),
                "level": "info" if overall_status < 400 else "error",
                "message": "demo request handled",
                "service": SERVICE_NAME,
                "version": VERSION,
                "tier": TIER,
                "environment": ENVIRONMENT,
                "namespace": NAMESPACE,
                "release_track": RELEASE_TRACK,
                "trace_id": trace_id,
                "span_id": span_id,
                "request_id": request_id,
                "method": self.command,
                "path": self.path,
                "status": overall_status,
                "duration_ms": duration_ms,
                "user_id": user_id,
                "failure_rate": FAILURE_RATE,
                "fixed_delay_ms": FIXED_DELAY_MS,
                "downstreams": downstream_results,
                "error": error_payload,
            }
        )

        self.send_response(overall_status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("x-trace-id", trace_id)
        self.send_header("x-request-id", request_id)
        self.end_headers()
        self.wfile.write(body)


def main() -> int:
    server = ThreadingHTTPServer(("0.0.0.0", PORT), DemoHandler)
    write_log(
        {
            "timestamp": now_iso(),
            "level": "info",
            "message": f"listening on {PORT}",
            "service": SERVICE_NAME,
            "version": VERSION,
            "tier": TIER,
            "environment": ENVIRONMENT,
            "namespace": NAMESPACE,
            "release_track": RELEASE_TRACK,
            "error": None,
        }
    )
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
