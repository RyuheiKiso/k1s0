# tier3-bff テンプレート

tier3 SPA から呼ばれる BFF（Backend-for-Frontend）を Go + 最小 GraphQL で生成する。`k1s0-scaffold` 経路の標準動線。

## 利用方法

```bash
k1s0-scaffold new tier3-bff \
  --name portal-bff \
  --owner @k1s0/web \
  --system k1s0
```

## 生成内容

- `{{name}}/go.mod`
- `{{name}}/cmd/{{name}}/main.go` — POST /graphql の最小実装 + k1s0 SDK State.Get サンプル
- `{{name}}/Dockerfile`
- `{{name}}/catalog-info.yaml`
- `{{name}}/README.md`

## 関連

- [`examples/tier3-bff-graphql/`](../../../../examples/tier3-bff-graphql/)
- [`docs/02_構想設計/02_tier1設計/03_開発者体験/14_tier3開発者体験設計.md`](../../../../docs/02_構想設計/02_tier1設計/03_開発者体験/14_tier3開発者体験設計.md)
