# infra/environments/prod — prod overlay

`infra/` 直下の base がそのまま prod を表現する設計（DS-OPS-ENV-007）。`patches/` は原則空、env 識別ラベルと SOPS 復号 secret のみが overlay の役割となる。

## 構成

| コンポーネント | 値（base = prod） |
|---|---|
| CloudNativePG instances | 3（HA primary 1 + standby 2） |
| Kafka KafkaNodePool replicas | 3（KRaft controller + broker dual-role） |
| ClusterIssuer 既定 | letsencrypt-prod |
| ログ保持 | 30 日（Loki SimpleScalable + S3） |
| Trace 保持 | 7 日（Tempo distributed + S3） |
| メトリクス保持 | 13 ヶ月（Mimir distributed + S3） |

## 適用方法

`deploy/apps/application-sets/` の Argo CD ApplicationSet が `infra/environments/prod/` を target として宣言的に同期する。手動承認必須、staging で 24 時間連続稼働検証を通過しない変更は同期されない構造ガード（DS-OPS-ENV-007）。

## SOPS / 緊急時操作

`secrets/` に置く Secret は SOPS + AGE で暗号化し、AGE 鍵は `ops/oncall/sops-key/` 配下で運用チームのみがアクセス可能。kubectl 直接操作は SEV1 インシデント時の例外を除き禁止（DS-OPS-ENV-009）、緊急変更は 24 時間以内に事後 PR で Git に反映する義務を負う。
