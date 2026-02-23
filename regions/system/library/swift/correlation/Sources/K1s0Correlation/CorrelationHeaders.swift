/// HTTP/gRPC ヘッダーを介した相関コンテキスト伝播。
public enum CorrelationHeaders: Sendable {
    public static let correlationIdKey = "x-correlation-id"
    public static let traceIdKey = "x-trace-id"

    /// コンテキストをヘッダー配列に変換する。
    public static func toHeaders(_ context: CorrelationContext) -> [(String, String)] {
        var headers: [(String, String)] = [
            (correlationIdKey, context.correlationId.asString),
        ]
        if let traceId = context.traceId {
            headers.append((traceIdKey, traceId.asString))
        }
        return headers
    }

    /// ヘッダー配列からコンテキストを生成する（キーは大文字小文字不問）。
    public static func fromHeaders(_ headers: [(String, String)]) -> CorrelationContext {
        var correlationId: CorrelationId?
        var traceId: TraceId?
        for (key, value) in headers {
            switch key.lowercased() {
            case correlationIdKey:
                correlationId = CorrelationId(string: value)
            case traceIdKey:
                traceId = TraceId.from(string: value)
            default:
                break
            }
        }
        let cid = correlationId ?? CorrelationId()
        let ctx = CorrelationContext(correlationId: cid)
        if let tid = traceId {
            return ctx.withTraceId(tid)
        }
        return ctx
    }
}
