// 本ファイルは k1s0 Go SDK の Audit 動詞統一 facade。
package k1s0

import (
	"context"
	"io"
	"time"

	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// AuditClient は AuditService の動詞統一 facade。
type AuditClient struct{ client *Client }

// Audit は親 Client から AuditClient を返す。
func (c *Client) Audit() *AuditClient { return c.audit }

// Record は監査イベント記録。WORM ストアに append-only、audit_id を返す。
// 共通規約 §「冪等性と再試行」: idempotencyKey が空でなければ tier1 が 24h dedup
// する（hash chain への二重追記を防ぐ）。空文字なら毎回新 entry が作られる。
func (a *AuditClient) Record(ctx context.Context, actor, action, resource, outcome string, attributes map[string]string, idempotencyKey string) (string, error) {
	resp, e := a.client.raw.Audit.Record(ctx, &auditv1.RecordAuditRequest{
		Event: &auditv1.AuditEvent{
			Timestamp:  timestamppb.New(time.Now().UTC()),
			Actor:      actor,
			Action:     action,
			Resource:   resource,
			Outcome:    outcome,
			Attributes: attributes,
		},
		IdempotencyKey: idempotencyKey,
		Context:        a.client.tenantContext(ctx),
	})
	if e != nil {
		return "", e
	}
	return resp.GetAuditId(), nil
}

// Query は監査イベント検索。範囲 + filter で取得、出力には PII Mask が自動適用される。
func (a *AuditClient) Query(ctx context.Context, from, to time.Time, filters map[string]string, limit int32) ([]*auditv1.AuditEvent, error) {
	resp, e := a.client.raw.Audit.Query(ctx, &auditv1.QueryAuditRequest{
		From:    timestamppb.New(from),
		To:      timestamppb.New(to),
		Filters: filters,
		Limit:   limit,
		Context: a.client.tenantContext(ctx),
	})
	if e != nil {
		return nil, e
	}
	return resp.GetEvents(), nil
}

// VerifyChainResult は VerifyChain RPC の応答を SDK 利用者向けに集約した型。
type VerifyChainResult struct {
	// チェーン整合性が取れていれば true。
	Valid bool
	// 検証対象だったイベント件数。
	CheckedCount int64
	// 不整合検出時、最初に失敗したグローバル sequence_number（1-based）。Valid 時は 0。
	FirstBadSequence int64
	// 不整合の理由。Valid 時は空文字。
	Reason string
}

// ExportFormat は Audit Export のフォーマット選択（CSV / NDJSON / JSON 配列）。
type ExportFormat = auditv1.ExportFormat

// 既定 / CSV / NDJSON / JSON 配列の便利定数（proto enum と整合）。
const (
	// ExportFormatUnspecified はサーバ側で NDJSON にフォールバックする。
	ExportFormatUnspecified = auditv1.ExportFormat_EXPORT_FORMAT_UNSPECIFIED
	// ExportFormatCSV は RFC 4180 準拠 CSV（ヘッダ行を最初の chunk）。
	ExportFormatCSV = auditv1.ExportFormat_EXPORT_FORMAT_CSV
	// ExportFormatNDJSON は改行区切り JSON（Splunk / fluentd 取込向け）。
	ExportFormatNDJSON = auditv1.ExportFormat_EXPORT_FORMAT_NDJSON
	// ExportFormatJSONArray は単一 JSON 配列（小規模向け）。
	ExportFormatJSONArray = auditv1.ExportFormat_EXPORT_FORMAT_JSON_ARRAY
)

// AuditChunk は Export RPC の 1 chunk（SDK 利用者向け、proto ExportAuditChunk と等価）。
type AuditChunk struct {
	// フォーマット済みデータ本体（UTF-8 想定）。
	Data []byte
	// 0 起点のチャンク連番。
	Sequence int64
	// この chunk に含まれる event 数。
	EventCount int64
	// ストリーム末尾の chunk なら true。
	IsLast bool
}

// AuditChunkHandler は Export で受信した各 chunk を処理するコールバック。
// non-nil error を返すと Export は中断される。
type AuditChunkHandler func(chunk AuditChunk) error

// Export は Audit のサーバストリーミング エクスポート（FR-T1-AUDIT-003）。
// 範囲 + フォーマットを指定し、tier1 が PII Mask 適用 + 改行区切りで chunk 列を返す。
// chunkBytes が 0 ならサーバ既定（65536）、上限は 1 MiB。
// from / to が zero 時刻なら全範囲（tier1 側で nil 扱い）。
func (a *AuditClient) Export(ctx context.Context, from, to time.Time, format ExportFormat, chunkBytes int32, handler AuditChunkHandler) error {
	// proto Request を構築する（zero time は nil 渡し）。
	req := &auditv1.ExportAuditRequest{
		Format:     format,
		ChunkBytes: chunkBytes,
		Context:    a.client.tenantContext(ctx),
	}
	if !from.IsZero() {
		req.From = timestamppb.New(from)
	}
	if !to.IsZero() {
		req.To = timestamppb.New(to)
	}
	// 生成 stub 経由でストリーミング呼出。
	stream, err := a.client.raw.Audit.Export(ctx, req)
	if err != nil {
		return err
	}
	// 各 chunk を受信して handler に渡す。
	for {
		c, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}
		if err := handler(AuditChunk{
			Data:       c.GetData(),
			Sequence:   c.GetSequence(),
			EventCount: c.GetEventCount(),
			IsLast:     c.GetIsLast(),
		}); err != nil {
			return err
		}
		// IsLast=true は handler 呼出後に明示的に終了する（EOF と二重に来ない backend のため）。
		if c.GetIsLast() {
			return nil
		}
	}
}

// VerifyChain は監査ハッシュチェーンの整合性を検証する（FR-T1-AUDIT-002）。
// from / to が zero 時刻なら全範囲を対象にする（gRPC 側で nil 扱い）。
func (a *AuditClient) VerifyChain(ctx context.Context, from, to time.Time) (VerifyChainResult, error) {
	// proto Request を構築する。zero time は nil で渡し、tier1 側が "未指定" として解釈する。
	req := &auditv1.VerifyChainRequest{Context: a.client.tenantContext(ctx)}
	if !from.IsZero() {
		req.From = timestamppb.New(from)
	}
	if !to.IsZero() {
		req.To = timestamppb.New(to)
	}
	// gRPC 呼出。
	resp, err := a.client.raw.Audit.VerifyChain(ctx, req)
	if err != nil {
		return VerifyChainResult{}, err
	}
	// SDK 型に詰め替えて返す。
	return VerifyChainResult{
		Valid:            resp.GetValid(),
		CheckedCount:     resp.GetCheckedCount(),
		FirstBadSequence: resp.GetFirstBadSequence(),
		Reason:           resp.GetReason(),
	}, nil
}
