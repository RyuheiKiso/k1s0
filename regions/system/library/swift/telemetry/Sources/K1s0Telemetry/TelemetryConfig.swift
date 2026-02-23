/// テレメトリ設定。
public struct TelemetryConfig: Sendable {
    public let serviceName: String
    public let version: String
    public let tier: String
    public let environment: String
    public let traceEndpoint: String?
    public let sampleRate: Double
    public let logLevel: LogLevel

    public init(
        serviceName: String,
        version: String,
        tier: String,
        environment: String,
        traceEndpoint: String? = nil,
        sampleRate: Double = 1.0,
        logLevel: LogLevel = .info
    ) {
        self.serviceName = serviceName
        self.version = version
        self.tier = tier
        self.environment = environment
        self.traceEndpoint = traceEndpoint
        self.sampleRate = sampleRate
        self.logLevel = logLevel
    }
}

/// ログレベル。
public enum LogLevel: String, Sendable {
    case debug
    case info
    case warn
    case error
}
