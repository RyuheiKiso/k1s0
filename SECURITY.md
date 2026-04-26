# Security Policy

k1s0 のセキュリティ脆弱性報告 / 対応ポリシー。

## 報告窓口

セキュリティ脆弱性を **公開 issue / Discussion / PR で報告しないでください**。攻撃者に手の内を見せる前に修正版を配布する必要があるため、以下の手段で **非公開** に報告してください。

- **GitHub Security Advisories**: [Report a vulnerability](https://github.com/k1s0/k1s0/security/advisories/new)（推奨）
- 上記が利用できない場合: メンテナへの非公開連絡（リポジトリ owner の GitHub プロフィール経由）

## 期待する報告内容

- 影響範囲（バージョン / 構成 / 影響を受ける tier）
- 再現手順（PoC、最小再現コード）
- 想定される攻撃シナリオ / 影響度
- 報告者の連絡先（advisory での credit 公開可否）

## 対応 SLA

- **初動応答**: 営業日 3 日以内（受領確認 + 重要度の暫定判定）
- **修正配布**: Severity に応じた embargo 期間（詳細は [`plan/16_OSS公開準備/10_coordinated_disclosure_policy.md`](plan/16_OSS公開準備/10_coordinated_disclosure_policy.md) ※非公開ファイル参照）
  - **Critical**: 7 営業日以内に embargo 解除 + advisory 公開
  - **High**: 30 営業日以内
  - **Medium / Low**: 次回マイナーリリースに含める
- **CVE 採番**: GitHub Security Advisory 経由で MITRE / GitHub CNA に申請

## 対応プロセス（概要）

1. 報告受領 → 受領確認返信
2. 重要度トリアージ + 影響範囲調査
3. 修正開発（プライベートブランチ、報告者と共有可）
4. 修正版リリース + advisory 公開（embargo 解除）
5. CVE 採番 + 公表
6. ポストモーテム（必要時、`docs/40_運用ライフサイクル/` 配下）

詳細は plan 16-10 の Coordinated Disclosure Policy 7 runbook（embargo 開始 / コミュニケーション / patch 開発 / advisory 起票 / CVE 申請 / 公開 / 後追い）を参照。

## サポート対象バージョン

リリース時点では `main` ブランチのみセキュリティパッチを提供する（リリース前のため）。タグ付けされたリリースが出た時点で「最新メジャー + 1 つ前のメジャー」を support window とし、それ以前のバージョンは EOL とする。

## 対象範囲

本ポリシーは k1s0 リポジトリの直接的な成果物（`src/` / `infra/` / `deploy/` / `tools/` / `ops/`）を対象とする。以下は対象外:

- 上流依存（Dapr / Istio / Postgres / Kafka / OpenBao 等）→ 各プロジェクトの security policy へ報告
- 採用側で改変したフォーク / 派生物
- 依存ライブラリの脆弱性（Renovate + Trivy で別途追跡、自動 PR が出る）

## 既知の制限事項

リリース時点で既知のセキュリティ未対応領域は以下:

- TLS / mTLS（SPIRE 連携）は plan フェーズ 13 で実装予定、リリース時点では `tlsConfig: insecure` 状態
- gRPC interceptor の認証 / 認可は plan 04-21 で実装予定、リリース時点では handler 層に未配線
- secret rotation は plan 13 / OpenBao policy で対応、リリース時点では手動

詳細は `docs/03_要件定義/30_非機能要件/E_セキュリティ/` 参照。

## ハードニング ガイダンス

採用側で k1s0 を運用する場合の最低限のハードニング:

- `infra/security/openbao/policies/` の最小権限ポリシーを必ず適用
- `infra/security/keycloak/realm-export.yaml` の DefaultRealm を本番用に置換
- `infra/security/spire/` の trust domain を組織固有値に変更
- `infra/k8s/networking/network-policy/` を有効化（namespace 越境トラフィック制限）
- 詳細は `docs/40_運用ライフサイクル/` の運用ガイド参照

## ライセンス

本リポジトリは Apache License 2.0 で提供されています（[LICENSE](LICENSE)）。脆弱性報告で生成される PoC コード等の知的財産権は報告者に帰属し、k1s0 メンテナはセキュリティ対応の範囲内でのみ使用します。
