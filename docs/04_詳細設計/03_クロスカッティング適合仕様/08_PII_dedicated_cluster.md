---
id: detail.cross_pii.pii_dedicated_cluster
axis: cross_pii
phase: cross_cutting
kind: cross_cut_spec
status: draft
depends_on:
  - arch.data.preservation_policy
  - arch.data.encryption_policy
  - arch.security.pii_protection_policy
  - detail.data.preservation_conformance
  - detail.data.data_enforcement
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes:
    - v1_program_correctness_proof
related_axes:
  - data
  - security
  - infra
---

# PII 専用 PostgreSQL クラスタ（v1）

## 位置づけ
- [data 保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md) / [data 暗号化方針](../../03_概要設計/06_data設計方針/04_暗号化方針.md) / [security PII 保護方針](../../03_概要設計/07_security設計方針/06_PII保護方針.md) と双方向 lock
- PII の物理的な隔離（backup 経路分離、KEK 分離、retention 分離、crypto-erase 分離）を、業務クラスタとは別 PostgreSQL クラスタとして達成する

## 不可避性
- PostgreSQL の point-in-time recovery (PITR) は WAL ベース。schema / table 単位での WAL 除外は構造的に不可能
- 同一クラスタ内で「PII schema だけを backup 対象から外す」と crash recovery / streaming replication / logical decoding のいずれかが必ず破綻する
- 「PII テーブルは backup 対象から分離」を物理的に達成するには、WAL chain そのものを別クラスタに分離するしかない
- 本ファイルが定義する PII 専用 PostgreSQL クラスタは、これを唯一現実的な解として選択する

## 配置
- **業務クラスタ**（Kubernetes cluster + CloudNativePG Cluster CR）: 既存 CloudNativePG 構成（PITR / streaming replication / RPO 5 min）。PII column は持たない
- **PII クラスタ**: 独立 Kubernetes cluster 上の独立 CloudNativePG Cluster CR
    - 物理 node は業務クラスタと別 K8s cluster（共有 node も共有 control plane も許さない）
    - 業務 control plane が侵害されても PII cluster の admission / deploy / secret 経路は影響を受けない物理境界

### PII cluster の control plane も独立配置
- Argo CD / Argo Rollouts / Argo Events / Kyverno / cert-manager / OpenBao / External Secrets Operator / Keycloak / 監査 ClickHouse / OTel Collector のいずれも PII cluster 内に独立 instance として配置
- 業務 cluster の同名 instance とは別 deployment、別 secret、別 root CA、別 OIDC realm、別 ClickHouse SoR
- Argo CD multi-cluster federation は採用しない
- OpenBao は PII cluster 用と業務 cluster 用で別の root token / KEK / HSM partition を持つ
- cross-region での KEK 物理冗長は Shamir Secret Sharing (M-of-N) で N region の独立 OpenBao instance + 独立 HSM partition に share を分散保管
- cert-manager の root CA も別

### cross-cluster の唯一の接続経路
- Debezium が業務 cluster の outbox を tail して PII cluster の tier2 service へ「outbox event の一方向流れ込み」のみ
- 逆方向（PII → 業務）の同期は存在しない
- PII KEK は PII cluster 内 HSM partition、PII cluster 内 OpenBao Transit path にのみ存在し、PII cluster の OpenBao は KEK 全体ではなく Shamir share を 1 個のみ保持

## 役割

### 役割 PII-A: 業務 schema → 業務クラスタ、PII column → PII クラスタ への transparent dual-write
- tier2 application は PII 列を含む aggregate を保存する際、業務クラスタには non-PII 列 + pii_ref（不透明 UUID）を atomic 書込し、PII 列は outbox 経由で PII クラスタに非同期書込
- 業務クラスタ側 outbox は PostgreSQL 単一 transaction 内で aggregate UPDATE + outbox INSERT + audit_local INSERT の 3 表 atomic
- PII クラスタ側書込は Debezium で outbox を tail し、eventual consistency

### 役割 PII-B: backup 戦略の分離
- 業務クラスタ: PITR（WAL ベース、RPO 5 min）
- PII クラスタ: logical backup のみ（pg_dump --column-inserts --no-owner 形式、AES-256-GCM 暗号化、PII KEK で wrap）
- PII クラスタの backup は GDPR / 個情法 retention に厳密準拠

### 役割 PII-C: crypto-erase 経路
- PII row 単位 crypto-erase は PII クラスタの当該 row の DEK を全 HSM replica で同期 destroy + offline tape の wrapped DEK を revocation manifest に記載
- litigation hold 中の row は erase queue で delay

### 役割 PII-D: 参照整合性
- 業務クラスタの pii_ref は PII クラスタの primary key を参照する論理 FK
- RDBMS レベルの FK constraint は cross-cluster なため張れない
- eventual consistency を前提に reconciliation job を 1 時間ごと実行し、孤児 pii_ref を検出
- 検出時は SLO event として記録、業務クラスタ側 row は masking 表示で degrade

## dual-write atomicity
- 業務クラスタ側に書く outbox イベントは「PII row 書込指示」のみを含む
- PII クラスタへの書込は Debezium + tier2 service が冪等に処理する（同 outbox event id を PII クラスタの Idempotency-Key として使う）
- outbox 配信失敗時の業務クラスタ row は「pii_pending」状態として PII column が masking 表示される

## cross-cluster referential check の eventual 性
- 業務クラスタの pii_ref → PII クラスタの primary key の referential integrity は、cross-cluster なので RDBMS の FK で担保できない
- reconciliation job（1 時間ごと）が両クラスタの pii_ref / PII row 集合を sync して孤児を検出
- 孤児を SLO event として page、業務 row 側は masking 表示で degrade

## 副作用
- K8s cluster 数が +1（業務クラスタ + PII クラスタ）
- control plane の独立配置による instance 数増加: 運用 instance 数が倍になるが、本企画は運用コスト度外視で隔離を優先（CLAUDE.md 至高路線と整合）
- HSM partition 数が +1（PII KEK 用）
- cross-cluster の Debezium outbox 一方向流れ込みは独立 K8s cluster 間で動かす必要があり、kafka MirrorMaker2 / Connect のいずれかで cluster 境界を跨ぐ wire を構成
- dual-write の余分な network hop で write latency が +5〜20 ms（許容範囲、SLO に取込済）
- cross-cluster referential check の eventual 性を業務 UI 側が masking 表示で吸収する責務

## 整合
- 整合 1: tier2/26 の「PII テーブルは backup 対象から分離する」を本クラスタ分離で達成
- 整合 2: tier2/27 の「PII は業務データと同じ RPO 5 分」は業務クラスタ側 pii_ref のみに適用
- 整合 3: [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md) の v1_global_replicated に「PII クラスタは別 retention / 別 backup」を追記
- 整合 4: [security PII 保護方針](../../03_概要設計/07_security設計方針/06_PII保護方針.md) の crypto-erase scope は本クラスタの DEK destroy 経路で物理達成
- 整合 5: tier2/32 の pii_segregated class は本クラスタへの dual-write 規約として再解釈

## 採用しない設計
- 同一クラスタ内 schema 除外で PII 分離: 物理不可能、本仕様で撤回
- PII cluster と業務 cluster で control plane 共有: 禁止
- Argo CD multi-cluster federation: 採用しない
- 業務 cluster の root CA から派生した cert で PII cluster の workload を認証: 禁止
- KEK の単一 region 復元: 禁止（Shamir M-of-N で N region 分散）

## 関連参照
- [data 設計方針 index](../../03_概要設計/06_data設計方針/README.md)
- [データ保全適合仕様](../01_適合仕様/14_データ保全適合仕様.md)
- [data 暗号化方針](../../03_概要設計/06_data設計方針/04_暗号化方針.md)
- [security PII 保護方針](../../03_概要設計/07_security設計方針/06_PII保護方針.md)
- [tier1 鍵管理適合仕様](../01_適合仕様/05_鍵管理適合仕様.md)
