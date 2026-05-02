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
    // BulkGet はテナント配下の全シークレットを一括取得する（FR-T1-SECRETS-001）。
    // 戻り値は シークレット名 → { values, version } の Map。
    async bulkGet() {
        const raw = this.client.rawSecrets();
        const resp = await raw.bulkGet({
            context: this.client.tenantContext(),
        });
        const out = new Map();
        for (const [name, sec] of Object.entries(resp.results)) {
            out.set(name, { values: sec.values, version: sec.version });
        }
        return out;
    }
    // Encrypt は Transit Engine 経由の暗号化（FR-T1-SECRETS-003）。
    // keyName は tier1 が <tenant_id>.<keyName> で自動 prefix する。
    // aad は GCM 追加認証データ（同じ aad を Decrypt 時にも渡す必要あり）。
    async encrypt(keyName, plaintext, aad = new Uint8Array()) {
        const raw = this.client.rawSecrets();
        const resp = await raw.encrypt({
            context: this.client.tenantContext(),
            keyName,
            plaintext,
            aad,
        });
        return { ciphertext: resp.ciphertext, keyVersion: resp.keyVersion };
    }
    // Decrypt は Transit Engine 経由の復号（FR-T1-SECRETS-003）。
    // keyName / aad は Encrypt 時と同じ値を渡すこと。
    async decrypt(keyName, ciphertext, aad = new Uint8Array()) {
        const raw = this.client.rawSecrets();
        const resp = await raw.decrypt({
            context: this.client.tenantContext(),
            keyName,
            ciphertext,
            aad,
        });
        return { plaintext: resp.plaintext, keyVersion: resp.keyVersion };
    }
    // RotateKey は Transit Engine の鍵をローテーションする（FR-T1-SECRETS-003）。
    // 既存版は保持され、その鍵で暗号化された ciphertext は引き続き Decrypt 可能。
    async rotateKey(keyName) {
        const raw = this.client.rawSecrets();
        const resp = await raw.rotateKey({
            context: this.client.tenantContext(),
            keyName,
        });
        return {
            newVersion: resp.newVersion,
            previousVersion: resp.previousVersion,
            rotatedAtMs: Number(resp.rotatedAtMs),
        };
    }
}
//# sourceMappingURL=secrets.js.map