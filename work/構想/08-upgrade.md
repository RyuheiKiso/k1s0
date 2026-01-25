## 8. framework のアップグレード支援（互換性ポリシー）

CLI が「自動アップグレード支援」を成立させるため、互換性の定義と破壊的変更の扱いを固定する。

### バージョンの単位

- k1s0 はリポジトリ全体で単一バージョン（SemVer）を持つ（例: `v1.2.3`）。
	- 対象: CLI / generator / templates / framework crates / framework 共通サービス雛形
	- 目的: 導入・アップグレード時に整合する組み合わせを常に 1 つにする

### SemVer（互換性）の定義

- MAJOR: 破壊的変更（後方互換なし）
	- 例: 公開 API の破壊（HTTP/gRPC/GraphQL の契約変更、主要レスポンス形/必須フィールド変更）
	- 例: framework crates の公開 API 破壊（公開関数/型/feature の削除、互換性のない挙動変更）
	- 例: テンプレの構造的変更（移動/削除により既存サービスがビルド不能になる等）
	- 例: DB の破壊（列削除・型変更など即時互換性を失う変更）
- MINOR: 後方互換ありの機能追加（例: 任意フィールド追加、エンドポイント追加、deprecate 導入）
- PATCH: 後方互換ありの不具合修正/小改善（例: バグ修正、互換性維持の依存更新、テンプレの軽微な安全修正）

### 破壊的変更の扱い（必須運用）

- 破壊的変更は原則禁止。必要な場合は段階移行（追加→移行→切替→削除）を基本とし、例外のみ MAJOR で実施する。
- MAJOR を行う場合、同一 PR で次を必須とする。
	- ADR
	- `UPGRADE.md` もしくはリリースノート
	- 自動/半自動アップグレード手段（CLI のパッチ適用、変換スクリプト、チェックコマンド追加等）

### テンプレ差分適用の戦略（CLI の責務）

CLI はアップグレード時に「上書きしてよい領域」と「業務コード領域」を分離して扱う。

- 原則: 管理対象（CLI が更新する）
	- `deploy/`（Kustomize）
	- `config/`（環境別 yaml）
	- `openapi/`（雛形・共通ルールに関する部分）
	- `migrations/`（生成/追加のみ。既存の適用済みファイルは改変しない）
	- ビルド/静的解析/CI に関する共通設定
- 原則: 非管理対象（CLI は変更しない）
	- `src/domain/` / `src/application/`
- 境界（衝突しやすい）
	- `src/main.rs` / `src/presentation/` / `src/infrastructure/` は更新候補の提示（diff 出力）を基本とし、衝突時は手動解決

衝突時の挙動

- `k1s0 upgrade --check`: 影響箇所と衝突を事前検出
- `k1s0 upgrade`: 安全に適用できる変更のみ自動適用し、衝突は明示して停止

### upgrade 成立のためのメタ情報（プロジェクト側に保存・必須運用）

CLI がテンプレ差分を安全に適用するため、プロジェクト側にメタ情報を必ず保存する。

- 保存先: `.k1s0/manifest.json`（CLI 管理。人手編集しない）
- 目的
	- 生成元テンプレ/バージョンを確定
	- 管理対象ファイルの特定と上書き可否/差分適用/衝突検知
	- upgrade 時に期待する生成手順（契約生成/CI 設定等）の固定

`.k1s0/manifest.json` の内容（案）

- `k1s0_version`
- `template`: `name`, `version`, `source`, `path`, `revision`, `fingerprint`
- `generated_at`
- `service`: `service_name`, `language`
- `managed_paths`
- `protected_paths`
- `update_policy`（例: `suggest_only`）
- `checksums`（任意だが推奨）
- `template_snapshot`（任意だが推奨）

CLI の挙動（メタ情報の利用）

- `k1s0 init` / `k1s0 new-feature`: テンプレ展開後に `.k1s0/manifest.json` を生成
- `k1s0 upgrade --check`
	- 現在の k1s0 と `manifest.json` の差を計算
	- 旧テンプレ特定情報が不足している場合は停止し、移行手順を案内
	- `managed_paths` の改変（`checksums`）を検知し、衝突リスクを提示
	- MAJOR 対象なら ADR / `UPGRADE.md` の存在をチェック
- `k1s0 upgrade`
	- 旧テンプレ→新テンプレ差分を作成し、`managed_paths` のみ更新
	- `suggest_only` は diff を提示して停止
	- `protected_paths` は原則触らない（差分検知のみ）

### マイグレーション自動適用の可否

- dev: 任意で自動適用を許可（例: `k1s0 upgrade --apply-migrations`）
- stg/prod: 自動適用は禁止（生成のみ）。適用は運用手順で行う
- 自動適用の有無に関わらず、マイグレーションファイルは差分として追加し、ロールバック方針を README に必ず記載する

---


