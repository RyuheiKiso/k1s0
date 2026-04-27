# infra/environments/dev — 開発者ローカル overlay

開発者の単一ノード kind / k3d クラスタで k1s0 全体を起動するための最小構成 overlay。

## 適用方法

```bash
kubectl apply -k infra/environments/dev/
```

`tools/local-stack/up.sh` から自動呼出されることを想定（手動 apply は緊急時のみ）。

## prod base からの差分

| コンポーネント | base（prod） | dev overlay |
|---|---|---|
| CloudNativePG instances | 3 | 1（patches/cloudnativepg-instances-1.yaml） |
| Kafka KafkaNodePool replicas | 3 | 1（patches/kafka-broker-1.yaml） |
| Kafka storage | 100Gi | 5Gi（patches/kafka-broker-1.yaml） |
| ClusterIssuer 既定 | letsencrypt-prod | letsencrypt-staging（自己署名 fallback、values/cert-manager/） |
| ログ保持 | 30 日 | 7 日（values/loki/） |
| Trace 保持 | 7 日 | 3 日（values/tempo/） |

## values/ の Helm 上書き

`values/<component>/values.yaml` は Helmfile / Argo CD が `infra/<component>/values.yaml` の上に重ねて適用する想定の差分ファイル。リリース時点 は最小セットのみ（loki / tempo / cert-manager）を同梱し、必要に応じて拡充する。

## secrets/

SOPS 暗号化された Secret 配置場所。リリース時点 は `.gitkeep` のみ（dev では平文 Secret を namespace 直作りで OK としているため、暗号化必須は staging / prod）。
