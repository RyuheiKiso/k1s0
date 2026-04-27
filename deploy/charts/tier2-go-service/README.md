# tier2-go-service Helm chart

tier2 Go service（notification-hub / stock-reconciler 等）の汎用 Helm chart。
各 service は overlay 用 values.yaml で `service.name` / `image.repository` を上書きする。

## 利用例

```sh
helm install notification-hub deploy/charts/tier2-go-service \
  -n tier2-services --create-namespace \
  --set service.name=notification-hub \
  --set image.repository=k1s0/k1s0/tier2-notification-hub \
  --set image.tag=v0.1.0
```

## 関連設計

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/03_go_services配置.md`
- ADR-TIER1-003（tier2/tier3 から tier1 内部言語不可視）
