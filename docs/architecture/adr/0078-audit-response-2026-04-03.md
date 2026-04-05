# ADR-0078: 外部技術監査 2026-04-03 対応記録

## ステータス

承認済み

## コンテキスト

2026-04-03 に専門別エージェントチーム7体による並列分散監査を実施した。
過去350件超の監査対応（2026-03-22 〜 2026-04-02）の成果を踏まえた深掘り分析を実施。

**発見件数**: CRITICAL×9, HIGH×22, MEDIUM×26, LOW×19（合計76件）
**実対応**: 67件（偽陽性・対処済み確認 9件を除く）

**総合評価**（監査報告書より）:
| 領域 | 評価 |
|------|------|
| Rust バックエンド | B |
| Go/CLI/フロントエンド | B+ |
| インフラ（docker-compose） | B- |
| Kubernetes/Helm | B- |
| ドキュメント整合性 | B- |
| CLI ツール | B |
| Docker Compose 動作確認 | B- |

## 決定

全76件を精査し、67件の実対応を実施した。以下が主要な対応内容。

### CRITICAL 対応（9件）

| ID | 内容 | 対応 |
|----|------|------|
| RUST-CRIT-001 | workflow テナント境界欠如（全テナントデータ横断アクセス可能） | DB 3テーブルに tenant_id + RLS 追加、ハンドラ/ユースケース/リポジトリ全層修正 |
| DOCKER-CRIT-001 | graphql-gateway 永続的 503（RUNTIME-001 第3回再発） | service_catalog チェック先を /healthz に変更、--no-cache 運用周知 |
| CRIT-CLI-001 | generate_navigation.rs 確認プロンプト空ブロック | `!= Yes` ガードパターンに統一 |
| INFRA-CRIT-001 | Kong healthz JWT バイパス設定ミス（グローバル JWT 上書き不可） | kong.dev.yaml から 4 ヘルスサービス定義（auth/config/saga/dlq-manager-health）を完全削除。K8s probe・Prometheus は ClusterIP 直接アクセス設計に移行。kong.yaml にも設計方針コメントを明記。 |
| K8S-CRIT-001 | 全32サービス seccompProfile 欠落（PSS restricted 違反） | 全33 values.yaml に seccompProfile: RuntimeDefault 追加 |
| K8S-CRIT-002 | NetworkPolicy クラスター・リポジトリ間ドリフト | ADR-0076 作成（GitOps 導入計画） |
| DOCS-CRIT-001 | board API パス構造が設計書と完全不一致 | 設計書を実装（フラット構造）に合わせて修正 |
| DOCS-CRIT-002 | activity approve/reject の HTTP メソッドと権限不整合 | 設計書を実装（PUT + activities/admin）に修正 |
| DOCS-CRIT-003 | project-master パスパラメータ型不一致（code vs UUID） | 設計書を実装（UUID ベース）に修正 |

### HIGH 対応（22件）

| ID | 内容 | 対応 |
|----|------|------|
| RUST-HIGH-001 | gRPC Bearer case-insensitive 未対応（9サービス10箇所） | eq_ignore_ascii_case パターンに統一 |
| RUST-HIGH-002 | featureflag DB に RLS なし | 005_add_rls.up.sql 追加 |
| RUST-HIGH-003 | rule-engine regex キャッシュなし（ReDoS リスク） | LruCache + パターン長上限 1024 文字 |
| FE-HIGH-001 | Go app-updater URL パラメータ未エスケープ | url.PathEscape/QueryEscape に修正 |
| FE-HIGH-002 | Go io.ReadAll 上限なし（5箇所） | io.LimitReader(resp.Body, 1<<20) に修正 |
| HIGH-CLI-001 | Go プロジェクトのバージョン検出未実装 | K1S0_SERVICE_VERSION → git describe フォールバック |
| INFRA-HIGH-001 | backup CronJob seccompProfile・readOnlyRootFilesystem 欠落 | postgres/vault/harbor backup に追加 |
| INFRA-HIGH-002 | business/service Vault ロール共有 | ADR-0077 + Terraform auth.tf の個別ロール化 |
| INFRA-HIGH-003 | docker-compose.dev.yaml パスワード平文 | .env.dev 変数参照に変更 |
| INFRA-HIGH-004 | promtail 2.9.0（非推奨） | 3.3.2 に更新 |
| K8S-HIGH-001~004 | NetworkPolicy 未適用・NotReady ノード | kustomization.yaml 確認（対応済み）、ノード運用手順追記 |
| DOCS-HIGH-001 + DOCKER-HIGH-001 | ADR-0068 readyz 形式未実装（24サービス） | health.rs 21ファイル + app_handler.rs + ratelimit_handler.rs + table_handler.rs + dlq_handler.rs + saga_handler.rs + bff-proxy health_handler.go を healthy/unhealthy + timestamp に統一（計27ファイル）。regions/ 配下で "ready"/"not_ready" ゼロ件確認済み。 |
| DOCS-HIGH-002~004 | 設計書不整合（config/board/task） | 設計書を実装に合わせて修正 |
| DOCKER-HIGH-003 | Keycloak 同期失敗（auth サービスアカウント未設定） | realm-k1s0.json に auth-rust-admin クライアント追加 |
| DOCKER-HIGH-004 | migrate-all business/service マイグレーション失敗 | justfile に search_path オプション追加 |

### MEDIUM 対応（26件中19件対応、7件対処不要）

- RUST-MED-002: workflow page 上限（10,000）追加
- RUST-MED-003: regex workspace.dependencies 統一
- RUST-MED-004: featureflag unwrap_or_default に tracing::warn 追加
- FE-MED-003: React appId に encodeURIComponent
- FE-MED-004: CSRF TTL フォールバックに削除 TODO コメント追加
- MED-CLI-003: 非インタラクティブエラーメッセージ定数化
- INFRA-MED-001: HPA メモリ閾値 80→70%（コメントとの不整合修正）
- INFRA-MED-002: observability egress スコープ説明コメント追加
- INFRA-MED-006: Vault パッチバージョン固定（1.19→1.19.5）
- K8S-MED-001: grafana Chart.yaml 作成
- K8S-MED-003: audit-policy 適用手順をドキュメント化
- K8S-MED-004: scheduler/controller-manager 再起動調査手順をドキュメント化
- DOCKER-MED-002: kafka-init メモリ増加（1g→1.5g）
- DOCKER-MED-003: rule-engine Kafka 設定追加

対処不要確認: RUST-MED-001, FE-MED-001, FE-MED-002, MED-CLI-001, MED-CLI-002, INFRA-MED-003, INFRA-MED-005

### LOW 対応（19件中15件対応、2件偽陽性、2件評価のみ）

- RUST-LOW-001: workflow initiator_id を JWT Claims から取得
- RUST-LOW-004: Internal エラーメッセージ固定文字列化
- FE-LOW-002: scaffold Dockerfile rust:1.75 → 1.85
- INFRA-LOW-002: Kafka CLUSTER_ID コメント修正
- INFRA-LOW-003: schema-registry ヘルスチェック curl → wget
- INFRA-LOW-005: bff-proxy Kong ルーティング設計をドキュメント化
- LOW-CLI-002: CLI エラーメッセージ具体化

偽陽性: K8S-LOW-001（allowVolumeExpansion 設定済み）, K8S-LOW-002（-f 混在なし）

## 理由

すべての修正は以下の方針に従って実施した:
1. セキュリティ問題（テナント分離・RLS・認証）を最優先
2. 実装が正しい場合は設計書を実装に合わせて修正（逆方向は採用しない）
3. 偽陽性は根拠を示して対応不要と判定し、不必要な変更を回避

## 影響

**ポジティブな影響**:
- workflow のマルチテナント対応が完了し、テナント間データ漏洩リスクが解消
- 全 Kubernetes デプロイが PSS restricted 環境で動作可能になる
- ADR-0068 の readyz 形式が全サービスで統一され監視の信頼性が向上
- Vault ロールが business/service ティアまで個別化される（ADR-0077）

**ネガティブな影響・トレードオフ**:
- workflow サービスの DB マイグレーション（008）は既存データの `tenant_id` を 'system' に設定するため、マルチテナント環境での本番適用は慎重に行う必要がある
- Keycloak realm-k1s0.json の変更はランタイム再インポートが必要

## 代替案

| 案 | 採用しなかった理由 |
|----|-----------------|
| workflow テナント分離を段階実装 | CRITICAL セキュリティ問題のため即時対応が必要 |
| 設計書ではなく実装を修正（DOCS-CRIT） | 実装の方が正しい設計判断をしている（フラット構造等） |

## 参考

- [ADR-0068: readyz レスポンス形式標準化](0068-readyz-response-format.md)
- [ADR-0045: Vault 個別ロール（system）](0045-vault-per-service-role-isolation.md)
- [ADR-0072: RLS tenant_id 型キャスト標準化](0072-rls-policy-type-cast-standardization.md)
- [ADR-0076: GitOps 導入による NetworkPolicy ドリフト防止](0076-gitops-networkpolicy-drift-prevention.md)
- [ADR-0077: Vault business/service ティアロール個別化](0077-vault-business-service-role-isolation.md)
- 外部技術監査報告書: `報告書.md`（2026-04-03）

## 実装ステータス（2026-04-04 更新）

本 ADR 記載の対応は全件実装完了。2026-04-04 外部監査報告書対応により確認済み。

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（外部技術監査 2026-04-03 全件対応記録） | kiso ryuhei |
| 2026-04-04 | 実装ステータス追記（全件完了確認） | kiso ryuhei |
