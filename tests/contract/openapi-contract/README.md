# tests/contract/openapi-contract

`tools/codegen/openapi/run.sh` が生成した tier1 公開 12 API の OpenAPI v2 spec を保管し、tier3 BFF / tier1 facade の応答を schemathesis / dredd で契約検証する。

## spec 正典

[`docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml`](../../../docs/02_構想設計/02_tier1設計/openapi/v1/k1s0-tier1.swagger.yaml)

本ディレクトリの `tier1-openapi-spec.yaml` は上記の物理コピーで、CI が `tools/codegen/openapi/run.sh --check` で diff 検出して drift を防止する。

## 利用例（schemathesis）

```bash
schemathesis run \
  --base-url=https://api.k1s0.example.com \
  tests/contract/openapi-contract/tier1-openapi-spec.yaml
```
