// 本ファイルは k1s0 TypeScript SDK の State 動詞統一 facade。
// `client.state.save(...)` 形式で StateService への呼出を提供する。
// StateFacade は StateService の動詞統一 facade。
export class StateFacade {
    // 親 Client への参照。
    client;
    constructor(client) {
        this.client = client;
    }
    // Get はキー単位の取得。未存在時は null を返す。
    async get(store, key) {
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
    async save(store, key, data, opts = {}) {
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
    async delete(store, key, expectedEtag = "") {
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
    async bulkGet(store, keys) {
        const raw = this.client.rawState();
        const resp = await raw.bulkGet({
            store,
            keys,
            context: this.client.tenantContext(),
        });
        const out = new Map();
        for (const [k, r] of Object.entries(resp.results)) {
            out.set(k, { data: r.data, etag: r.etag, found: !r.notFound });
        }
        return out;
    }
    // Transact はトランザクション境界付き複数操作（FR-T1-STATE-005）。
    // 全操作が成功するか全て失敗するの 2 値。最大 10 操作 / トランザクション。
    // ops は { kind: "set" | "delete", key, data?, expectedEtag?, ttlSec? } の配列。
    async transact(store, ops) {
        const raw = this.client.rawState();
        // SDK の TransactOpInput を proto TransactOp（oneof）に詰め替える。
        const operations = ops.map((o) => {
            if (o.kind === "set") {
                return {
                    op: {
                        case: "set",
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
                    case: "delete",
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
//# sourceMappingURL=state.js.map