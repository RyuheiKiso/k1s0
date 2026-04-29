import type { K1s0Client } from "./client.js";
export interface RotateOptions {
    gracePeriodSec?: number;
    policy?: string;
    idempotencyKey?: string;
}
export declare class SecretsFacade {
    private readonly client;
    constructor(client: K1s0Client);
    get(name: string): Promise<{
        values: Record<string, string>;
        version: number;
    }>;
    rotate(name: string, opts?: RotateOptions): Promise<{
        newVersion: number;
        previousVersion: number;
    }>;
    /**
     * getDynamic は動的 Secret 発行（FR-T1-SECRETS-002）。
     * engine="postgres" / "mysql" / "kafka" 等の OpenBao Database Engine 種別を指定する。
     * ttlSec=0 で既定 1 時間（3600）、上限 24 時間（86400）に clamp される。
     */
    getDynamic(engine: string, role: string, ttlSec?: number): Promise<DynamicSecret>;
}
/**
 * 動的 Secret 発行（FR-T1-SECRETS-002）の応答を SDK 利用者向けに整理した型。
 */
export interface DynamicSecret {
    /** credential 一式（"username" / "password" など、engine 別の field）。 */
    values: Record<string, string>;
    /** OpenBao の lease ID（renewal / revoke 用）。 */
    leaseId: string;
    /** 実際に付与された TTL 秒（要求値から ceiling までクランプされる）。 */
    ttlSec: number;
    /** 発効時刻（Unix epoch ミリ秒）。 */
    issuedAtMs: number;
}
//# sourceMappingURL=secrets.d.ts.map