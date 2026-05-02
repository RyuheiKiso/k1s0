// 本ファイルは k1s0 TypeScript SDK の Log 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { LogService } from "./proto/k1s0/tier1/log/v1/log_service_connect.js";
import { Severity, LogEntry } from "./proto/k1s0/tier1/log/v1/log_service_pb.js";
import { Timestamp } from "@bufbuild/protobuf";
/** LogFacade は LogService の動詞統一 facade。 */
export class LogFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** send は単一エントリ送信。 */
    async send(severity, body, attributes = {}) {
        const raw = createPromiseClient(LogService, this.client.transport);
        await raw.send({
            entry: new LogEntry({
                timestamp: Timestamp.now(),
                severity,
                body,
                attributes,
            }),
            context: this.client.tenantContext(),
        });
    }
    info = (body, attrs = {}) => this.send(Severity.INFO, body, attrs);
    warn = (body, attrs = {}) => this.send(Severity.WARN, body, attrs);
    error = (body, attrs = {}) => this.send(Severity.ERROR, body, attrs);
    debug = (body, attrs = {}) => this.send(Severity.DEBUG, body, attrs);
    /**
     * bulkSend は LogEntry の一括送信（FR-T1-LOG-* 共通、send の高スループット版）。
     * 各 entry の timestamp が省略されていれば呼出時刻を自動設定する。
     * 戻り値は { accepted, rejected }（rejected は PII / schema 違反による却下件数）。
     */
    async bulkSend(entries) {
        const raw = createPromiseClient(LogService, this.client.transport);
        const now = Timestamp.now();
        const pe = entries.map((e) => new LogEntry({
            timestamp: e.timestamp ? Timestamp.fromDate(e.timestamp) : now,
            severity: e.severity,
            body: e.body,
            attributes: e.attributes ?? {},
        }));
        const resp = await raw.bulkSend({
            entries: pe,
            context: this.client.tenantContext(),
        });
        return { accepted: resp.accepted, rejected: resp.rejected };
    }
}
//# sourceMappingURL=log.js.map