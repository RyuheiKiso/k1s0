// 本ファイルは k1s0 TypeScript SDK の PubSub 動詞統一 facade（publish + subscribe）。

import type { K1s0Client } from "./client.js";
import type { Event } from "./proto/k1s0/tier1/pubsub/v1/pubsub_service_pb.js";

// Publish オプション。
export interface PublishOptions {
  // 冪等性キー（24h 重複抑止）。
  idempotencyKey?: string;
  // メタデータ（partition_key / trace_id 等）。
  metadata?: Record<string, string>;
}

// PubSubFacade は PubSubService の動詞統一 facade。
export class PubSubFacade {
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  // Publish は単発 Publish。Kafka offset を返す。
  async publish(
    topic: string,
    data: Uint8Array,
    contentType: string,
    opts: PublishOptions = {},
  ): Promise<bigint> {
    // raw client を生成する。
    const raw = this.client.rawPubSub();
    // RPC 呼出。
    const resp = await raw.publish({
      topic,
      data,
      contentType,
      idempotencyKey: opts.idempotencyKey ?? "",
      metadata: opts.metadata ?? {},
      context: this.client.tenantContext(),
    });
    // offset は proto3 int64 のため bigint を返却する。
    return resp.offset;
  }

  /** subscribe はトピックの購読。AsyncIterable<Event> を返す。
   *  利用例:
   *    for await (const event of client.pubsub.subscribe("orders", "consumer-A")) {
   *      handle(event);
   *    }
   */
  subscribe(topic: string, consumerGroup: string): AsyncIterable<Event> {
    const raw = this.client.rawPubSub();
    return raw.subscribe({
      topic,
      consumerGroup,
      context: this.client.tenantContext(),
    });
  }

  /**
   * bulkPublish は複数エントリの一括 Publish（FR-T1-PUBSUB-001）。
   * 各エントリの結果を個別に返す（部分成功あり、全体エラーにはしない）。
   */
  async bulkPublish(
    topic: string,
    entries: BulkPublishEntryInput[],
  ): Promise<Array<{ entryIndex: number; offset: bigint; errorCode: string }>> {
    const raw = this.client.rawPubSub();
    const tctx = this.client.tenantContext();
    // SDK の入力 → proto PublishRequest の配列に詰め替える。
    const pe = entries.map((e) => ({
      topic,
      data: e.data,
      contentType: e.contentType,
      idempotencyKey: e.idempotencyKey ?? "",
      metadata: e.metadata ?? {},
      context: tctx,
    }));
    const resp = await raw.bulkPublish({
      topic,
      entries: pe,
    });
    return resp.results.map((r) => ({
      entryIndex: r.entryIndex,
      offset: r.offset,
      errorCode: r.errorCode,
    }));
  }
}

/** BulkPublishEntryInput は bulkPublish の 1 件分の入力。 */
export interface BulkPublishEntryInput {
  /** データ本文。 */
  data: Uint8Array;
  /** Content-Type（application/json / application/protobuf 等）。 */
  contentType: string;
  /** 冪等性キー（24h 重複抑止）。省略可。 */
  idempotencyKey?: string;
  /** メタデータ（partition_key 等）。省略可。 */
  metadata?: Record<string, string>;
}
