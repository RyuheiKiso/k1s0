/**
 * Metrics は Prometheus 互換メトリクスのヘルパークラスである。
 * RED メソッド（Rate, Errors, Duration）のメトリクスを提供する。
 * Go 実装の metrics.go と同等の機能を持つ。
 */
const DEFAULT_BUCKETS = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10];
function labelsKey(labels) {
    return Object.entries(labels)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([k, v]) => `${k}="${v}"`)
        .join(',');
}
function formatLabels(labels) {
    return Object.entries(labels)
        .map(([k, v]) => `${k}="${v}"`)
        .join(',');
}
export class Metrics {
    serviceName;
    httpRequestsTotal;
    httpRequestDuration;
    grpcHandledTotal;
    grpcHandlingDuration;
    constructor(serviceName) {
        this.serviceName = serviceName;
        this.httpRequestsTotal = {
            name: 'http_requests_total',
            help: 'Total number of HTTP requests',
            type: 'counter',
            entries: new Map(),
        };
        this.httpRequestDuration = {
            name: 'http_request_duration_seconds',
            help: 'Histogram of HTTP request latency',
            type: 'histogram',
            entries: new Map(),
        };
        this.grpcHandledTotal = {
            name: 'grpc_server_handled_total',
            help: 'Total number of RPCs completed on the server',
            type: 'counter',
            entries: new Map(),
        };
        this.grpcHandlingDuration = {
            name: 'grpc_server_handling_seconds',
            help: 'Histogram of response latency of gRPC',
            type: 'histogram',
            entries: new Map(),
        };
    }
    recordHTTPRequest(method, path, status) {
        const labels = { service: this.serviceName, method, path, status: String(status) };
        const key = labelsKey(labels);
        const existing = this.httpRequestsTotal.entries.get(key);
        if (existing) {
            existing.value++;
        }
        else {
            this.httpRequestsTotal.entries.set(key, { labels, value: 1 });
        }
    }
    recordHTTPDuration(method, path, durationSeconds) {
        const labels = { service: this.serviceName, method, path };
        const key = labelsKey(labels);
        const existing = this.httpRequestDuration.entries.get(key);
        if (existing) {
            existing.sum += durationSeconds;
            existing.count++;
            for (let i = 0; i < DEFAULT_BUCKETS.length; i++) {
                if (durationSeconds <= DEFAULT_BUCKETS[i]) {
                    existing.buckets[i]++;
                }
            }
        }
        else {
            const buckets = DEFAULT_BUCKETS.map((b) => (durationSeconds <= b ? 1 : 0));
            this.httpRequestDuration.entries.set(key, {
                labels,
                sum: durationSeconds,
                count: 1,
                buckets,
            });
        }
    }
    recordGRPCRequest(grpcService, grpcMethod, grpcCode) {
        const labels = {
            service: this.serviceName,
            grpc_service: grpcService,
            grpc_method: grpcMethod,
            grpc_code: grpcCode,
        };
        const key = labelsKey(labels);
        const existing = this.grpcHandledTotal.entries.get(key);
        if (existing) {
            existing.value++;
        }
        else {
            this.grpcHandledTotal.entries.set(key, { labels, value: 1 });
        }
    }
    recordGRPCDuration(grpcService, grpcMethod, durationSeconds) {
        const labels = { service: this.serviceName, grpc_service: grpcService, grpc_method: grpcMethod };
        const key = labelsKey(labels);
        const existing = this.grpcHandlingDuration.entries.get(key);
        if (existing) {
            existing.sum += durationSeconds;
            existing.count++;
            for (let i = 0; i < DEFAULT_BUCKETS.length; i++) {
                if (durationSeconds <= DEFAULT_BUCKETS[i]) {
                    existing.buckets[i]++;
                }
            }
        }
        else {
            const buckets = DEFAULT_BUCKETS.map((b) => (durationSeconds <= b ? 1 : 0));
            this.grpcHandlingDuration.entries.set(key, {
                labels,
                sum: durationSeconds,
                count: 1,
                buckets,
            });
        }
    }
    getMetrics() {
        const lines = [];
        this.serializeCounter(lines, this.httpRequestsTotal);
        this.serializeHistogram(lines, this.httpRequestDuration);
        this.serializeCounter(lines, this.grpcHandledTotal);
        this.serializeHistogram(lines, this.grpcHandlingDuration);
        return lines.join('\n');
    }
    serializeCounter(lines, def) {
        lines.push(`# HELP ${def.name} ${def.help}`);
        lines.push(`# TYPE ${def.name} counter`);
        for (const entry of def.entries.values()) {
            lines.push(`${def.name}{${formatLabels(entry.labels)}} ${entry.value}`);
        }
    }
    serializeHistogram(lines, def) {
        lines.push(`# HELP ${def.name} ${def.help}`);
        lines.push(`# TYPE ${def.name} histogram`);
        for (const entry of def.entries.values()) {
            const lblStr = formatLabels(entry.labels);
            let cumulative = 0;
            for (let i = 0; i < DEFAULT_BUCKETS.length; i++) {
                cumulative += entry.buckets[i];
                lines.push(`${def.name}_bucket{${lblStr},le="${DEFAULT_BUCKETS[i]}"} ${cumulative}`);
            }
            lines.push(`${def.name}_bucket{${lblStr},le="+Inf"} ${entry.count}`);
            lines.push(`${def.name}_sum{${lblStr}} ${entry.sum}`);
            lines.push(`${def.name}_count{${lblStr}} ${entry.count}`);
        }
    }
}
