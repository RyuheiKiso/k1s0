/// 相関コンテキスト。
public struct CorrelationContext: Sendable {
    public let correlationId: CorrelationId
    public let traceId: TraceId?

    public init() {
        self.correlationId = CorrelationId()
        self.traceId = nil
    }

    public init(correlationId: CorrelationId) {
        self.correlationId = correlationId
        self.traceId = nil
    }

    public func withTraceId(_ traceId: TraceId) -> CorrelationContext {
        CorrelationContext(correlationId: correlationId, traceId: traceId)
    }

    private init(correlationId: CorrelationId, traceId: TraceId?) {
        self.correlationId = correlationId
        self.traceId = traceId
    }
}
