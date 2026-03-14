// trace.go: OpenTelemetry を使ったトレーシングラッパー関数を提供する。
// Rust の #[k1s0_trace] マクロに相当する機能を Go で実現する。
// スパンの作成・エラー記録・DB操作・Kafkaコンシューマーのトレーシングをサポートする。
package telemetrymacros

import (
	"context"
	"fmt"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
)

// Trace はスパンを作成して fn を実行するラッパー関数。
// Rust の #[k1s0_trace] マクロに相当する。
// fn がエラーを返した場合、スパンに記録してからエラーを返す。
func Trace(ctx context.Context, name string, fn func(ctx context.Context) error) error {
	tracer := otel.Tracer("k1s0")
	ctx, span := tracer.Start(ctx, name)
	defer span.End()
	if err := fn(ctx); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return err
	}
	return nil
}

// TraceValue はスパンを作成して値を返す fn を実行するラッパー関数。
// ジェネリクスを使用して任意の型の戻り値をサポートする。
func TraceValue[T any](ctx context.Context, name string, fn func(ctx context.Context) (T, error)) (T, error) {
	tracer := otel.Tracer("k1s0")
	ctx, span := tracer.Start(ctx, name)
	defer span.End()
	v, err := fn(ctx)
	if err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
	}
	return v, err
}

// InstrumentDB はデータベース操作をトレースするラッパー関数。
// operation はDB操作名（例: "query", "exec", "scan"）、table はテーブル名。
func InstrumentDB(ctx context.Context, operation, table string, fn func(ctx context.Context) error) error {
	tracer := otel.Tracer("k1s0")
	spanName := fmt.Sprintf("db.%s %s", operation, table)
	ctx, span := tracer.Start(ctx, spanName,
		trace.WithSpanKind(trace.SpanKindClient),
	)
	defer span.End()
	// DB操作のセマンティクス属性を設定する。
	span.SetAttributes(
		attribute.String("db.operation", operation),
		attribute.String("db.sql.table", table),
		attribute.String("db.system", "postgresql"),
	)
	if err := fn(ctx); err != nil {
		span.RecordError(err)
		span.SetStatus(codes.Error, err.Error())
		return err
	}
	return nil
}

// KafkaTracingMiddleware は Kafka メッセージにトレースコンテキストを付与するミドルウェア。
// handler はメッセージを処理する関数、topic はKafkaトピック名。
// W3C TraceContext フォーマット（traceparent/tracestate）でヘッダーに伝播する。
func KafkaTracingMiddleware(topic string, handler func(ctx context.Context, payload []byte, headers map[string]string) error) func(ctx context.Context, payload []byte, headers map[string]string) error {
	return func(ctx context.Context, payload []byte, headers map[string]string) error {
		// ヘッダーからトレースコンテキストを抽出する。
		carrier := propagation.MapCarrier(headers)
		ctx = otel.GetTextMapPropagator().Extract(ctx, carrier)
		tracer := otel.Tracer("k1s0")
		ctx, span := tracer.Start(ctx, fmt.Sprintf("kafka.consume %s", topic),
			trace.WithSpanKind(trace.SpanKindConsumer),
		)
		defer span.End()
		// Kafkaメッセージングのセマンティクス属性を設定する。
		span.SetAttributes(
			attribute.String("messaging.system", "kafka"),
			attribute.String("messaging.destination", topic),
		)
		if err := handler(ctx, payload, headers); err != nil {
			span.RecordError(err)
			span.SetStatus(codes.Error, err.Error())
			return err
		}
		return nil
	}
}
