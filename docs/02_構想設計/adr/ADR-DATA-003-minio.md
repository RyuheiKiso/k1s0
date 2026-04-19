# ADR-DATA-003: オブジェクトストレージに MinIO を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / データ基盤チーム / 運用チーム

## コンテキスト

tier1 の Binding API でのオブジェクト入出力、監査ログの長期保管（NFR-H-AUD-001 で WORM 要件）、PostgreSQL バックアップの集約先（ADR-DATA-001 の barman バックエンド）、Velero での Kubernetes 全体バックアップ先、Temporal の成果物保管など、k1s0 全体で S3 互換オブジェクトストレージが要求される。

制約条件は以下の通り。

- オンプレミス完結（クラウドの S3/GCS 等は選択肢外）
- S3 API 互換必須（多くの OSS が S3 API 前提）
- データ耐久性 99.9% 以上（11 ナインは過剰、オンプレとしては現実的値）
- 暗号化（転送時 TLS、保管時 SSE）
- WORM（Write Once Read Many）のオブジェクトロック機能必須

オンプレミス S3 互換 OSS の主要候補は MinIO、Ceph RGW、SeaweedFS、OpenIO（DEAD）、Scality（商用）。

## 決定

**オブジェクトストレージは MinIO（AGPL-3.0）を独立クラスタで運用する。**

- MinIO は AGPL-3.0 のため、ADR-0003（AGPL 分離アーキテクチャ）に従い、本体コードとプロセス・ネットワーク境界で明確に分離
- 通信は S3 API（HTTPS）のみ、独自クライアント・内部パッケージは使わない
- Distributed Mode（4 ノード以上で erasure coding）を標準構成、k8s StatefulSet で管理
- Bucket ごとに Lifecycle Policy（世代管理・自動削除）を設定
- Object Lock モードで WORM 化、監査ログのコンプライアンス要件（NFR-H-AUD-001）を満たす
- SSE-S3（サーバサイド暗号化）を有効化、鍵は OpenBao（ADR-SEC-002）経由

AGPL 回避のため、MinIO のコードを直接組込んだり、MinIO のフォーク/拡張を書くことは禁止。純粋に「S3 API で話す」利用に限定する。

## 検討した選択肢

### 選択肢 A: MinIO（採用）

- 概要: Go 製 S3 互換オブジェクトストレージ、業界デファクト
- メリット:
  - S3 API 互換性が高く、既存ツール（aws-cli、boto3、velero）がそのまま使える
  - K8s Operator あり、StatefulSet 運用で Argo CD 親和性高
  - Object Lock、Lifecycle、Versioning、Replication すべて揃う
  - Prometheus メトリクス、Grafana ダッシュボード公式提供
- デメリット:
  - AGPL-3.0 ライセンス（ADR-0003 で対応）
  - Community Edition と Enterprise の機能差（Key Management 等）が年次で広がる傾向

### 選択肢 B: Ceph RGW（Rook Operator）

- 概要: Ceph のオブジェクトストレージインタフェース、CNCF Graduated (Rook)
- メリット: Ceph の巨大エコシステム、Block/File/Object すべて統合可能
- デメリット:
  - 運用の複雑度が高い（mon/mgr/osd/rgw のコンポーネント多数）
  - 2 名チームでの運用工数が過大
  - S3 API 互換性は MinIO に劣る部分あり

### 選択肢 C: SeaweedFS

- 概要: Go 製分散ファイルシステム、S3 API 対応
- メリット: 軽量、シンプル、学習曲線緩やか
- デメリット:
  - 採用実績が MinIO/Ceph より少ない
  - エンタープライズ機能（Object Lock 等）の成熟度が低い

### 選択肢 D: Scality RING / NetApp StorageGRID（商用）

- 概要: 商用 S3 互換ソリューション
- メリット: ベンダーサポート、SLA 付き
- デメリット:
  - 商用ライセンス費用がコスト削減目標（BC-COST-003）と逆行
  - ベンダーロック（OR-EXIT-004）が発生

## 帰結

### ポジティブな帰結

- S3 API 互換で既存ツール資産をそのまま活用（Velero、barman、aws-cli 等）
- Object Lock で監査コンプライアンス要件を技術的に保証
- Kubernetes StatefulSet で宣言的運用、Argo CD との統合
- Lifecycle Policy で古いデータの自動削除、ストレージ費用最適化

### ネガティブな帰結

- AGPL-3.0 への対応として ADR-0003 準拠の運用が必要（監査で繰り返し説明を要する）
- Enterprise 版との機能差が年次で広がる可能性、商用切替時の再評価が必要
- Erasure Coding の設計（データ N + パリティ M）は初期に決定、後から変更困難

## 実装タスク

- MinIO Operator の Helm Chart バージョン固定、Argo CD で管理
- Bucket ごとの Lifecycle / Object Lock / Versioning テンプレートを Backstage Software Template 化
- SSE-S3 の鍵ローテーション手順を Runbook 化
- Velero バックアップの保管先として MinIO を設定、復元訓練を四半期実施
- AGPL 分離の証跡（分離図、通信プロトコル記録）を半期監査に提出

## 参考文献

- MinIO 公式: min.io
- AGPL-3.0 本文: gnu.org/licenses/agpl-3.0
- Amazon S3 API 仕様: AWS 公式
- Velero 公式: velero.io
