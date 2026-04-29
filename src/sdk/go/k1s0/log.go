// 本ファイルは k1s0 Go SDK の Log 動詞統一 facade。
package k1s0

import (
	"context"
	"time"

	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// LogClient は LogService の動詞統一 facade。
type LogClient struct{ client *Client }

// Log は親 Client から LogClient を返す。
func (c *Client) Log() *LogClient { return c.log }

// Severity は OTel SeverityNumber と整合する重大度。利用者向けの type alias。
type Severity = logv1.Severity

// 重大度の便利定数。
const (
	SeverityTrace = logv1.Severity_TRACE
	SeverityDebug = logv1.Severity_DEBUG
	SeverityInfo  = logv1.Severity_INFO
	SeverityWarn  = logv1.Severity_WARN
	SeverityError = logv1.Severity_ERROR
	SeverityFatal = logv1.Severity_FATAL
)

// Send は単一エントリ送信。
// 動詞統一 facade（README サンプル）と整合: `c.Log().Info(ctx, "msg", attrs)`。
func (l *LogClient) Send(ctx context.Context, severity Severity, body string, attributes map[string]string) error {
	// proto Request を構築する。
	req := &logv1.SendLogRequest{
		Entry: &logv1.LogEntry{
			Timestamp:  timestamppb.New(time.Now().UTC()),
			Severity:   severity,
			Body:       body,
			Attributes: attributes,
		},
		Context: l.client.tenantContext(ctx),
	}
	// RPC 呼出。
	_, err := l.client.raw.Log.Send(ctx, req)
	return err
}

// Info は INFO 重大度のショートカット。
func (l *LogClient) Info(ctx context.Context, body string, attrs map[string]string) error {
	return l.Send(ctx, SeverityInfo, body, attrs)
}

// Warn は WARN 重大度のショートカット。
func (l *LogClient) Warn(ctx context.Context, body string, attrs map[string]string) error {
	return l.Send(ctx, SeverityWarn, body, attrs)
}

// Error は ERROR 重大度のショートカット。
func (l *LogClient) Error(ctx context.Context, body string, attrs map[string]string) error {
	return l.Send(ctx, SeverityError, body, attrs)
}

// Debug は DEBUG 重大度のショートカット。
func (l *LogClient) Debug(ctx context.Context, body string, attrs map[string]string) error {
	return l.Send(ctx, SeverityDebug, body, attrs)
}

// tenantContext は ctx に WithTenant で attach された per-request override を優先し、
// 未 attach の場合は client.cfg を fallback として TenantContext proto を構築する。
//
// per-request override の動機:
//   tier3 BFF など、1 SDK インスタンスで複数エンドユーザのリクエストを処理する経路では、
//   各リクエストの JWT tenant_id を SDK 呼出時に伝搬する必要がある。WithTenant(ctx, ...)
//   で attach された override が cfg より優先されることで、static cfg.TenantID を全
//   リクエストで共用してしまう越境を防ぐ（NFR-E-AC-003）。
func (c *Client) tenantContext(ctx context.Context) *commonv1.TenantContext {
	// per-request override（BFF middleware が attach する）を最優先で確認する。
	if ov, ok := tenantOverrideFromContext(ctx); ok {
		return &commonv1.TenantContext{
			TenantId:      ov.TenantID,
			Subject:       ov.Subject,
			CorrelationId: ov.CorrelationID,
		}
	}
	// fallback: static cfg を使う（既存利用経路の互換性を維持）。
	return &commonv1.TenantContext{
		TenantId: c.cfg.TenantID,
		Subject:  c.cfg.Subject,
	}
}
