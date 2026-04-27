# Step 1: Sidecar 接続

`K1s0.Legacy.Sidecar` を既存 .NET Framework App と同 Pod に同居させ、Dapr sidecar から k1s0 公開 API を呼べる状態を作る。

## 前提

- Kubernetes クラスタに Windows Node が 1 台以上存在
- Dapr Control Plane が稼働中（`infra/dapr/control-plane/` 配置済）
- 既存 .NET Framework App の Docker image（Windows container 形式）が ghcr / 社内 registry に push 済

## 手順

1. `K1s0.Legacy.Sidecar` の Docker image を build する:

   ```bash
   docker build -f sidecars/K1s0.Legacy.Sidecar/Dockerfile.windows \
       -t ghcr.io/k1s0/legacy-sidecar:0.1.0 .
   docker push ghcr.io/k1s0/legacy-sidecar:0.1.0
   ```

2. Kubernetes Pod 定義に sidecar container を追加する。Dapr 注入 annotation を併置する:

   ```yaml
   apiVersion: v1
   kind: Pod
   metadata:
     name: legacy-app
     annotations:
       dapr.io/enabled: "true"
       dapr.io/app-id: "legacy-app"
       dapr.io/app-port: "80"
   spec:
     nodeSelector:
       kubernetes.io/os: windows
     containers:
       - name: legacy-app
         image: ghcr.io/your-org/legacy-app:1.0.0
       - name: k1s0-sidecar
         image: ghcr.io/k1s0/legacy-sidecar:0.1.0
   ```

3. デプロイ後、`/api/k1s0bridge/healthz` が 200 を返すことを確認する。

## 検証

```bash
kubectl exec -it legacy-app -c k1s0-sidecar -- powershell -c "Invoke-RestMethod http://localhost/api/k1s0bridge/healthz"
# -> @{ status = ok }
```
