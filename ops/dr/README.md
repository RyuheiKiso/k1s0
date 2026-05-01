# dr — Disaster Recovery

本ディレクトリは Disaster Recovery のシナリオ・自動化スクリプト・演習記録を集約する。
[`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md`](../../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md) §「dr/ の構造」、
[`docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md) §「DS-OPS-ENV-012 DR ゼロスタート再構築リハーサル」、
NFR-A-REC-001（RTO 4h）/ NFR-A-CONT-001 に対応する。

## 配置

```text
dr/
├── README.md                      # 本ファイル
├── scenarios/
│   ├── RB-DR-001-cluster-rebuild.md      # クラスタ全壊 → ゼロから再構築（Runbook）
│   ├── pg-restore.md                     # CNPG クラスタ単独リストア
│   ├── kafka-topic-restore.md            # Kafka トピック消失からの MirrorMaker 復元
│   └── minio-tenant-restore.md           # MinIO Tenant 消失からの archive 復元
├── scripts/                       # 復旧自動化スクリプト
│   ├── restore-pg-from-barman.sh
│   ├── restore-minio-from-archive.sh
│   └── rebuild-cluster-from-scratch.sh
└── drills/                        # DR 演習記録（半期 1 回 / 四半期 table-top）
    └── DR-drill-YYYY-Qn.md        # 命名規則
```

## scenarios/ の責務

各シナリオは「特定コンポーネント or クラスタ全体の喪失からの復旧」を扱う。
[`RB-DR-001`](scenarios/RB-DR-001-cluster-rebuild.md) はクラスタ全壊シナリオで、他 3 シナリオは部分復旧。

| シナリオ | 想定 RPO | 想定 RTO | 主要手順 |
|------|------|------|------|
| RB-DR-001 cluster-rebuild | 24h | 4h | DR クラスタブートストラップ + 全データリストア + DNS 切替 |
| pg-restore | 1h | 1h | Barman アーカイブから単一 CNPG cluster をリストア |
| kafka-topic-restore | N/A（消失時のみ） | 30 分 | MirrorMaker で別クラスタからミラー、または `kafka-reassign-partitions` で再配置 |
| minio-tenant-restore | 24h | 2h | MinIO archive bucket から `mc mirror` で復元 |

## scripts/ の責務

シナリオ手順を 1 コマンド化したシェルスクリプト。手順誤り防止と RTO 短縮が目的。

- **restore-pg-from-barman.sh**: `pg-restore.md` の手順を CNPG `Cluster.spec.bootstrap.recovery` で自動化。
- **restore-minio-from-archive.sh**: archive bucket → 新 Tenant への並列 mirror。
- **rebuild-cluster-from-scratch.sh**: `RB-DR-001` の Phase 1〜3 を逐次実行（Cloud Provider CLI + ArgoCD）。

各スクリプトは `--dry-run` をサポートし、必須引数を欠いた場合は usage を表示する（[`../scripts/rollback.sh`](../scripts/rollback.sh) と同規約）。

## drills/ の責務

DR 演習の実施記録を `DR-drill-YYYY-Qn.md` として残す。記載項目:

- 日時 / 参加者 / シナリオ
- 想定 RTO/RPO と実測値
- 発見された問題と対応 PR リンク
- 次回までの改善アクション

カデンス:

| 種別 | 頻度 | 想定参加者 |
|------|------|------|
| Table-top exercise | 四半期に 1 回 | 起案者 + 協力者 + EM |
| 実機演習（staging） | 半期に 1 回 | 起案者 + 協力者 + 採用組織 SRE |
| 本番 DR 発動演習 | 年 1 回 | 全関係者（Product Council 召集） |

## リリース時点までの整備状況

リリース時点では `RB-DR-001-cluster-rebuild.md` のみが整備済。残る 3 シナリオと 3 スクリプト、および初回 drill 記録は順次整備。

リリース時点必須:
- `scenarios/RB-DR-001-cluster-rebuild.md` ✅
- `scenarios/pg-restore.md` 〜 整備中
- `scenarios/kafka-topic-restore.md` 〜 整備中
- `scenarios/minio-tenant-restore.md` 〜 整備中
- `scripts/rebuild-cluster-from-scratch.sh` 〜 整備中（手動手順を自動化）
- `drills/DR-drill-2026-Q2.md` 〜 初回 table-top 実施後に作成

## 関連

- 関連 NFR: [NFR-A-REC-001（RTO 4h）/ NFR-A-CONT-001](../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md)
- 関連 ADR: [ADR-DATA-001（CNPG）](../../docs/02_構想設計/adr/ADR-DATA-001-cnpg.md), [ADR-DATA-003（MinIO）](../../docs/02_構想設計/adr/ADR-DATA-003-minio.md)
- 関連 Runbook: [`scenarios/RB-DR-001-cluster-rebuild.md`](scenarios/RB-DR-001-cluster-rebuild.md), [`../runbooks/incidents/RB-DB-002-postgres-primary-failover.md`](../runbooks/incidents/RB-DB-002-postgres-primary-failover.md)
