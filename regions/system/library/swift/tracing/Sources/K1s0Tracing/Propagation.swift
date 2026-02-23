public func injectContext(into headers: inout [String: String], context: TraceContext, baggage: Baggage? = nil) {
    headers["traceparent"] = context.toTraceparent()
    if let baggage = baggage {
        let header = baggage.toHeader()
        if !header.isEmpty {
            headers["baggage"] = header
        }
    }
}

public func extractContext(from headers: [String: String]) -> (context: TraceContext?, baggage: Baggage) {
    let context = TraceContext.fromTraceparent(headers["traceparent"] ?? "")
    let baggage = Baggage.fromHeader(headers["baggage"] ?? "")
    return (context, baggage)
}
