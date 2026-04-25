# ADR-DEP-001: 依存更新中枢に Renovate を採用

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / セキュリティチーム / 法務部 / 契約レビュー担当 / SRE

## コンテキスト

k1s0 は tier1 Rust（Cargo crate）、tier1 Go（Go module）、tier2 .NET（NuGet）、tier3 Web（npm / pnpm workspace）、tier3 Native（.NET MAUI）、SDK 4 言語、infra 層の Helm Chart / Kustomize / OpenTofu provider、CI で利用する GitHub Actions アクション、`third_party/` 配下にフォーク vendoring した社内 OSS パッチ群と、計 9 種類の依存グラフを抱える。10 年保守を前提とし、CVE 対応・License 変更検知・breaking change の早期警告を継続的に回す必要がある。

k1s0 は起案者個人による開発であり、採用側の運用体制が小規模であっても拡大しても、依存更新を人手で追う運用は以下の理由で破綻する。

- **CVE 発見から緩和 SLO は 48 時間以内**（NFR-E-SEC 系）である一方、依存パッケージは数千件規模に到達し、週次目視レビューでは到底捕捉できない
- **AGPL 分離アーキテクチャ**（[ADR-0003](ADR-0003-agpl-isolation-architecture.md)）の恒常維持には、間接依存を含めた全ライセンス変化を毎 PR で検証する必要がある。AGPL 系 OSS が同一プロセス境界に紛れ込む回帰を早期に遮断しなければ、法務リスクが蓄積する
- **tier2/tier3 から内部言語を不可視化する方針**（[ADR-TIER1-003](ADR-TIER1-003-language-opacity.md)）の下、tier1 Go の内部依存が tier2 に漏洩しないかを依存グラフで機械検証する必要がある
- **`third_party/` vendoring** はフォーク保守が必須で、`UPSTREAM.md`（上流リビジョン固定）と `PATCHES.md`（独自パッチ一覧）を機械的に突き合わせないと、時間経過でパッチが上流に吸収されても検知できず、不要パッチの負債化が進む
- **Kyverno**（[ADR-CICD-003](ADR-CICD-003-kyverno.md)）の admission ポリシーは「イメージ署名検証」「禁止バージョン除外」等を宣言するが、依存の「更新タイミング」自体を決める機構は持たない。admission の上流で更新提案・PR 化する役割は別途必要になる

GitHub 純正の Dependabot は枯れており GitHub Actions との親和性が最大だが、エコシステム対応数・ライセンス検査・グループ化更新の表現力・AGPL 含む License 台帳連動に決定打を欠く。一方 Mend Renovate（旧 WhiteSource Renovate）は CNCF Landscape に長く在籍し、Go modules / Cargo / npm / pnpm workspace / NuGet / Helm / Kustomize / Terraform / GitHub Actions / Docker / Gradle / Maven・pre-commit hook など、k1s0 が持つ 9 種類の依存グラフを単一の `renovate.json` で横断管理できる唯一の OSS である。

本 ADR は Renovate を k1s0 の依存更新中枢として採用し、自動マージの閾値、vendoring 管理との接続、AGPL 分離恒常検証、採用側の運用規律までを定める。

## 決定

**Renovate（self-hosted OSS 版、Apache 2.0）を k1s0 の依存更新中枢として採用する。**

### 配置と実行形態

- Renovate は self-hosted で運用する。Mend Cloud への委託はオンプレ制約（NFR-E-SEC）により採らない
- 実行基盤は GitHub Actions 上の `renovatebot/github-action` を毎時トリガで起動する schedule workflow とする
- 設定ファイルはリポジトリルートの `renovate.json` 単一に寄せる。サブディレクトリ別設定の分散は小規模運用での棚卸しコストが見合わないため禁止
- `renovate.json` の変更は契約レビュー担当（CODEOWNERS）と SRE の 複数名承認必須。依存更新方針の恣意的な緩和を防ぐ

### 自動マージ閾値

リリース時点では全更新を人手レビューで通す。採用側の運用蓄積後、以下の階層で段階的に自動マージを許容する。

- **patch レベル**（SemVer の 3 桁目変更）かつ CI 全通過の場合、dev 依存に限り自動マージを許容
- **minor レベル**は常時人手レビュー必須。API 追加・内部実装変更を含むため、破壊的でなくとも影響範囲を読む工程を省けない
- **major レベル**は常に人手レビュー必須。breaking change の読み取りと移行計画の伴走が前提となる
- **security 更新**（GHSA / OSV-DB / RustSec 連動）は patch/minor/major を問わず最優先キューに載せ、48 時間 SLO に連動した Slack 通知を行う

自動マージ条件に CI 全通過に加え、`kyverno-policy-check`（禁止バージョン・禁止レジストリの除外）と `license-check`（[ADR-0003](ADR-0003-agpl-isolation-architecture.md) 準拠のライセンス境界維持）の 2 つの必須チェックを追加する。

### `third_party/` vendoring との接続

`third_party/<project>/` 配下には `UPSTREAM.md`（上流リポジトリ URL・pin 済みコミットハッシュ・取り込み日）と `PATCHES.md`（独自パッチ一覧・上流 PR 送付状況）を必須化する。Renovate は `third_party/*/UPSTREAM.md` 内の `upstream_ref:` フィールドを regex manager で解釈し、上流 HEAD との差分を週次で PR 化する。上流にパッチが取り込まれた場合は `PATCHES.md` から該当行を削除し vendoring から外す作業を人手で行うが、差分の検知自体は Renovate が担う。

### AGPL 分離の恒常検証

`license-check` job では間接依存を含む全パッケージのライセンスを `cargo-deny` / `go-licenses` / `license-checker`（npm）/ `dotnet list package --include-transitive` で列挙し、AGPL-3.0 / SSPL / BUSL が k1s0 本体コードの依存グラフに混入していないことを毎 PR で検証する。混入した場合は Renovate PR を自動クローズし、責任者（法務部 + 契約レビュー担当）へ通知する。

### グループ化戦略

無意味に細かい PR が並ぶと小規模運用のレビュー注意力を消耗する。以下のグループ化を `packageRules` で設定する。

- OpenTelemetry 関連 crate / module は単一 PR にまとめる（`otel-*`）
- Argo CD / Argo Rollouts / Argo Workflows の更新を `argo-stack` グループに束ねる
- AWS SDK for Rust / Go のサービス別 crate を `aws-sdk` グループに束ねる
- devDependencies（lint / formatter / type 定義）は月次まとめて 1 PR

### スケジュール

- `schedule: ["before 9am on monday"]` を既定し、月曜朝に PR を集中生成。開発者の認知負荷を週前半に寄せる
- security 更新は `prHourlyLimit: 0` で即時化、`schedule` 除外

## 検討した選択肢

### 選択肢 A: Renovate self-hosted（採用）

- 概要: Apache 2.0 の OSS を GitHub Actions 上で自走、`renovate.json` で全依存グラフを横断管理
- メリット:
  - k1s0 が持つ 9 種類の依存グラフを単一ツールで統括、認知負荷最小
  - `packageRules` の表現力が最も高く、グループ化・スケジュール・自動マージ階層を宣言的に管理
  - regex manager で `third_party/*/UPSTREAM.md` のような独自フォーマットも pin 対象にできる
  - self-hosted で外部 SaaS 依存なし、NFR-E-SEC のオンプレ制約と整合
  - CNCF Landscape での採用実績（Kubernetes / Envoy / Istio プロジェクト等が利用）
- デメリット:
  - `renovate.json` の学習コストが Dependabot より高い（presets の理解が必要）
  - self-hosted 運用でトークン管理・失敗時再実行の Runbook 整備が必要

### 選択肢 B: GitHub Dependabot

- 概要: GitHub 純正、設定は `.github/dependabot.yml`
- メリット:
  - GitHub 側のメンテナンスで可用性が担保される
  - 学習コストが最も低い
  - security-alerts との連携が純正
- デメリット:
  - `third_party/` vendoring のような独自フォーマットへの対応が貧弱
  - ライセンス検査（AGPL 混入検知）の表現力が Renovate より劣る
  - グループ化の粒度が粗く、採用側の運用拡大期に PR 洪水になる
  - Helm Chart / Kustomize / Cargo のエコシステム対応が遅れがち

### 選択肢 C: 人手管理（依存更新ツール不採用）

- 概要: 月次棚卸しで SRE が手動アップデート
- メリット: ツール導入ゼロ
- デメリット:
  - 48 時間 CVE SLO を人手で達成することは数千パッケージ規模では数学的に不可能
  - 小規模運用で破綻、運用拡大期でも続かない
  - AGPL 分離の継続検証を人手で担保することは現実的でない

### 選択肢 D: 外部 SaaS（Snyk / Mend Cloud 等）

- 概要: 商用 SaaS で依存更新・脆弱性スキャン・ライセンス検査を統合提供
- メリット:
  - ベンダーサポートで失敗時の解析負担が減る
  - 脆弱性データベースの独自 curation で検知精度が高い
- デメリット:
  - オンプレ制約（NFR-E-SEC）に合致しない、社内依存グラフを外部に送出
  - 年間ライセンス費が数百万〜千万円規模
  - 採用側で追加の SaaS 契約を立ち上げる意思決定コストが重い

## 帰結

### ポジティブな帰結

- 9 種類の依存グラフを単一 `renovate.json` で統括でき、10 年保守の認知負荷を構造的に最小化
- patch 自動マージにより routine な更新レビューから SRE の注意力が解放され、minor/major のレビューに集中できる
- `third_party/` vendoring が Renovate の regex manager で継続監視され、パッチ吸収による負債化を早期検知
- AGPL 分離検証が毎 PR に組み込まれ、法務リスクが構造的に抑止される
- security 更新の 48 時間 SLO 達成経路が自動化され、エラーバジェット消費を最小化

### ネガティブな帰結

- `renovate.json` の `packageRules` 設計・棚卸しが恒常的な運用コストとして残る
- Renovate のバージョン自体も依存対象であり、Renovate 自身の更新規律を別途定める必要がある
- GitHub Actions の API rate limit に触れる可能性があり、schedule 調整と failback を Runbook 化する必要がある
- 自動マージの閾値緩和を安易に行うと事故を誘発するため、`renovate.json` 変更の 複数名承認ゲートを恒常的に維持する規律が必要

### 移行・対応事項

- `renovate.json` 初期版を k1s0 リリース時点でリポジトリ配下に配置（`packageRules` / `schedule` / `regex manager` 完備）
- GitHub Actions workflow `renovate.yml` を `tools/ci/` 配下に追加し、schedule + manual dispatch で駆動
- `third_party/*/UPSTREAM.md` / `PATCHES.md` の雛形を `docs/00_format/` 配下に追加
- `license-check` job を CI に追加、`cargo-deny` / `go-licenses` / `license-checker` / `dotnet list package` を tier 別に呼び分け
- 自動マージ閾値の開放タイミングで本 ADR を Review ステータスに戻し、運用実績を反映した改訂版を起票
- SRE オンコールの Runbook 目録（`docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md`）に `RB-OPS-003: Renovate 停止時の人手 fallback` を追加
- `CLAUDE.md` の「プロジェクト構造」節に `renovate.json` の位置と所有権を明記

## 参考資料

- [ADR-0003: AGPL ライセンス OSS の分離アーキテクチャ](ADR-0003-agpl-isolation-architecture.md)
- [ADR-CICD-003: Kyverno 採用](ADR-CICD-003-kyverno.md)
- [ADR-TIER1-003: tier2/tier3 からの内部言語不可視](ADR-TIER1-003-language-opacity.md)
- [ADR-DIR-003: Git Sparse Checkout cone mode 採用](ADR-DIR-003-sparse-checkout-cone-mode.md)
- [CLAUDE.md](../../../CLAUDE.md)
- Renovate 公式ドキュメント: [docs.renovatebot.com](https://docs.renovatebot.com)
- GitHub Dependabot 公式ドキュメント: [docs.github.com/en/code-security/dependabot](https://docs.github.com/en/code-security/dependabot)
- OSV-Schema 仕様: [ossf.github.io/osv-schema](https://ossf.github.io/osv-schema)
- GHSA (GitHub Security Advisories) データベース: [github.com/advisories](https://github.com/advisories)
- RustSec Advisory Database: [rustsec.org](https://rustsec.org)
- cargo-deny: [embarkstudios.github.io/cargo-deny](https://embarkstudios.github.io/cargo-deny)
- go-licenses: [github.com/google/go-licenses](https://github.com/google/go-licenses)
- CNCF Landscape "Continuous Delivery - Dependency Management" カテゴリ
