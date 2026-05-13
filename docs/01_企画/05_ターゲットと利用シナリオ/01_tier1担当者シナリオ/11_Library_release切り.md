---
id: plan.tier1.scenario_library_release
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - B
    - E
  proof_classes: []
---

# Library release 切り

## 一文方針

tier1 担当者が 4 言語 Library（Rust / C# / Go / TypeScript）の SemVer release（minor / major / patch）を切り、各 package registry（crates.io mirror / NuGet Harbor mirror / pkg.go.dev / npm Harbor mirror）に publish する。

> 朝 10 時、本社 IT 室の tier1 担当者（シニア級）が GitHub PR list を確認し、「v0.18 minor release milestone が close できる状態になった」という milestone 更新通知に気付く。手元には `public_api_snapshot.lock.yaml` と 4 言語の CI ダッシュボード、Mattermost 越しに dual reviewer 2 名と tier2 担当者がいる。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: `release_gate.lock.yaml` 全 cell green を物理前提条件として 4 言語 Library の SemVer release を切り各レジストリに publish する

## 現状業務での痛み

- release 可否判断が属人的で、CVE 混入や conformance fail の状態で release が出てしまうことがある
- release 手順書が陳腐化し、手順を省略した release がたびたび発生している
- major release の breaking change に migration guide が付かず、tier2/tier3 担当者が自力で対応する羽目になる
- cosign 署名のステップが抜け落ちて未署名パッケージが公開される事例が散見される

## k1s0 でこう変わる

- `release_gate.lock.yaml` の全 cell（conformance test / CVE / snapshot / dry-run 等）green が物理的な release 前提条件となる
- CHANGELOG の自動生成（Conventional Commits）と breaking change への migration guide 必須付与で手順の省略を構造的に防ぐ
- cosign 署名と Harbor push を CI の release gate に組み込み、未署名 publish を物理拒否する
- release 後 24h フィードバック監視を Mattermost `#tier1-release` で標準化し、初動対応を迅速化する

## Trigger（発火条件）

Library に十分な機能追加 / バグ修正が蓄積し、release milestone が達成された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次〜四半期（minor release）/ 年次（major release）/ 随時（patch release）
- 典型きっかけ: 「tier1 Library v0.18 minor release に向け 5 件の機能追加と 2 件の bugfix が蓄積され release milestone が close できる状態になった」「CVSS 対応の hotfix で patch release v0.17.3 が必要になった」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: tier2 担当者（互換性確認）
- 承認: dual reviewer（tier1 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | GitHub PR list | SemVer 決定・release branch 作成・version bump・cosign 署名・Harbor push |
| 関与（tier2 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | 互換性確認・release 後 24h フィードバック |
| 承認（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | CHANGELOG レビュー・sign-off・release tag 確認 |
| 承認（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | CHANGELOG レビュー・sign-off・release tag 確認 |

## 個人 KPI / 達成感

- 4 言語 Harbor mirror への publish 完了（未署名 0 件）
- Testcontainers conformance test green（release 後）
- major release の breaking change への migration guide 添付率 100%
- release 後 24h フィードバックでのクリティカル issue 0 件

## 工数 / 関与人数 / コスト感

- 初回（minor release）: 半日〜1 日（SemVer 確定・version bump・CI green・cosign・publish）、関与 3〜4 名（主役 + dual reviewer 2 名 + tier2 担当者 1 名）
- 平常（patch release 緊急 hotfix）: 2〜3h、関与 3 名
- 失敗時（major release で tier2/tier3 build fail・postmortem）: +1〜2 日、関与 5 名以上

## 前提

- 公開 API snapshot が `public_api_snapshot.lock.yaml` に記録済み
- CHANGELOG は commit message から自動生成（Conventional Commits 形式）
- 4 言語全ての Testcontainers conformance test が CI green
- 月次 SBOM CVE トリアージ（シナリオ 10）で CVSS 9.0 以上の未対応 CVE がある場合は release を保留し、CVE 対応が完了してから本シナリオを実施する。

## 流れ

1. release 対象の commit を確認し SemVer バンプ種別（major / minor / patch）を決定する
   - public API snapshot diff を確認: breaking change がある場合は major
   - 新機能追加は minor、bugfix のみは patch
   - `cargo-semver-checks` で Rust の breaking change を自動検出
2. release branch を切り、4 言語の version bump を実施する
   - Rust: `Cargo.toml` の `version` フィールドを更新
   - C#: `.csproj` の `<Version>` タグを更新
   - Go: `go.mod` の module path に `/v<major>` suffix を追加（major の場合のみ）
   - TypeScript: `package.json` の `version` を更新
3. 4 言語全ての Testcontainers conformance test + L2* 同族保証 test を CI で実行し green を確認する
4. リリースノート（CHANGELOG）を review し、breaking change / 新機能 / bugfix の節を確認する
   - breaking change には migration guide（業務コード移行ガイド）を必ず付与
5. `public_api_snapshot.lock.yaml` を新バージョンの snapshot で更新する
6. cosign で image / package に署名してから Harbor mirror へ push する
7. 内部 Harbor mirror で動作確認（tier2 / tier3 が pull できることを Testcontainers で確認）
8. dual reviewer sign-off を取得し release tag を push する
9. release 後 24h は tier2 / tier3 担当者からのフィードバックを監視する（Mattermost `#tier1-release`）

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | SemVer 確定・release branch 作成・4 言語 version bump | `v0.18.0 minor / branch: release/0.18.0 / 4 言語 bump 中` |
| 2h | tier1 担当者 | conformance test CI・CHANGELOG review・snapshot 更新 | `CI green / CHANGELOG 確認完了 / snapshot 更新済` |
| 3h | tier1 担当者 | cosign 署名・Harbor push・内部動作確認 | `cosign signed / Harbor push 完了 / tier2 pull 確認中` |
| 4h | dual reviewer A/B | sign-off・release tag push | `dual sign-off 完了 / tag v0.18.0 push 済` |
| +24h | tier1 担当者 | tier2/tier3 フィードバック監視 | `#tier1-release 監視中 / issue なし` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **受注**: Library release は受注業務が依存する tier1 API の変更を全下流に届ける節目であり、major release での backward 非互換変更は受注処理フローの再検証を必要とする。
- **警報配信**: Library release に含まれる bugfix や security patch は警報配信の信頼性・暗号化強度を直接改善し、patch release の迅速な展開が警報業務の継続稼働を支える。

## 関連適合仕様 / 関連 OSS

- 関連適合仕様
  - [検証規律適合仕様（公開 API snapshot 関連）](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md)
  - [OSS ライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- 関連 OSS
  - cargo-semver-checks（Rust breaking change 検出）
  - Cosign（署名）
  - Harbor（mirror）
  - Conventional Commits（CHANGELOG 自動生成）

## 期待結果 / 観測指標

- 4 言語で Harbor mirror への publish 完了
- cosign 署名 green
- Testcontainers conformance test green
- `public_api_snapshot.lock.yaml` 更新完了
- dual sign-off 取得

## 失敗時の挙動 / escalation

- **major release で tier2 / tier3 のコードが build fail**: 即座に rollback は不可（既 publish）。migration guide を補強し tier2 / tier3 担当者を Mattermost `#tier1-release` で支援（**SLA: 24h 以内**に migration guide 完成）。**postmortem 期限: 3 営業日以内**。
- **cosign 署名失敗**: 未署名の package を registry に push しない。infra 担当者に Mattermost `#infra-incident` で Cosign 鍵確認を依頼（**SLA: 1h 以内**）。
- **Testcontainers conformance fail（release 後）**: パッチリリース（patch bump）で修正し再 release。**postmortem 期限: 2 営業日以内**。

## 失敗パターン (anti-pattern)

- **CVSS 9.0 以上の未対応 CVE があるまま release**: セキュリティ上危険な状態のパッケージが公開される。月次 CVE トリアージ（シナリオ 10）の完了を release gate の必須条件とする。
- **major release に migration guide を付けずに publish**: tier2/tier3 担当者が自力で対応できず build fail が発生し、緊急サポート対応に大量の時間を消費する。breaking change には migration guide を必ず同梱することを CI の release checklist で強制する。
- **cosign 署名なしで register に push**: 未署名パッケージが内部 Harbor mirror に混入し、supply chain lint が fail し始める。署名なし push を Harbor の push policy で物理拒否する設定を infra 軸で担保する。

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成と担当者プロフィール
- [公開 API snapshot 違反対応](./08_公開API_snapshot違反対応.md) — release 前に snapshot 不一致が検出された場合のシナリオ
- [OSS ライフサイクルイベント対応](./02_OSSライフサイクルイベント対応.md) — major release が OSS lifecycle イベントと重なる場合の扱い
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md) — Library の 3 パッケージ構成（core / frontend / backend）と 17 カテゴリの SoT
