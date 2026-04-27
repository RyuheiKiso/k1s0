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
}
//# sourceMappingURL=secrets.d.ts.map