# k1s0 ガバナンス

本ファイルは k1s0 プロジェクトの意思決定プロセスとロールを規定する。
個人開発 OSS としての小規模性を前提に、最小限のガバナンス構造を採る。

## プロジェクトの性質

k1s0 は起案者個人が業務外時間で開発した OSS であり、リリース時点で全機能を
一気通貫に同梱した状態で公開されている（README「リリース範囲」参照）。

本プロジェクトは以下を満たさないことを明示する:

- 専任の開発チームを抱えない
- 24/365 の対応コミットメントを持たない
- ベンダー・スポンサーシップによる優先機能開発を受け付けない

採用側組織が本プロジェクトを業務に組み込む場合は、必要な機能の追加・拡張を
組織側で実施し、上流還元を歓迎する Maintainer 関与モデルで運用される。

## ロール

| ロール | 権限 | 責務 |
|---|---|---|
| **Maintainer** | リポジトリ管理（branch protection / merge / release） | 全 PR レビュー / リリース判断 / 行動規範施行 |
| **Reviewer** | PR レビュー権限（CODEOWNERS で領域別） | 担当領域の PR の建設的レビュー |
| **Contributor** | PR 作成・Issue 起票 | 貢献内容の品質保証（CONTRIBUTING.md 準拠） |

リリース時点では Maintainer = 起案者単独。Reviewer / Contributor は
コミュニティの自然成長に応じて任命する。

## 意思決定プロセス

### 軽微な変更（バグ修正・ドキュメント追加・テスト追加等）

- Issue 不要、PR 直接で OK
- Maintainer 1 名の Approve でマージ

### 構造的変更（新 API / 新 tier / 新 ADR / アーキテクチャ変更）

- 事前に Issue または ADR PR で議論を起こす
- ADR を起票し、CONTRIBUTING.md および `docs-adr-authoring` Skill に従って
  検討肢 3 件以上を比較する
- ADR PR がマージされた後、実装 PR を切り出す
- Maintainer 1 名の Approve（リリース時点単独メンテナ運用のため）

### 破壊的変更（後方互換性を壊す変更）

- ADR 必須かつ「Superseded 旧 ADR」の明示が必須
- リリース時点では SemVer 0.x 系で運用するため、minor 単位で破壊的変更を許容
- 1.0 以降は major bump 必須

## リリースプロセス

詳細は `docs/05_実装/70_リリース設計/` 参照。要点:

- リリースタグは `v<major>.<minor>.<patch>`（SemVer 2.0）
- 各リリースの change log は GitHub Releases で管理
- リリース時点（v0.1.0）以降の実装拡張は、SHIP_STATUS.md のマチュリティ表で
  領域別に開示される

## コードオーナーシップ

詳細は [`.github/CODEOWNERS`](.github/CODEOWNERS) 参照。

## 行動規範

[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)（Contributor Covenant 2.1 準拠）。

## セキュリティ報告

[`SECURITY.md`](SECURITY.md) 参照。脆弱性報告は GitHub private vulnerability
reporting 機能、または同ファイル記載の経路で。

## ライセンス

[Apache License 2.0](LICENSE)。

依存 OSS のうち AGPL ライセンスのもの（MinIO ほか）の隔離設計と義務発動なしの
判定は `docs/02_構想設計/05_法務とコンプライアンス/` を参照。
