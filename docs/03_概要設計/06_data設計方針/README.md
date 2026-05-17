---
id: arch.data.data_index
axis: data
phase: architecture
kind: index
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# data 設計方針 index

## 一文方針
- 本フォルダは data 軸（5 階層中のデータ層、保全 / lifecycle / 暗号化）の概要設計を集約する。data は tier1 / tier2 / tier3 が永続化を要求する全業務データ・観測データ・メッセージを「物理層に堕とした時にも保全 / 復元 / 失効が一意に定まる」状態で運用する層。

## 至高路線における立ち位置
- 「永続化された後に巻き戻せない事象」（書き込み確定・logical replication の divergence・物理 disk 暗号化失敗・archive 期限切れ）の最終 safety net 提供層
- 13 軸の各 spec の defense-in-depth 層 E（物理）のうち、永続化 / 復元 / 暗号化に関する物理基盤は data 層が所有する instance で実 enforcement される
- data は 13 軸の 1 軸（preservation = data01）の保有層であると同時に、全 13 軸の defense-in-depth 層 E のうち永続化に関わる物理基盤を提供する基盤層という二重の役割

## 至高路線における原則
- **単一 OSS 深耕**: 1 機能カテゴリ（RDB / Stream / Cache / Analytics / Schema Registry）に対して採用する OSS は 1 種類に絞る
- **宣言的真**: preservation_class / replication_topology / encryption_layer / restore_drill 状態は GitOps + lock.yaml でのみ表現
- **不可逆性の禁止**: archive_to_offline 以外の経路で物理 delete を発火する経路を持たない
- **段階的 release 禁止**: 1.0.0 時点で 5 preservation_class 全てが restore_drill green
- 「運用が大変だから restore_drill cadence を緩める」判断は採らない

## 配下ドキュメント

| 番号 | ドキュメント | 主題 |
|---|---|---|
| 01 | [データストア構成方針](01_データストア構成方針.md) | CloudNativePG / Strimzi / Valkey / ClickHouse / Apicurio の 5 OSS L1+ 単一深耕 + 4 layer 分離 |
| 02 | [スキーマ運用方針](02_スキーマ運用方針.md) | sqlx-cli + go-migrate + Apicurio CLI、forward-only migration、phantom write、Outbox |
| 03 | [保全方針](03_保全方針.md) | 4 層保全（online / sync / async / cold archive）+ preservation_class 別 retention |
| 04 | [暗号化方針](04_暗号化方針.md) | 3 層暗号化（in-transit / at-rest / application-layer envelope）+ AES-256-GCM L1+ |
| 05 | [レプリケーション方針](05_レプリケーション方針.md) | preservation_class 別 topology + split-brain 防止 + replication lag SLI |
| 06 | [ライフサイクル方針](06_ライフサイクル方針.md) | hot → warm → cold → archive_to_offline → purge の単一経路 + crypto-erase |
| 07 | [復旧訓練方針](07_復旧訓練方針.md) | 4 種 drill（full restore / cross-region failover / crypto-erase / archive_to_offline）+ AND-gate |
| 08 | [マイグレーション方針](08_マイグレーション方針.md) | expand-contract pattern + dual-write/dual-read + forward-only |

## 5 preservation_class

| class | durability | replication_topology | restore_window | drill_cadence_days |
|---|---|---|---|---|
| `v1_local_only` | local | none | 4 h | 180 |
| `v1_zone_replicated` | zone_quorum | sync_within_cluster | 60 sec | 90 |
| `v1_cluster_replicated` | cluster_quorum | async_cross_cluster | 5 min | 60 |
| `v1_cross_region_replicated` | region_quorum（C-priority during partition）| sync_cross_region | 5 min | 30 |
| `v1_global_replicated` | global_quorum | cell_partitioned_multi_region | 0（per-cell continuous）| 14 |

## 上位フェーズへの依存
- [提供スコープ](../../02_要件定義/01_スコープ/01_提供スコープ.md): data 軸の責務範囲
- [非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md): 委譲先
- [可用性災害対策](../../02_要件定義/03_非機能要件/02_可用性災害対策.md): infra + data + ops の cross 要件
- [OSS 採用一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md): data 軸採用 OSS

## 下位フェーズへの委譲
- [データ保全適合仕様](../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md): structural spec、5 preservation_class + 5 enforcement orchestrator
- [data 強制機構](../../04_詳細設計/02_強制機構/05_data強制機構.md): 5 層 defense-in-depth + 25+ Kyverno admission policy
- [data 運用 UI](../../04_詳細設計/04_運用UI開発者体験/02_data運用UI.md)
- [PII 専用クラスタ](../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md): PII の物理隔離

## 横断軸との bind
- [infra 設計方針](../05_infra設計方針/README.md): topology_class と preservation_class の double bind
- [tier1 鍵管理](../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md): KEK / DEK 階層
- [tier2 テナント分離](../../04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md): RLS FORCE
- [security PII 保護方針](../07_security設計方針/06_PII保護方針.md): 5 PII class
- [test 故障注入検証方針](../10_test設計方針/05_故障注入検証方針.md): chaos drill 連携

## 関連参照
- [親フォルダ index](../README.md)
- [規約層 index](../../00_format/README.md)
