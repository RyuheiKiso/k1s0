# ADR-0079: 外部技術監査 2026-04-03 v2 対応記録

## ステータス

承認済み

## コンテキスト

2026-04-03 に外部プリンシパル・エンジニアによる第三者技術監査を実施した。
`docker compose up` での実際の起動確認（動作確認）を含む静的・動的解析の結果、
**運用環境でのデプロイを実質的に不可能にする CRITICAL 問題を含む 25 件**が検出された。

**発見件数**: CRITICAL×4, HIGH×6, MEDIUM×9, LOW×6（合計25件）
**実対応**: 22件（対応不要確認 3件を除く）

**総合評価**（監査報告書より）:
| 領域 | 評価 | 主な根拠 |
|------|------|---------|
| セキュリティ設計 | A- | AES-256-GCM、JWT/JWKS、RLS、Bearer case-insensitive 実装良好 |
| アーキテクチャ整合性 | A- | DDD 統一構造、Proto バージョニング、ADR 管理良好 |
| コード品質 | B | 600+ unwrap/expect、ヘルスチェック応答形式の不統一 |
| インフラ設定 | B+ | Network Policy 全 NS 実装、Distroless、CAP_DROP、ダイジェスト固定 |
| **運用・デプロイ信頼性** | **D** | **Kafka トピック大量欠落・DB migration バグ・並列ビルド OOM** |
| CI/CD | B+ | 最小権限、SHA ピン留め、多層脆弱性スキャン |
| ドキュメント整合性 | B+ | ADR 78 件・設計書整合性概ね良好 |

## 決定

全25件を精査し、22件の実対応を実施した。以下が主要な技術的決定を要した内容。

### CRITICAL 対応（4件）

#### CRIT-001: kafka-init 並列度制限と exit code 正常化
**ファイル**: `infra/messaging/kafka/create-topics.sh`

- `create_topic()` ヘルパー関数を導入し、`MAX_PARALLEL=5` でセマフォ制御
- 57 JVM の全並列実行は `mem_limit=1.5g` 環境で OOM を引き起こす根本原因を修正
- `ACTUAL_COUNT < EXPECTED_TOPIC_COUNT` 時に `exit 1` を返すよう修正
- `WARN_COUNT > WARN_THRESHOLD(5)` 時にも `exit 1` を返すよう修正
- これにより `docker-compose restart: "on-failure:3"` が正しくリトライを発動する

#### CRIT-002/003: migration SET LOCAL search_path 移行
**ファイル**: workflow-db/008, featureflag-db/005 の up/down 4ファイル

- `SET search_path TO <schema>` → `SET LOCAL search_path TO <schema>, public;`
- `SET LOCAL` はトランザクションスコープのみに影響し、セッション汚染を防止
- `public` を含めることで sqlx の `_sqlx_migrations` テーブルへのアクセスを保証
- 対象 migration はフレッシュ環境で失敗していたため checksum 問題なし

#### CRIT-004: 並列ビルド OOM ドキュメント整備
- README.md に全サービスビルド時の要件（16GB以上推奨）を明記
- `just docker-build-safe`（`--parallel 2`）の使用を推奨
- WSL2 `.wslconfig` 設定例を追記
- `COMPOSE_PARALLEL_LIMIT=4` が `build` に無効な旨を明記

### HIGH 対応（6件）

#### HIGH-001: unwrap/expect 段階的排除
**ファイル**: `.github/workflows/_rust-service-ci.yaml`

- Clippy に `-W clippy::unwrap_used -W clippy::expect_used` を追加（`continue-on-error: true`）
- 初回は warn のみ（CI 非ブロッキング）で件数を可視化する段階的アプローチを採用
- `-D warnings` に昇格するタイミング: 四半期ごとに件数を確認し、0 件になった時点
- 600+ 件を一括 deny にすると全 CI が停止するため段階的アプローチが必要

#### HIGH-002: k1s0-admin RBAC read-only 化
**ファイル**: `infra/kubernetes/rbac/cluster-roles.yaml`

- `roles`/`rolebindings` の verbs を `["get", "list", "watch"]` のみに制限
- 理由: Namespace RoleBinding で ClusterRole を参照することによる権限昇格経路を遮断
- roles/rolebindings の作成・変更は k1s0-security-admin のみが担当

#### HIGH-003: Kong SecurityContext 追加
**ファイル**: `infra/helm/services/system/kong/values.yaml`

- `podSecurityContext`: `runAsNonRoot: true`, `runAsUser: 1000`, `seccompProfile: RuntimeDefault`
- `containerSecurityContext`: `allowPrivilegeEscalation: false`, `capabilities.drop: [ALL]`, `readOnlyRootFilesystem: true`
- `KONG_PREFIX=/tmp/kong` を env に追加し、kong の実行時ファイルを emptyDir に書き込む設計に変更
- `extraVolumes`/`extraVolumeMounts` で `/tmp` を emptyDir マウントし、KONG_PREFIX=/tmp/kong を実現
- 報告書推奨の `readOnlyRootFilesystem: true` を完全実装（暫定処置なし）

#### HIGH-004: Rust CI カバレッジ閾値 65% 導入
**ファイル**: `.github/workflows/_rust-service-ci.yaml`

- tarpaulin に `--fail-under 65` を追加
- 65% は現行の最低カバレッジに合わせた控えめな値
- `_test.yaml` の 75% に段階的に引き上げる

#### HIGH-005: 脆弱性無視リスト統一
**ファイル**: `scripts/security/cargo-audit.sh`

- `RUSTSEC-2023-0071` の `--ignore` エントリを削除（rsa 0.9.10 で解消済み、deny.toml から削除済み）
- コメントで `deny.toml` との同期義務を明記

#### HIGH-006: Kong Admin API TLS 証明書設定方針文書化
**ファイル**: `infra/helm/services/system/kong/values.yaml`

- cert-manager / 既存 Secret / 自己署名の 3 方法をコメントで文書化

### MEDIUM 対応（6件）

- **MED-001**: featureflag healthz に `(StatusCode::OK, Json(...))` タプルを明示
- **MED-005**: topics.yaml の 2 トピックのパーティション数を 6 に統一（create-topics.sh との整合）
- **MED-006**: PrometheusRule 9ファイルに SLO根拠コメントを追加（SLO設計.md 参照）
- **MED-007**: observability egress から `part-of: k1s0` の過剰 podSelector を削除し、個別コンポーネントのみに絞り込み
- **MED-008**: security-gate の needs に image-scan を追加（skipped 許容、failure のみブロック）
- **MED-009**: tests/e2e/README.md に task-crud.spec.ts 追記と CI スケジュール情報追記

### LOW 対応（4件）

- **LOW-002**: e2e.yaml の Keycloak ヘルスチェックを `/health/ready` → `/health` に統一
- **LOW-003**: servicemonitor.yaml に `process_*`/`promhttp_*` の drop ルール追加
- **LOW-004**: deny.toml の RUSTSEC-2025-0111 に解消目標日 `2026-06-30` を追記
- **LOW-005**: security.yaml config-validation ジョブに example.com 検出ステップ追加（INFO レベル）

### 対応不要確認（3件）

- **MED-003**: CLI 確認プロンプトは既に ConfirmResult match パターンで統一済み
- **MED-004**: Tauri CSP `unsafe-inline` は TauriGUI設計.md に既に文書化済み
- **LOW-001**: Vault ServiceAccount 命名は既に `auth-rust` で統一済み

## 理由

### kafka-init セマフォ vs バッチ実行

**選択**: セマフォパターン（`MAX_PARALLEL=5`）

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全並列（現状） | 57 JVM 同時起動 | OOM で大多数が失敗 |
| 順次実行 | 1 プロセスずつ | 起動時間が大幅増（57 × 約5秒 = 約5分） |
| バッチ実行 | 5件ずつ待機 | セマフォと同等だが実装が複雑 |
| **セマフォ（採用）** | MAX_PARALLEL=5 で制限 | メモリ効率と速度のバランスが最良 |

また `exit code` の正常化により `restart: "on-failure:3"` が機能するため、
OOM 発生時でも自動リトライで完全なトピック作成が保証される。

### SET LOCAL vs スキーマ修飾 DDL 維持

`SET search_path` は削除せず `SET LOCAL` に変更する方針を採用。
理由: DDL が完全修飾名を使用しているため `SET search_path` は機能的に冗長だが、
コードの可読性（どのスキーマを対象にしているかが明示的）を維持するため残す。

### RBAC unwrap 段階的排除

一括 deny にすると 600+ 件が全て CI エラーになり開発が停止するため、
4 フェーズの段階的移行を採用:
1. **現在**: `continue-on-error: true` で件数を可視化
2. **Phase 2** (1ヶ月後): 新規コードへの deny 適用
3. **Phase 3** (四半期後): 件数削減後に warn → deny 昇格
4. **Phase 4** (6ヶ月後): 全件排除完了

## 影響

**ポジティブな影響**:
- フレッシュ環境での `docker compose up` が確実に成功するようになる
- `just migrate-all` がフレッシュ DB で失敗しなくなる
- k1s0-admin の RBAC 権限エスカレーション経路が遮断される
- Rust CI でのカバレッジ基準が明確になる
- セキュリティゲートが image-scan の失敗を検知できるようになる

**ネガティブな影響・トレードオフ**:
- kafka-init の起動時間が増加（最大で MAX_PARALLEL=5 の場合、57/5 × 約2秒 = 約23秒）
- Kong の KONG_PREFIX を /tmp/kong に変更したため、kong ログが /tmp/kong/logs/ に出力される（stdout/stderr に加えて）
- Rust CI カバレッジ 65% 閾値により、カバレッジが低いサービスで CI が失敗する可能性

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| kafka-init 新マイグレーション | 新しい migration 009 で `RESET search_path` | SET LOCAL の方が根本原因を修正できる |
| RBAC 全権限維持 | 運用の利便性を優先 | セキュリティリスクが高い |
| Clippy 全件 deny | 即座に全 unwrap を排除 | 600+ 件で全 CI が停止する |
| カバレッジ閾値 75% | _test.yaml と統一 | 現行が 65% 未満のサービスで即時 CI 失敗 |
| Kong readOnlyRootFilesystem: false 維持 | /tmp 要件を理由に暫定処置 | セキュリティ要件への妥協となるため不採用。KONG_PREFIX 変更で解決 |

## 参考

- [報告書.md](../../../../報告書.md) - 外部技術監査報告書（2026-04-03）
- [ADR-0078: 外部技術監査 2026-04-03 対応記録](0078-audit-response-2026-04-03.md)
- [ADR-0038: 最小権限原則](0038-kubernetes-rbac-least-privilege.md)
- [SLO設計.md](../observability/SLO設計.md)
- [CI-CD設計.md](../../infrastructure/cicd/CI-CD設計.md)

## 実装ステータス（2026-04-04 更新）

本 ADR 記載の対応は全件実装完了。2026-04-04 外部監査報告書対応により確認済み。

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（外部技術監査 v2 全25件対応） | @k1s0 team |
| 2026-04-03 | HIGH-003 完全実装（readOnlyRootFilesystem: true + KONG_PREFIX + emptyDir）; fresh-deploy-smoke.yaml 追加（報告書7.3対応） | @k1s0 team |
| 2026-04-03 | HIGH-005 完全実装（advisory-ignore-list.txt 一元管理 + CI整合性チェック）; LOW-003 全9サービスに適用 | @k1s0 team |
| 2026-04-04 | 実装ステータス追記（全件完了確認） | kiso ryuhei |
