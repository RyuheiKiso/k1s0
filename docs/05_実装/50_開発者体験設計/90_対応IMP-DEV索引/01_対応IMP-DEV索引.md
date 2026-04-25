# 01. 対応 IMP-DEV 索引

本ファイルは `50_開発者体験設計/` 配下で採番した全 49 件の `IMP-DEV-*` ID を一覧化し、ID 採番ルール / 接頭辞別の所在 / 上流 ADR・DS-SW-COMP・NFR への逆引きを提供する。実装段階で「どの ID がどの設計判断に対応するか」を機械可読に追跡するための正典として固定する。

## ID 採番ルール

`IMP-DEV-<sub-prefix>-<番号>` 形式で採番する。`<sub-prefix>` は章ごとに固有で、番号は接頭辞をまたいで**単一番号空間を共有**する（例: POL は 001-009、DC は 010-017、GP は 020-029 のように使い分け、別接頭辞で同番号を再利用しない）。これは将来「番号だけ見て該当文書を見つける」ための一意性確保。

新規採番時は本ファイルの末尾欠番を優先利用し、既存 ID の renumber は禁止（ADR / NFR / 他章の参照リンクが破断するため）。

## 接頭辞別の所在と範囲

| sub-prefix | 章 | 番号レンジ | 件数 | 設計対象 |
|---|---|---|---|---|
| POL | 00_方針/01_開発者体験原則.md | 001-007 | 7 | 開発者体験原則（Paved Road / SLI 原則） |
| DC | 10_DevContainer_10役/01_DevContainer_10役設計.md | 010-017 | 8 | Dev Container 10 役 / sparse-checkout cone |
| GP | 20_Golden_Path_examples/01_Golden_Path_examples.md | 020-026 | 7 | Golden Path examples / Hello World |
| SO | 30_Scaffold_CLI運用/01_Scaffold_CLI運用.md | 030-037 | 8 | Scaffold CLI / Backstage UI 同等性 |
| BSN | 40_Backstage連携/01_Backstage連携.md | 040-048 | 9 | Backstage Catalog / TechDocs / TechInsights |
| ONB | 50_オンボーディング/01_オンボーディング.md | 050-059 | 10 | 入社者 Day 1 〜 Month 1 動線 |

接頭辞 6 種で機能境界が物理的に分離されている。「どの章で起こっている問題か」を ID 接頭辞だけで判定でき、ID を grep するだけで影響範囲を特定できる構造を保つ。

## 全 49 件の ID 一覧

### POL: 開発者体験原則（00_方針）

| ID | 設計内容 |
|---|---|
| IMP-DEV-POL-001 | Paved Road 思想（Netflix 由来）の k1s0 への適用範囲 |
| IMP-DEV-POL-002 | 開発者は「自己責任で off-road」できるが、Paved Road 上では生産性最大化を保証 |
| IMP-DEV-POL-003 | DX を NFR 化し、time-to-first-commit / build-feedback-time を SLI 計測 |
| IMP-DEV-POL-004 | Dev Container / Scaffold / Backstage / Golden Path の 4 軸を「開発者体験パッケージ」として束ねる |
| IMP-DEV-POL-005 | チーム内ローカル文化を許容する範囲（IDE 設定 / 個人 git alias）と禁ずる範囲（リポジトリレベル設定） |
| IMP-DEV-POL-006 | ドキュメント / 動線の自己更新サイクル（詰まりを `@k1s0/platform-dx` がフィードバック化） |
| IMP-DEV-POL-007 | Backstage を「ポータル」ではなく「機械可読 metadata 真実源」として位置付ける |

### DC: Dev Container 10 役（10_DevContainer_10役）

| ID | 設計内容 |
|---|---|
| IMP-DEV-DC-010 | 10 役のリスト確定（tier1-rust / tier2-go / tier3-web / tier3-mobile / sdk / infra-platform / platform-build / release-cd / release-rl / sre-on-call） |
| IMP-DEV-DC-011 | 役割と Dev Container image の一対一対応 |
| IMP-DEV-DC-012 | base image の共通レイヤと役割固有レイヤの分離方針 |
| IMP-DEV-DC-013 | `tools/sparse/checkout-role.sh <role>` による cone 切替自動化 |
| IMP-DEV-DC-014 | 複数 cone 同時保持（`tools/sparse/list-cones.sh`）のサポート |
| IMP-DEV-DC-015 | 社内 cache mirror（image pull 高速化）の運用 |
| IMP-DEV-DC-016 | postCreate script（pre-commit / mise / make seed の初回実行） |
| IMP-DEV-DC-017 | image 更新時の Renovate 連動と全役割同時バージョン管理 |

### GP: Golden Path / examples（20_Golden_Path_examples）

| ID | 設計内容 |
|---|---|
| IMP-DEV-GP-020 | `src/examples/` 配下に役割別 `goldenpath/<role>-hello.md` を配置 |
| IMP-DEV-GP-021 | 5 step 完走を Hello World 完了の定義とする |
| IMP-DEV-GP-022 | 5 step が崩れたらメンターが即時 PR を起こす（参画者にバグ修正させない） |
| IMP-DEV-GP-023 | examples は make 1 コマンドで完了させ、条件付き README を禁ずる |
| IMP-DEV-GP-024 | examples の対象は Dapr Local + Postgres + Redis の最小構成 |
| IMP-DEV-GP-025 | examples のテストを CI で常時実行し、壊れたら即 issue 化 |
| IMP-DEV-GP-026 | Golden Path のメンテナ責任を `@k1s0/platform-dx` に集約 |

### SO: Scaffold CLI 運用（30_Scaffold_CLI運用）

| ID | 設計内容 |
|---|---|
| IMP-DEV-SO-030 | Scaffold 経由を全新規 component 作成の必須経路とする |
| IMP-DEV-SO-031 | CLI 直接 / Backstage UI の 2 経路は共有 scaffold engine で同等性保証 |
| IMP-DEV-SO-032 | 出力同等性を golden test で機械検証（バイト一致） |
| IMP-DEV-SO-033 | 3 sub-command 固定（`list` / `new` / `new --dry-run`）と引数拡張禁止 |
| IMP-DEV-SO-034 | 4 入力項目固定（template / name / owner / system）と入力拡張禁止 |
| IMP-DEV-SO-035 | 4 出力 artifact 必須（コード雛形 / catalog-info.yaml / CODEOWNERS 追記 / docs README） |
| IMP-DEV-SO-036 | 出力後の `make check` 通過を生成成功の判定とし、失敗時は ロールバック |
| IMP-DEV-SO-037 | Backstage の自動 discovery 連動と template 更新時の影響範囲可視化 |

### BSN: Backstage 連携（40_Backstage連携）

| ID | 設計内容 |
|---|---|
| IMP-DEV-BSN-040 | entity 5 種別固定（Component / API / Group / System）と種別追加の ADR 起票必須化 |
| IMP-DEV-BSN-041 | Component の path 直下同居配置 vs Group / System の `catalog/` 集約配置の使い分け |
| IMP-DEV-BSN-042 | catalog-info.yaml の必須 5 属性と `k1s0.io/template-version` annotation |
| IMP-DEV-BSN-043 | GitHub provider 5 分 polling と webhook 不採用の判断 |
| IMP-DEV-BSN-044 | CI 段の `backstage-cli catalog:validate` 必須化と pre-validation 経路 |
| IMP-DEV-BSN-045 | `@backstage/cli` バージョン pin と Backstage 本体との同期更新 |
| IMP-DEV-BSN-046 | Catalog Errors の `@k1s0/platform-dx` 日次目視運用（Slack 連動は採用初期） |
| IMP-DEV-BSN-047 | TechInsights 4 ファクト（coverage / lint / sbom / cosign）の実装範囲 |
| IMP-DEV-BSN-048 | catalog-info.yaml を真実源とし Backstage は表示層とする復旧構造 |

### ONB: オンボーディング（50_オンボーディング）

| ID | 設計内容 |
|---|---|
| IMP-DEV-ONB-050 | time-to-first-commit を SLI 化し、Day 1 4 時間以内（採用拡大期 2 時間以内）を目標に固定 |
| IMP-DEV-ONB-051 | Day 0 の HR / IT / メンター責務分担と入社前々日までの Backstage Group 登録 PR |
| IMP-DEV-ONB-052 | Day 1 4 step（役割確定 / 環境構築 / Hello World / 微小 PR）と各 step の時間予算 |
| IMP-DEV-ONB-053 | Step 3 の `goldenpath/<role>-hello.md` 5 step 完走の絶対要件と崩壊時のメンター修繕義務 |
| IMP-DEV-ONB-054 | 微小 PR を「最初の merge を儀式化する」設計とその範囲（typo / catalog-info / docs） |
| IMP-DEV-ONB-055 | SLI 計測経路（onboardingTimeFactRetriever / Scaffold 自動フッタ付与）と計測のみのリリース運用 |
| IMP-DEV-ONB-056 | Week 1 の学習リスト（ADR-DIR-001/003 / ADR-TIER1-001 / 90_knowledge）と Scorecards 連動 |
| IMP-DEV-ONB-057 | Week 2 〜 4 の実 task 着手と複数 cone 併用の段階導入 |
| IMP-DEV-ONB-058 | Month 1 自走判定の 4 軸（PR 量 / レビュー受領 / Slack / オンコール）と判定後 HR 1on1 |
| IMP-DEV-ONB-059 | `onboarding-stumble` label による動線詰まり記録と月次 PR による帰着先（DC / GP / SO / BSN / ONB） |

## ADR 逆引き

各 IMP-DEV ID が根拠とする ADR を逆引きするための表。複数 ADR にまたがる ID は複数行に展開する。

| ADR | 関連 IMP-DEV ID |
|---|---|
| ADR-DEV-001（開発者体験パッケージ採用） | POL-001 〜 007 / DC-010 / GP-020 / SO-030 / BSN-040 / ONB-050 |
| ADR-BS-001（Backstage 採用） | POL-007 / SO-037 / BSN-040 〜 048 / ONB-056 |
| ADR-DIR-001（contracts 昇格 / monorepo 構造） | DC-013 / SO-035 / BSN-041 |
| ADR-DIR-003（sparse-checkout cone 必須） | DC-013 / DC-014 / DC-015 / ONB-052 / ONB-057 |
| ADR-TIER1-001（Go / Rust ハイブリッド） | DC-010 / DC-011 / GP-024 / ONB-056 |
| ADR-CICD-001（GitHub Actions 採用） | SO-036 / BSN-044 / GP-025 |

ADR-DEV-001 / ADR-BS-001 が本章の主要根拠で、DC / SO / BSN / ONB の 4 章はほぼ全 ID がこの 2 ADR に帰着する。POL は ADR-DEV-001 を全面的に物理化する位置付け。

## DS-SW-COMP 逆引き

| DS-SW-COMP | 関連 IMP-DEV ID |
|---|---|
| DS-SW-COMP-132（SDK 4 言語配布） | POL-002 / GP-020 〜 026 / SO-035 / ONB-052 / ONB-056 |
| DS-SW-COMP-135（配信系インフラ = Harbor / ArgoCD / Backstage / Scaffold） | POL-002 / POL-007 / SO-030 〜 037 / BSN-040 〜 048 / ONB-051 / ONB-055 |

DS-SW-COMP-135（配信系インフラ）が Scaffold / Backstage 系の主要帰着点。SDK 4 言語配布（132）は examples / Hello World / 学習リストから間接参照される。Onboarding 章の Day 1 4 step / Month 1 自走判定など「動線そのもの」を直接受ける DS-SW-COMP は リリース時点 では未確定（DS-SW-COMP-141 以降での DX 系コンポーネント定義は採用初期で起こす）。

## NFR 逆引き

| NFR | 関連 IMP-DEV ID |
|---|---|
| NFR-C-NOP-001（学習容易性） | POL-001 / POL-002 / GP-020 / GP-021 / GP-024 / ONB-050 / ONB-052 / ONB-053 |
| NFR-C-NOP-002（可視性） | POL-007 / BSN-040 / BSN-046 / BSN-048 / ONB-055 / ONB-058 |
| NFR-C-MGMT-001（設定 Git 管理） | DC-013 / SO-031 / SO-035 / BSN-042 / BSN-048 |
| NFR-C-MGMT-002（変更容易性） | POL-006 / SO-037 / BSN-045 / ONB-059 |
| NFR-E-OPR-001（運用性） | DC-015 / DC-017 / BSN-046 |

NFR-C-NOP-001（学習容易性）は本章の主目的の 1 つで、Day 1 4 時間 SLI（ONB-050）と Hello World 5 step（GP-021）が両輪。NFR-C-NOP-002（可視性）は Backstage を真実源化（POL-007 / BSN-040）し SLI で検証する（ONB-055）構造で支える。

## トレーサビリティ更新運用

新規 IMP-DEV-ID 採番時の手順:

1. 本ファイルの「全 49 件の ID 一覧」を更新（接頭辞表とレンジ管理表も同時更新）
2. `docs/05_実装/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md` の DEV 行を更新
3. 該当 ADR を `docs/05_実装/99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md` で更新
4. 該当 NFR を `docs/05_実装/99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md` で更新
5. DS-SW-COMP-132 / 135 を `docs/05_実装/99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md` で更新

5 ファイルを同一 PR で更新することで、索引のドリフトを防ぐ。漏れた場合は PR レビュー段階で `docs-review-checklist` Skill が指摘する設計。
