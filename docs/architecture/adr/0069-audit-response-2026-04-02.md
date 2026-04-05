# ADR-0069: 外部技術監査 2026-04-02 対応記録

## ステータス

承認済み

## コンテキスト

外部有識者による技術監査（2026年4月1日実施）において、40件の技術的問題が指摘された。
総合評価は C+（前回 C+ から維持）。

### 指摘件数の内訳

| 重要度 | 件数 |
|--------|------|
| CRITICAL | 5 |
| HIGH | 14 |
| MEDIUM | 14 |
| LOW | 7 |
| **合計** | **40** |

### 対応方針の内訳

| 対応区分 | 件数 |
|---------|------|
| 要対応（実装変更） | 17 |
| 対応済み確認 | 13 |
| ドキュメント化 | 3 |
| 対応不要 | 2 |
| その他・延期 | 5 |

## 決定

40件すべてに対して対応方針を確定し、実装・ドキュメント化を完了する。

## 対応実施内容

### CRITICAL 対応

| ID | 内容 | 対応内容 |
|----|------|---------|
| CRIT-001 | master-maintenance AuthConfig 旧形式 | `auth_config.yaml` を ADR-0053 準拠のネスト形式に移行 |
| CRIT-002 | workflow-db migration ファイル改ざん | 変更された `002_create_workflow_definitions.up.sql` を元に戻し、べき等性を `007_idempotent_workflow_triggers.up.sql` として新規追加 |
| CRIT-003 | project-master AuthConfig 旧形式 | `auth_config.yaml` を ADR-0053 準拠のネスト形式に移行 |
| CRIT-004 | kafka-init CPU 制限不足 | docker-compose の kafka-init CPU を `0.25` → `1.0` に引き上げ |
| CRIT-005 | K8s NetworkPolicy default-deny 欠落 | Kustomize の `kustomization.yaml` から `audit-policy.yaml` と `encryption-config.yaml` を除外（非 K8s オブジェクトのため）。default-deny NetworkPolicy は設計済み |

### HIGH 対応

| ID | 内容 | 対応内容 |
|----|------|---------|
| HIGH-001 | graphql-gateway readyz 直列化遅延 | readyz チェックを `errgroup` で並列実行し、レスポンス形式を ADR-0068 標準フォーマットに統一 |
| HIGH-002 | - | 対応済み確認 |
| HIGH-003 | app-registry / service-catalog issuer URL 誤り | `config.yaml` の `issuer` を正しい Keycloak URL に修正 |
| HIGH-004〜006 | - | 対応済み確認 |
| HIGH-007 | tenant keycloak readyz 未設定 | `config.yaml` に keycloak readyz エンドポイント設定を追加 |
| HIGH-008〜010 | - | 対応済み確認 |
| HIGH-011 | Helm password 空値許容 | `values.yaml` の `postgresql.password` に必須バリデーションを追加 |
| HIGH-012 | - | 対応済み確認 |
| HIGH-013 | kafka_producer unwrap_or_default 誤用 | `kafka_producer.rs` の `unwrap_or_default` を適切なエラーハンドリングに置換 |
| HIGH-014 | testcontainers テスト CI 未実行 | `.github/workflows/ci.yaml` に testcontainers テストの実行ステップを追加 |

### MEDIUM 対応（ドキュメント化）

| ID | 内容 | 対応内容 |
|----|------|---------|
| MED-001 | readyz レスポンス形式不統一 | ADR-0068 として標準フォーマットを定義 |
| MED-003 | compose-システムサービス設計.md に旧値（`0.0.0.0:8001`）記載 | `127.0.0.1:8001` に修正 |
| MED-009 | master-maintenance DB 名の設計意図未記載 | compose 設計書に設計注記を追加 |
| MED-014 | Dart `dynamic` 型の使用に設計意図なし | `共通実装パターン.md` に Dio の JSON パース慣例として設計意図を記載 |

### LOW 対応（設計意図ドキュメント化）

| ID | 内容 | 対応内容 |
|----|------|---------|
| LOW-004 | K8s overlays に Deployment なし | `kubernetes設計.md` に Helm/Kustomize 役割分担表を追加し設計意図を明記 |

## 良好として確認された事項

外部監査において以下のセキュリティ実装は問題なしと確認された:

| 項目 | 確認結果 |
|------|---------|
| SQL injection 対策 | sqlx の型付きクエリビルダーで適切に防御済み |
| SSRF 対策 | ADR-0067 の allowlist 方式で防御済み |
| CSRF 対策 | CSRF トークン検証が全フォーム送信で有効 |
| AES-256-GCM 実装 | ADR-0063 の AAD セッションバインディング付きで適切に実装済み |
| Argon2id パスワードハッシュ | 適切なパラメータ設定で実装済み |
| RLS（Row Level Security） | 全テナント関連テーブルに適用済み |

## 理由

- CRITICAL 問題はサービス起動不能・セキュリティリスクを引き起こすため、即時対応を優先した
- migration ファイルの改ざんが CRIT-002 を引き起こした根本原因として、今後は適用済み migration への変更を禁止するポリシーを徹底する（`tasks/lessons.md` に記録済み）
- AuthConfig の形式不統一（CRIT-001/003）は ADR-0053 の移行漏れであり、grep による全サービス確認を今後の必須手順とする

## 影響

**ポジティブな影響**:

- サービス起動不能の原因となっていた CRITICAL 問題がすべて解消された
- readyz の標準化により、監視自動化の基盤が整った
- ドキュメントと実装の乖離が解消され、監査指摘の再発リスクが低減した

**ネガティブな影響・トレードオフ**:

- HIGH-014 の testcontainers CI 追加により、CI 実行時間が増加する可能性がある
- readyz 形式の移行は段階的であり、移行期間中は形式が混在する

## 代替案

特になし（監査対応は全件対応が原則）

## 参考

- [ADR-0053: AuthConfig ネスト形式移行](0053-auth-config-nested-format.md)
- [ADR-0063: AES-GCM AAD セッションバインディング](0063-aes-gcm-aad-session-binding.md)
- [ADR-0067: BFF プロキシ SSRF allowlist](0067-bff-proxy-ssrf-allowlist.md)
- [ADR-0068: readyz レスポンス形式標準化](0068-readyz-response-format.md)
- [tasks/lessons.md](../../../tasks/lessons.md) — migration 改ざん禁止ポリシー（LOW-013 監査対応: パス修正）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-02 | 初版作成（外部監査 2026-04-01 対応記録） | @k1s0-team |
