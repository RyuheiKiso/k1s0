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
    /**
     * getDynamic は動的 Secret 発行（FR-T1-SECRETS-002）。
     * engine="postgres" / "mysql" / "kafka" 等の OpenBao Database Engine 種別を指定する。
     * ttlSec=0 で既定 1 時間（3600）、上限 24 時間（86400）に clamp される。
     */
    async getDynamic(engine, role, ttlSec = 0) {
        const raw = this.client.rawSecrets();
        const resp = await raw.getDynamic({
            engine,
            role,
            ttlSec,
            context: this.client.tenantContext(),
        });
        return {
            values: resp.values,
            leaseId: resp.leaseId,
            ttlSec: resp.ttlSec,
            issuedAtMs: Number(resp.issuedAtMs),
        };
    }
}
//# sourceMappingURL=secrets.js.map