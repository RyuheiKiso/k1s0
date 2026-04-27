# tier3-bff Helm chart

tier3 Backend-for-Frontend（portal-bff / admin-bff 等）の汎用 Helm chart。
Go 製 GraphQL / REST BFF が tier1 / tier2 を集約し、frontend SPA に HTTP / GraphQL で公開する。

## 利用例

```sh
helm install portal-bff deploy/charts/tier3-bff \
  -n tier3-bff --create-namespace \
  --set service.name=portal-bff \
  --set image.repository=k1s0/k1s0/tier3-portal-bff \
  --set image.tag=v0.1.0 \
  --set ingress.host=portal.k1s0.example.com
```

## tier2-go-service との差異

- `oidc:` block を追加（Keycloak 連携、frontend セッション持ち）
- `ingress:` block を追加（Istio Ambient Gateway 連携で外部公開）
- label `k1s0.io/tier: tier3`、`k1s0.io/lang: go-bff`

## 関連設計

- `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/02_bff配置.md`
- ADR-TIER3-001（BFF パターン採用根拠）
