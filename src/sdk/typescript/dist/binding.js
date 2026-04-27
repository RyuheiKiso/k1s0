// 本ファイルは k1s0 TypeScript SDK の Binding 動詞統一 facade。
import { createPromiseClient } from "@connectrpc/connect";
import { BindingService } from "./proto/k1s0/tier1/binding/v1/binding_service_connect.js";
/** BindingFacade は BindingService の動詞統一 facade。 */
export class BindingFacade {
    client;
    constructor(client) {
        this.client = client;
    }
    /** invoke は出力バインディング呼出。 */
    async invoke(name, operation, data, metadata = {}) {
        const raw = createPromiseClient(BindingService, this.client.transport);
        const resp = await raw.invoke({
            name,
            operation,
            data,
            metadata,
            context: this.client.tenantContext(),
        });
        return { data: resp.data, metadata: resp.metadata };
    }
}
//# sourceMappingURL=binding.js.map