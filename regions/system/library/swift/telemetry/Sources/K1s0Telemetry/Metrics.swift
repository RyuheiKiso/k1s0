/// HTTP メトリクスキー。
public struct HttpMetricsKey: Hashable, Sendable {
    public let method: String
    public let path: String
    public let status: Int
}

/// メトリクス（メモリベース）。
public actor Metrics: Sendable {
    private var httpRequestsTotal: [HttpMetricsKey: Int] = [:]
    private var httpRequestDurationMs: [HttpMetricsKey: [Double]] = [:]

    public init() {}

    /// HTTP リクエスト数をインクリメントする。
    public func incrementHttpRequests(method: String, path: String, status: Int) {
        let key = HttpMetricsKey(method: method, path: path, status: status)
        httpRequestsTotal[key, default: 0] += 1
    }

    /// HTTP リクエスト時間を記録する。
    public func recordHttpDuration(method: String, path: String, status: Int, durationMs: Double) {
        let key = HttpMetricsKey(method: method, path: path, status: status)
        httpRequestDurationMs[key, default: []].append(durationMs)
    }

    /// HTTP リクエスト数を返す。
    public func httpRequestCount(method: String, path: String, status: Int) -> Int {
        let key = HttpMetricsKey(method: method, path: path, status: status)
        return httpRequestsTotal[key] ?? 0
    }

    /// Prometheus テキスト形式でメトリクスをエクスポートする。
    public func exportPrometheus() -> String {
        var lines: [String] = []
        lines.append("# HELP http_requests_total Total HTTP requests")
        lines.append("# TYPE http_requests_total counter")
        for (key, count) in httpRequestsTotal {
            lines.append(
                "http_requests_total{method=\"\(key.method)\",path=\"\(key.path)\",status=\"\(key.status)\"} \(count)"
            )
        }
        return lines.joined(separator: "\n") + "\n"
    }
}
