package telemetry

import (
	"context"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

// Trace は名前付きスパンを開始し、fn を実行して終了する汎用トレースヘルパー。
// fn がエラーを返した場合、スパンにエラーを記録する。
// tracerName はスパンの計装名（通常はパッケージパス）。
func Trace(ctx context.Context, tracerName, spanName string, fn func(context.Context) error, opts ...trace.SpanStartOption) error {
	tracer := otel.Tracer(tracerName)
	ctx, span := tracer.Start(ctx, spanName, opts...)
	defer span.End()

	if err := fn(ctx); err != nil {
		// エラーをスパンに記録してから呼び出し元に返す
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return err
	}
	span.SetStatus(codes.Ok, "")
	return nil
}

// TraceWithResult は戻り値を持つ関数をトレースする汎用ヘルパー。
// fn がエラーを返した場合、スパンにエラーを記録する。
func TraceWithResult[T any](ctx context.Context, tracerName, spanName string, fn func(context.Context) (T, error), opts ...trace.SpanStartOption) (T, error) {
	tracer := otel.Tracer(tracerName)
	ctx, span := tracer.Start(ctx, spanName, opts...)
	defer span.End()

	result, err := fn(ctx)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return result, err
	}
	span.SetStatus(codes.Ok, "")
	return result, nil
}

// StartSpan はスパンを開始して context とスパンを返す便利関数。
// defer span.End() と組み合わせて使用する。
// 例: ctx, span := telemetry.StartSpan(ctx, "myservice", "operation")
//
//	defer span.End()
func StartSpan(ctx context.Context, tracerName, spanName string, opts ...trace.SpanStartOption) (context.Context, trace.Span) {
	return otel.Tracer(tracerName).Start(ctx, spanName, opts...)
}

// AddSpanAttributes は現在のスパンに属性を追加する。
// スパンが存在しない場合は何もしない（安全に呼び出せる）。
func AddSpanAttributes(ctx context.Context, attrs ...attribute.KeyValue) {
	span := trace.SpanFromContext(ctx)
	if span.IsRecording() {
		span.SetAttributes(attrs...)
	}
}

// RecordError は現在のスパンにエラーを記録する。
// スパンが存在しない場合は何もしない（安全に呼び出せる）。
func RecordError(ctx context.Context, err error) {
	if err == nil {
		return
	}
	span := trace.SpanFromContext(ctx)
	if span.IsRecording() {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
	}
}
