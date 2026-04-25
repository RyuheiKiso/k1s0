# ADR-STOR-001: 分散ブロックストレージに Longhorn を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / SRE

## コンテキスト

k1s0 はオンプレミス Kubernetes 上で稼働し、PostgreSQL（CloudNativePG）、Kafka（Strimzi）、MinIO、Temporal、Valkey など多数のステートフルワークロードが PVC を要求する。オンプレ環境でこれらを支える分散ブロックストレージの選定が必要。

制約条件は以下の通り。

- オンプレミス完結（SaaS・商用ストレージアプライアンスはコスト制約上不可）
- Kubernetes ネイティブ（CSI 対応）
- スナップショット / バックアップ / ReadWriteOnce 標準対応
- 3 台以上のレプリカで HA
- 運用工数が過大にならない
- AGPL / 商用制約の少ない OSS

候補は Longhorn、Rook/Ceph、OpenEBS、Portworx、Piraeus Datastore（LINSTOR）。

## 決定

**分散ブロックストレージは Longhorn（CNCF Incubating、Apache 2.0）を採用する。**

- Longhorn 1.6+
- Node 間でボリュームを 3 レプリカ、自動フェイルオーバー
- PVC の動的プロビジョニング、StorageClass 標準化
- バックアップ先として MinIO（ADR-DATA-003）を使用、定期スナップショット
- 高性能ワークロード（PostgreSQL primary、Kafka）は Longhorn の "data locality" 設定で同一 Node 優先
- リリース時点では Node ローカル NVMe を Longhorn が管理、採用側のクラスタ規模拡大時に分離 Storage Cluster 検討

## 検討した選択肢

### 選択肢 A: Longhorn（採用）

- 概要: Rancher Labs 発、CNCF Incubating
- メリット:
  - Kubernetes ネイティブ、Helm Chart 成熟
  - GUI（Longhorn UI）で可視化・運用容易
  - スナップショット / バックアップ（S3 / NFS）標準
  - 軽量、運用工数が Rook/Ceph より小
  - Node ローカル NVMe を Pool 化可能
- デメリット:
  - Ceph ほどの機能豊富さはなし（オブジェクトストレージは別途 MinIO）
  - 大規模運用実績は Ceph よりは浅い
  - レプリカ 3 で容量効率は 33%

### 選択肢 B: Rook/Ceph

- 概要: CNCF Graduated、高機能分散ストレージ
- メリット:
  - ブロック / オブジェクト / ファイル すべて対応
  - 大規模運用実績（CERN、Red Hat 等）
  - Erasure Coding で容量効率向上
- デメリット:
  - 運用コスト膨大（MON / OSD / MDS / MGR 構成、Ceph 専門知識必須）
  - 小規模クラスタではオーバースペック
  - アップグレードが複雑

### 選択肢 C: OpenEBS

- 概要: MayaData 発、CNCF Sandbox
- メリット: Kubernetes ネイティブ
- デメリット:
  - Longhorn に比べて GUI / Backup 機能が弱い
  - コミュニティ活発度で Longhorn 優位

### 選択肢 D: Portworx

- 概要: Pure Storage 傘下、商用製品
- メリット: 機能最大、商用サポート
- デメリット:
  - 商用ライセンス（年間数千万円）
  - 採用側のコスト制約で不可

### 選択肢 E: Piraeus Datastore (LINSTOR/DRBD)

- 概要: LINBIT 発、DRBD ベース
- メリット: 高性能、低レイテンシ
- デメリット:
  - GUI / スナップショット運用で Longhorn に劣る
  - コミュニティ規模が小さい

## 帰結

### ポジティブな帰結

- Kubernetes ネイティブで運用者の学習コスト最小化
- GUI でボリューム状態・バックアップが可視化
- MinIO へ定期バックアップ、DR 対策実現
- Apache 2.0 で商用利用・改変自由

### ネガティブな帰結

- レプリカ 3 で容量効率 33%、NVMe コストへの影響
- 超大規模（数 PB 級）では Ceph 検討、採用側のクラスタ規模拡大時の見直し候補
- Longhorn 自体の HA 設計（CSI Driver、Manager の冗長化）
- Node 障害時の rebuild 時間（数時間）、SLO 影響範囲の定義

## 実装タスク

- Longhorn Helm Chart バージョン固定、Argo CD 管理
- StorageClass 設計（replica=3、data-locality=best-effort）
- MinIO バックアップ先設定、スケジュール（日次 / 週次）
- Longhorn UI SSO 統合（Keycloak）
- FMEA: ストレージ障害シナリオ（Node 障害、Disk 障害、rebuild 中の 2 重障害）
- 容量監視、自動 PV 拡張の閾値設定
- Longhorn バージョンアップ Runbook

## 参考文献

- Longhorn 公式: longhorn.io
- CNCF Longhorn Project
- Rancher Longhorn Documentation
- Kubernetes CSI 仕様
