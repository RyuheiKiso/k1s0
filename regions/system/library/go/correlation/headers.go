package correlation

// HTTP ヘッダー定数
const (
	HeaderCorrelationId = "X-Correlation-Id"
	HeaderTraceId       = "X-Trace-Id"
)

// ToHeaders は CorrelationContext を HTTP ヘッダーマップに変換する。
func ToHeaders(ctx CorrelationContext) map[string]string {
	headers := make(map[string]string)
	if !ctx.CorrelationId.IsEmpty() {
		headers[HeaderCorrelationId] = ctx.CorrelationId.String()
	}
	if !ctx.TraceId.IsEmpty() {
		headers[HeaderTraceId] = ctx.TraceId.String()
	}
	return headers
}

// FromHeaders は HTTP ヘッダーマップから CorrelationContext を生成する。
// ヘッダーが存在しない場合は自動生成する。
func FromHeaders(headers map[string]string) CorrelationContext {
	var correlationId CorrelationId
	var traceId TraceId

	if v, ok := headers[HeaderCorrelationId]; ok && v != "" {
		correlationId = ParseCorrelationId(v)
	} else {
		correlationId = NewCorrelationId()
	}

	if v, ok := headers[HeaderTraceId]; ok && v != "" {
		parsed, err := ParseTraceId(v)
		if err != nil {
			traceId = NewTraceId()
		} else {
			traceId = parsed
		}
	} else {
		traceId = NewTraceId()
	}

	return CorrelationContext{
		CorrelationId: correlationId,
		TraceId:       traceId,
	}
}
