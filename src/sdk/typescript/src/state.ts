// 本ファイルは k1s0 TypeScript SDK の State 動詞統一 facade。
// `client.state.save(...)` 形式で StateService への呼出を提供する。

import type { K1s0Client } from "./client.js";

// Save / Delete のオプション。
export interface SaveOptions {
  // 期待 ETag（楽観的排他、空文字は無条件）。
  expectedEtag?: string;
  // TTL 秒（0 / 省略は永続）。
  ttlSec?: number;
  // 冪等性キー（共通規約 §「冪等性と再試行」: 24h 重複抑止、空文字 / 省略で dedup 無効）。
  idempotencyKey?: string;
}

// StateFacade は StateService の動詞統一 facade。
export class StateFacade {
  // 親 Client への参照。
  private readonly client: K1s0Client;

  constructor(client: K1s0Client) {
    this.client = client;
  }

  // Get はキー単位の取得。未存在時は null を返す。
  async get(
    store: string,
    key: string,
  ): Promise<{ data: Uint8Array; etag: string } | null> {
    // raw client を生成する（毎回新規、connect-es は安価）。
    const raw = this.client.rawState();
    // RPC 呼出。
    const resp = await raw.get({
      store,
      key,
      context: this.client.tenantContext(),
    });
    // 未存在時は null。
    if (resp.notFound) {
      return null;
    }
    // 存在時は data / etag を返却する。
    return { data: resp.data, etag: resp.etag };
  }

  // Save はキー単位の保存。新 ETag を返す。
  async save(
    store: string,
    key: string,
    data: Uint8Array,
    opts: SaveOptions = {},
  ): Promise<string> {
    // raw client を生成する。
    const raw = this.client.rawState();
    // RPC 呼出。
    const resp = await raw.set({
      store,
      key,
      data,
      // 期待 ETag は省略時空文字（無条件保存）。
      expectedEtag: opts.expectedEtag ?? "",
      // TTL は省略時 0（永続）。
      ttlSec: opts.ttlSec ?? 0,
      idempotencyKey: opts.idempotencyKey ?? "",
      context: this.client.tenantContext(),
    });
    // 新 ETag を返却する。
    return resp.newEtag;
  }

  // Delete はキー単位の削除。expected_etag が空なら無条件。
  async delete(store: string, key: string, expectedEtag = ""): Promise<boolean> {
    // raw client を生成する。
    const raw = this.client.rawState();
    // RPC 呼出。
    const resp = await raw.delete({
      store,
      key,
      expectedEtag,
      context: this.client.tenantContext(),
    });
    // deleted フラグを返却する。
    return resp.deleted;
  }
}
