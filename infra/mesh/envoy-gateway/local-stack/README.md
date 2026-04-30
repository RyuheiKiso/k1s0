# infra/mesh/envoy-gateway/local-stack — Gateway API + Envoy Gateway 検証

ADR-MIG-002 (Envoy Gateway による north-south traffic) を kind ベースラインで
検証するための最小構成。

## 構成

- Gateway API CRD v1.2.1 (kubernetes-sigs/gateway-api)
- Envoy Gateway operator v1.2.4 (envoy-gateway-system namespace)
- GatewayClass `eg` (controller: gateway.envoyproxy.io/gatewayclass-controller)
- Gateway `k1s0-eg` (HTTP listener port 80)
- HTTPRoute `tier3-portal` (hostname portal.k1s0.local → tier3 portal-web:8080)

## install

```bash
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.2.1/standard-install.yaml
helm install envoy-gateway oci://docker.io/envoyproxy/gateway-helm \
  --version v1.2.4 -n envoy-gateway-system
```

## 検証 (2026-04-30)

```bash
$ kubectl port-forward -n envoy-gateway-system svc/envoy-envoy-gateway-system-k1s0-eg-... 51510:80 &
$ curl -i -H "Host: portal.k1s0.local" http://127.0.0.1:51510/
HTTP/1.1 200 OK
server: nginx/1.29.8
content-type: text/html
content-length: 322
<!doctype html>...
```

→ `Host: portal.k1s0.local` で envoy gateway → tier3 portal-web へのルーティングが
動作。HTTPRoute v1 規約 (parentRefs / hostnames / rules.matches.path /
backendRefs.port) すべて正しく機能。
