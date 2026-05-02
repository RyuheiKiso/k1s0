// 本ファイルは k1s0 TypeScript SDK の Audit 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import type { K1s0Client } from "./client.js";
import { AuditService } from "./proto/k1s0/tier1/audit/v1/audit_service_connect.js";
import {
  AuditEvent,
  ExportAuditChunk,
  ExportFormat,
} from "./proto/k1s0/tier1/audit/v1/audit_service_pb.js";
import { Timestamp } from "@bufbuild/protobuf";

/** AuditFacade は AuditService の動詞統一 facade。 */
export class AuditFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  /** record は監査イベント記録。auditId を返す。 */
  async record(
    actor: string,
    action: string,
    resource: string,
    outcome: string,
    attributes: Record<string, string> = {},
    idempotencyKey = "",
  ): Promise<string> {
    const raw = createPromiseClient(AuditService, this.client.transport);
    const resp = await raw.record({
      event: new AuditEvent({
        timestamp: Timestamp.now(),
        actor,
        action,
        resource,
        outcome,
        attributes,
      }),
      idempotencyKey,
      context: this.client.tenantContext(),
    });
    return resp.auditId;
  }

  /** query は監査イベント検索（時刻範囲 + filter）。 */
  async query(
    fromDate: Date,
    toDate: Date,
    filters: Record<string, string> = {},
    limit = 100,
  ): Promise<AuditEvent[]> {
    const raw = createPromiseClient(AuditService, this.client.transport);
    const resp = await raw.query({
      from: Timestamp.fromDate(fromDate),
      to: Timestamp.fromDate(toDate),
      filters,
      limit,
      context: this.client.tenantContext(),
    });
    return resp.events;
  }

  /**
   * export は Audit のサーバストリーミング エクスポート（FR-T1-AUDIT-003）。
   * 範囲 + フォーマット指定で逐次 chunk を AsyncIterable で返す。利用例:
   *   for await (const c of facade.export(undefined, undefined, ExportFormat.NDJSON, 0)) { ... }
   * fromDate / toDate に undefined を渡すと全範囲。
   * chunkBytes が 0 ならサーバ既定（65536）、上限は 1 MiB。
   */
  async *export(
    fromDate: Date | undefined,
    toDate: Date | undefined,
    format: ExportFormat = ExportFormat.NDJSON,
    chunkBytes = 0,
  ): AsyncGenerator<ExportAuditChunk> {
    const raw = createPromiseClient(AuditService, this.client.transport);
    const stream = raw.export({
      from: fromDate ? Timestamp.fromDate(fromDate) : undefined,
      to: toDate ? Timestamp.fromDate(toDate) : undefined,
      format,
      chunkBytes,
      context: this.client.tenantContext(),
    });
    for await (const chunk of stream) {
      yield chunk;
      if (chunk.isLast) {
        break;
      }
    }
  }

  /**
   * verifyChain は監査ハッシュチェーンの整合性を検証する（FR-T1-AUDIT-002）。
   * fromDate / toDate に未指定（undefined）を渡すと全範囲を対象にする。
   */
  async verifyChain(
    fromDate?: Date,
    toDate?: Date,
  ): Promise<VerifyChainResult> {
    const raw = createPromiseClient(AuditService, this.client.transport);
    const resp = await raw.verifyChain({
      from: fromDate ? Timestamp.fromDate(fromDate) : undefined,
      to: toDate ? Timestamp.fromDate(toDate) : undefined,
      context: this.client.tenantContext(),
    });
    return {
      valid: resp.valid,
      checkedCount: Number(resp.checkedCount),
      firstBadSequence: Number(resp.firstBadSequence),
      reason: resp.reason,
    };
  }
}

/** VerifyChain（FR-T1-AUDIT-002）の応答を SDK 利用者向けに整理した型。 */
export interface VerifyChainResult {
  /** チェーン整合性が取れていれば true。 */
  valid: boolean;
  /** 検証対象だったイベント件数。 */
  checkedCount: number;
  /** 不整合検出時、最初に失敗した sequence_number（1-based）。valid 時は 0。 */
  firstBadSequence: number;
  /** 不整合の理由。valid 時は空文字。 */
  reason: string;
}
