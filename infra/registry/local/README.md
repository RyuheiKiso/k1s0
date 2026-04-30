# infra/registry/local — Kyverno ImageVerify 検証用ローカル OCI レジストリ

ADR-CICD-003 / ADR-SUP-001 の cosign signature 検証を実 cluster 上で確認するため
の最小 OCI レジストリ。production では GHCR / ECR / Artifactory を使う想定だが、
kind ローカル環境では in-cluster registry が必要。

## 構成

- `registry:2.8.3` 1 replica (emptyDir、データは Pod 寿命依存)
- service: registry.registry.svc.cluster.local:5000 (HTTP only)

## 検証 (2026-04-30)

1. tier1-state image を local registry に push
2. cosign v2.4.3 (in-cluster Pod から実行) で `--allow-insecure-registry`
   `--tlog-upload=false` 付きで sign。tag list に `.sig` 追加を確認
3. `infra/security/kyverno/image-verify.yaml` の ClusterPolicy が:
   - **Unsigned image**: registry.../k1s0-foobar:unsigned を deploy 試行
     → admission blocked (`no signatures found`)
   - **Signed image (digest 指定)**: registry.../k1s0-tier1-state@sha256:...
     → admission **passed**（ErrImagePull は kubelet の registry DNS 解決
     問題で本検証スコープ外）

## Kyverno 設定

`kyverno-admission-controller` の args に `--allowInsecureRegistry=true` を
patch 済（HTTP-only registry 用）。production では HTTPS registry + cert を
信頼する設定に切替。
