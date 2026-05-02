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

  // BulkGet は複数キーの一括取得（FR-T1-STATE-003）。
  // 1 回の呼出で最大 100 キー（tier1 側で強制、超過は ResourceExhausted）。
  // 返却は キー → { data, etag, found } の Map。found=false は未存在。
  async bulkGet(
    store: string,
    keys: string[],
  ): Promise<Map<string, { data: Uint8Array; etag: string; found: boolean }>> {
    const raw = this.client.rawState();
    const resp = await raw.bulkGet({
      store,
      keys,
      context: this.client.tenantContext(),
    });
    const out = new Map<string, { data: Uint8Array; etag: string; found: boolean }>();
    for (const [k, r] of Object.entries(resp.results)) {
      out.set(k, { data: r.data, etag: r.etag, found: !r.notFound });
    }
    return out;
  }

  // Transact はトランザクション境界付き複数操作（FR-T1-STATE-005）。
  // 全操作が成功するか全て失敗するの 2 値。最大 10 操作 / トランザクション。
  // ops は { kind: "set" | "delete", key, data?, expectedEtag?, ttlSec? } の配列。
  async transact(store: string, ops: TransactOpInput[]): Promise<boolean> {
    const raw = this.client.rawState();
    // SDK の TransactOpInput を proto TransactOp（oneof）に詰め替える。
    const operations = ops.map((o) => {
      if (o.kind === "set") {
        return {
          op: {
            case: "set" as const,
            value: {
              store,
              key: o.key,
              data: o.data ?? new Uint8Array(),
              expectedEtag: o.expectedEtag ?? "",
              ttlSec: o.ttlSec ?? 0,
              idempotencyKey: "",
              context: undefined,
            },
          },
        };
      }
      return {
        op: {
          case: "delete" as const,
          value: {
            store,
            key: o.key,
            expectedEtag: o.expectedEtag ?? "",
            context: undefined,
          },
        },
      };
    });
    const resp = await raw.transact({
      store,
      operations,
      context: this.client.tenantContext(),
    });
    return resp.committed;
  }
}

// TransactOpInput は Transact の 1 操作の入力。
export interface TransactOpInput {
  // "set" or "delete"。
  kind: "set" | "delete";
  // 対象キー。
  key: string;
  // 値本文（kind="set" でのみ意味を持つ）。
  data?: Uint8Array;
  // 期待 ETag（楽観的排他、空 / 省略は無条件）。
  expectedEtag?: string;
  // TTL 秒（kind="set" でのみ意味を持つ、0 / 省略は永続）。
  ttlSec?: number;
}
