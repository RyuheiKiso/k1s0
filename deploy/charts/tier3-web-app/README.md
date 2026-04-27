# tier3-web-app Helm chart

tier3 web-app（React + Vite SPA）の汎用 Helm chart。
ビルド済 SPA を nginx-distroless で配信し、`/api/*` を tier3-bff にリバースプロキシ。

## 利用例

```sh
helm install portal deploy/charts/tier3-web-app \
  -n tier3-bff --create-namespace \
  --set service.name=portal \
  --set image.repository=k1s0/k1s0/tier3-portal \
  --set image.tag=v0.1.0 \
  --set bffUpstream=http://portal-bff.tier3-bff.svc.cluster.local:8080 \
  --set ingress.host=portal.k1s0.example.com
```

## 構造

- `Deployment`: nginx 実行（`/var/cache/nginx`, `/var/run` は emptyDir で書込み許可）
- `Service`: ClusterIP（外部公開は Istio Ambient Gateway 経由）
- `ConfigMap`-nginx: SPA fallback / `/api/` → BFF reverse proxy / 静的アセットキャッシュ
- `ServiceAccount`: 既定で作成
- `HPA`: `autoscaling.enabled=true` で有効化

## 関連設計

- `docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/01_web配置.md`
- ADR-TIER3-002（SPA + BFF パターン）
