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
}
//# sourceMappingURL=state.js.map