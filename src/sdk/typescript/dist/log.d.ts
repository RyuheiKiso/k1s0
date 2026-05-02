import type { K1s0Client } from "./client.js";
import { Severity } from "./proto/k1s0/tier1/log/v1/log_service_pb.js";
/** LogFacade は LogService の動詞統一 facade。 */
export declare class LogFacade {
    private readonly client;
    constructor(client: K1s0Client);
    /** send は単一エントリ送信。 */
    send(severity: Severity, body: string, attributes?: Record<string, string>): Promise<void>;
    info: (body: string, attrs?: Record<string, string>) => Promise<void>;
    warn: (body: string, attrs?: Record<string, string>) => Promise<void>;
    error: (body: string, attrs?: Record<string, string>) => Promise<void>;
    debug: (body: string, attrs?: Record<string, string>) => Promise<void>;
    /**
     * bulkSend は LogEntry の一括送信（FR-T1-LOG-* 共通、send の高スループット版）。
     * 各 entry の timestamp が省略されていれば呼出時刻を自動設定する。
     * 戻り値は { accepted, rejected }（rejected は PII / schema 違反による却下件数）。
     */
    bulkSend(entries: Array<{
        severity: Severity;
        body: string;
        attributes?: Record<string, string>;
        timestamp?: Date;
    }>): Promise<{
        accepted: number;
        rejected: number;
    }>;
}
//# sourceMappingURL=log.d.ts.map