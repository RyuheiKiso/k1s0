# tests/contract — tier 間 API 契約整合テスト

Pact（Consumer-Driven Contract）と OpenAPI で tier 間の API 契約整合を検証する。

## 構造

```text
contract/
├── README.md              # 本ファイル
├── pact/
│   ├── consumers/         # tier3 (BFF / SPA) の consumer 期待値
│   │   ├── portal-bff/
│   │   └── admin-bff/
│   ├── providers/         # tier1 / tier2 の provider verification
│   │   ├── tier1-state/
│   │   └── tier2-payroll/
│   └── broker-config.yaml # Pact Broker 設定
└── openapi-contract/      # OpenAPI spec ベースの契約検証（schemathesis / dredd）
    └── tier1-openapi-spec.yaml
```

## Pact フロー

1. **Consumer** (tier3 BFF) が期待する provider レスポンスを `consumers/<bff>/` 配下に記録（pact-go / pact-net 等）
2. **Pact Broker** (`tools/local-stack/` 上で起動) に Pact ファイルを publish
3. **Provider** (tier1 / tier2) が Broker から Pact を pull し、実装で再生して契約違反を検出
4. CI で Provider verification が落ちた PR は block

## OpenAPI 契約

[`openapi-contract/tier1-openapi-spec.yaml`](openapi-contract/tier1-openapi-spec.yaml) は `tools/codegen/openapi/run.sh` が生成した tier1 公開 12 API の OpenAPI v2 spec のコピー。tier3 BFF が spec 通りに応答するかを `schemathesis` / `dredd` で検証する。

spec の正典は `docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml` であり、本ディレクトリのコピーは CI で diff 検出する（drift 防止）。

## CI

`.github/workflows/_reusable-test.yml` の `contract` job で PR 毎に実行。Pact Broker は GitHub Actions の services として起動するか、共有 Broker（k1s0-pact-broker.k1s0.example.com）に publish する。
