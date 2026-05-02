# @k1s0/backstage-plugin-scaffolder

`k1s0-scaffold` CLI を Backstage Scaffolder の Custom Action として公開する plugin（skeleton）。
Web UI からのフォーム入力で tier2/tier3 サービスを生成できるようにする。

## input schema

採用組織は本 package が export する `K1S0_SCAFFOLDER_INPUT_SCHEMA` を Backstage の
SoftwareTemplate（YAML）の `parameters` 節から参照し、Web UI フォームを自動生成する。

```yaml
# software-template.yaml の例
apiVersion: scaffolder.backstage.io/v1beta3
kind: Template
metadata:
  name: k1s0-scaffold
spec:
  parameters:
    - title: k1s0 service
      properties:
        # @k1s0/backstage-plugin-scaffolder の K1S0_SCAFFOLDER_INPUT_SCHEMA に整合
        serviceType:
          type: string
          enum: [tier2-go, tier2-dotnet, tier3-bff, tier3-web]
        name:
          type: string
          pattern: "^[a-z][a-z0-9-]*[a-z0-9]$"
        owner:
          type: string
  steps:
    - id: k1s0-scaffold
      name: Generate k1s0 service
      action: k1s0:scaffold        # 採用組織が createTemplateAction で登録
      input:
        serviceType: ${{ parameters.serviceType }}
        name: ${{ parameters.name }}
        owner: ${{ parameters.owner }}
```

## 採用組織が実装する Custom Action

```ts
import { createTemplateAction } from "@backstage/plugin-scaffolder-node";
import { spawn } from "child_process";
import { K1S0_SCAFFOLDER_PLUGIN_ID, K1S0_SCAFFOLDER_INPUT_SCHEMA } from "@k1s0/backstage-plugin-scaffolder";

export const k1s0ScaffoldAction = createTemplateAction({
  id: "k1s0:scaffold",
  schema: { input: K1S0_SCAFFOLDER_INPUT_SCHEMA },
  async handler(ctx) {
    // workspace ディレクトリで k1s0-scaffold CLI を起動する
    const { serviceType, name, owner, namespace } = ctx.input;
    const args = [serviceType, "--name", name, "--owner", owner, "--out", ctx.workspacePath];
    if (namespace) args.push("--namespace", namespace);
    // ... spawn k1s0-scaffold and stream stdout to ctx.logger
  },
});
```

## 関連

- ADR-BS-001（Backstage 採用）/ ADR-DEV-001（Golden Path）
- src/platform/scaffold/（k1s0-scaffold 本体、Rust crate、IMP-CODEGEN-SCF-030）
