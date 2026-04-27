// 本ファイルは k1s0 TypeScript SDK の PubSub 動詞統一 facade。
// `client.pubsub.publish(...)` 形式で PubSubService への呼出を提供する。

import type { K1s0Client } from "./client.js";

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
}
