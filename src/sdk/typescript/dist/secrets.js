// 本ファイルは k1s0 TypeScript SDK の Secrets 動詞統一 facade。
// `client.secrets.get(...)` 形式で SecretsService への呼出を提供する。
// SecretsFacade は SecretsService の動詞統一 facade。
export class SecretsFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    // Get はシークレット名で値（key=value マップ）と version を取得する。
    async get(name) {
        // raw client を生成する。
        const raw = this.client.rawSecrets();
        // RPC 呼出。
        const resp = await raw.get({
            name,
            context: this.client.tenantContext(),
        });
        // (values, version) を返却する。
        return { values: resp.values, version: resp.version };
    }
    // Rotate はシークレットのローテーション。新バージョンと旧バージョンを返す。
    async rotate(name, opts = {}) {
        // raw client を生成する。
        const raw = this.client.rawSecrets();
        // RPC 呼出。
        const resp = await raw.rotate({
            name,
            context: this.client.tenantContext(),
            gracePeriodSec: opts.gracePeriodSec ?? 3600,
            policy: opts.policy,
            idempotencyKey: opts.idempotencyKey ?? "",
        });
        // (newVersion, previousVersion) を返却する。
        return {
            newVersion: resp.newVersion,
            previousVersion: resp.previousVersion,
        };
    }
}
//# sourceMappingURL=secrets.js.map