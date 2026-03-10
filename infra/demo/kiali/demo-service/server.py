#!/usr/bin/env python3
import json
import os
import secrets
import sys
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

SERVICE_NAME = os.getenv("SERVICE_NAME", "demo-service")
NAMESPACE = os.getenv("NAMESPACE_NAME", "default")
VERSION = os.getenv("VERSION", "stable")
PORT = int(os.getenv("PORT", "8080"))
DOWNSTREAMS = [url.strip() for url in os.getenv("DOWNSTREAMS", "").split(",") if url.strip()]


def now_iso() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def hex_id(bytes_len: int) -> str:
    return secrets.token_hex(bytes_len)


def write_log(payload: dict) -> None:
    print(json.dumps(payload, separators=(",", ":")), flush=True)


class DemoHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, format: str, *args: object) -> None:
        return

    def do_GET(self) -> None:
        trace_id = self.headers.get("x-b3-traceid") or hex_id(16)
        span_id = self.headers.get("x-b3-spanid") or hex_id(8)
        sampled = self.headers.get("x-b3-sampled", "1")
        canary_header = self.headers.get("x-canary")

        overall_status = 200
        downstream_results = []
        started = time.time()

        for url in DOWNSTREAMS:
            child_span_id = hex_id(8)
            headers = {
                "x-b3-traceid": trace_id,
                "x-b3-spanid": child_span_id,
                "x-b3-parentspanid": span_id,
                "x-b3-sampled": sampled,
                "x-request-id": trace_id,
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
            except Exception as exc:  # noqa: BLE001
                child_status = 599
                overall_status = 502
                write_log(
                    {
                        "timestamp": now_iso(),
                        "level": "ERROR",
                        "service": SERVICE_NAME,
                        "namespace": NAMESPACE,
                        "version": VERSION,
                        "trace_id": trace_id,
                        "span_id": child_span_id,
                        "message": "downstream request failed",
                        "target": url,
                        "error": str(exc),
                    }
                )

            if child_status >= 400:
                overall_status = 502

            downstream_results.append(
                {
                    "target": url,
                    "status": child_status,
                }
            )

        duration_ms = round((time.time() - started) * 1000, 2)
        body = json.dumps(
            {
                "service": SERVICE_NAME,
                "namespace": NAMESPACE,
                "version": VERSION,
                "trace_id": trace_id,
                "downstreams": downstream_results,
            },
            separators=(",", ":"),
        ).encode("utf-8")

        write_log(
            {
                "timestamp": now_iso(),
                "level": "INFO" if overall_status < 400 else "ERROR",
                "service": SERVICE_NAME,
                "namespace": NAMESPACE,
                "version": VERSION,
                "trace_id": trace_id,
                "span_id": span_id,
                "message": "demo request handled",
                "path": self.path,
                "status": overall_status,
                "duration_ms": duration_ms,
                "downstreams": downstream_results,
            }
        )

        self.send_response(overall_status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("x-trace-id", trace_id)
        self.end_headers()
        self.wfile.write(body)


def main() -> int:
    server = ThreadingHTTPServer(("0.0.0.0", PORT), DemoHandler)
    write_log(
        {
            "timestamp": now_iso(),
            "level": "INFO",
            "service": SERVICE_NAME,
            "namespace": NAMESPACE,
            "version": VERSION,
            "message": f"listening on {PORT}",
        }
    )
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
