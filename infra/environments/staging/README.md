# infra/environments/staging — staging overlay

prod 同構成で replica / リソースを 1/3 に縮小した overlay。実顧客データを含まない擬似プロダクション環境として、Blue/Green で prod デプロイ前の最終検証に使う。

## prod base からの差分

| コンポーネント | base（prod） | staging overlay |
|---|---|---|
| CloudNativePG instances | 3 | 2（patches/cloudnativepg-instances-2.yaml） |
| Kafka storage | 100Gi | 50Gi（patches/kafka-storage-50gi.yaml、replica 数は HA 維持） |
| ClusterIssuer 既定 | letsencrypt-prod | letsencrypt-staging（rate limit 回避、values/cert-manager/） |
| ログ保持 | 30 日 | 30 日（prod と同） |
| Trace 保持 | 7 日 | 7 日（prod と同） |

## 何を staging で検証するか

- 24 時間連続稼働（夜間バッチ・スケジュール処理を含む時間依存性）
- k6 / Gatling による負荷試験
- DriftDetection による prod base からの想定外差分検知
- Argo CD ApplicationSet の同期順序保証（staging → prod）

## 適用方法

`deploy/apps/application-sets/` の Argo CD ApplicationSet が `infra/environments/staging/` を target として宣言的に同期する。手動 apply は禁止（DS-OPS-ENV-009、kubectl 直接操作禁止）。
