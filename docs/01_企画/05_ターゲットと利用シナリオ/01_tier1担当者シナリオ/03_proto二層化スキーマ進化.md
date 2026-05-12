---
id: plan.tier1.scenario_proto_schema_evolution
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# proto 二層化スキーマ進化

## 一文方針

External / Internal 2 層 proto 構成を維持しつつ、[Apicurio Schema Registry](../../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md) の compatibility check と 4 言語コード生成の全 green を merge 条件として、backward / forward 互換を tier1 facade 内部で完結させる。

> 朝 9 時、本社 IT 室の tier1 担当者（シニア級）が buf CI ダッシュボードを確認し、tier2 から来た「受注 Domain Event への配送先住所フィールド追加」要求の PR が上がっていることに気付く。手元には Apicurio Schema Registry UI と buf CI、Mattermost 越しに dual reviewer と tier2 担当者がいる。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: External/Internal 二層 proto の境界を守りつつ backward/forward 互換を facade 内部で完結させる

## 現状業務での痛み

- External/Internal proto の境界が曖昧で、External への breaking change が tier3 まで波及し気付くのが遅い
- buf breaking CI が整備されておらず、proto の breaking change を PR review の人手確認のみに依存している
- Apicurio の compatibility check が CI に組み込まれておらず、スキーマの後方互換性確認が手動になっている
- 4 言語コード生成の CI が分散しており、一部言語で生成 fail が見落とされる

## k1s0 でこう変わる

- External/Internal 二層化 + buf breaking CI + Apicurio GitOps で breaking change を merge 前に物理拒否する
- FULL_TRANSITIVE compatibility check が CI gate となり、過去全バージョンとの互換性を自動担保する
- 4 言語（Rust/C#/Go/TypeScript）のコード生成 CI が一括 green を merge 条件とし、言語ごとの見落としを排除する
- `apicurio_gitops_sot.lock.yaml` がスキーマバージョンの SoT となり、手動確認の属人性をゼロにする

## Trigger（発火条件）

業務 API 変更で External Proto か Internal Proto の backward / forward 互換を確保する必要が生じた時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 週次〜月次（External Proto の breaking change は四半期）
- 典型きっかけ: 「受注 Domain Event に配送先住所フィールドを追加する要求が tier2 から来た」「検査結果スキーマで数値精度型変更が必要になった」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ tier2 担当者（下流影響の確認のみ）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | buf CI / Apicurio UI | External/Internal proto 変更・互換方針決定・lock ファイル更新・PR 提出 |
| 関与（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | proto 差分レビュー・sign-off |
| 関与（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | proto 差分レビュー・sign-off |
| 関与（tier2 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | 下流影響の確認・フィールド追加要求の提起 |

## 個人 KPI / 達成感

- buf breaking 意図外検出 0 件（unexpected breaking change ゼロ）
- Apicurio FULL_TRANSITIVE compatibility check green 率
- 4 言語コード生成 all green 維持
- dual reviewer 応答時間 ≤ 24h

## 工数 / 関与人数 / コスト感

- 初回（non-breaking フィールド追加）: 半日、関与 3〜4 名（主役 + dual reviewer 2 名 + tier2 担当者 1 名）
- 平常（非互換で互換ラッパー追加）: 1〜2 日、関与 3〜4 名
- 失敗時（major SemVer bump・下流影響広範）: 2〜3 日、関与 5〜6 名（+ 影響 tier2/tier3 担当者）

## 前提

- External / Internal 2 層 proto + Transport Adapter Layer 構成が確立済み（[tier1 設計方針 01_Server系](../../../03_概要設計/02_tier1設計方針/01_Server系.md) 参照）
- Apicurio Schema Registry が稼働しており `apicurio_gitops_sot.lock.yaml` が管理されている
- Buf によるプロトコルバッファ lint / breaking change 検出が CI に組み込まれている
- 4 言語（Rust / C# / Go / TypeScript）のコード生成パイプラインが CI に組み込まれている

## 流れ

1. **External Proto 変更案の draft**: 変更種別を明示した draft を作成する。
   - field 追加（backward 互換）: field number を新規採番し既存 field には影響なし
   - field 削除（backward 非互換）: 削除前に reserved 宣言を挟む移行ステップが必要
   - 型変更（backward 非互換）: 原則として型変更は禁止。新 field を追加し旧 field を deprecated マークする形で対応する
   - oneof / map の変更: 追加は互換、削除・型変更は非互換として扱う

2. **External / Internal 差分検査**: CI の diff 検査で「External Proto の変更が Internal Proto に意図せず漏洩していないか」を確認する。Transport Adapter Layer は External と Internal を明示的に分離する責務を持つため、layer 間の proto フィールド対応が崩れていないことを検証する。

3. **互換方針の判定**: backward / forward 互換性の有無を判定し対応方針を決定する。
   - 互換あり: External Proto のみ更新し Internal Proto は変更不要の場合は Transport Adapter Layer のみ修正
   - 非互換で External/Internal 二層吸収: Internal Proto に互換ラッパーを追加し External は新旧両 API surface を提供
   - 非互換で SemVer major bump 必要: External Proto の major バージョンを上げ旧 endpoint を deprecation 期間後に廃止

4. **Apicurio Schema Registry への push と compatibility check**: 候補スキーマを Apicurio Schema Registry に push し compatibility check を実行する。
   - FULL_TRANSITIVE: 過去全バージョンに対して互換性を検証（推奨）
   - BACKWARD: 直前バージョンのみ検証（移行期間中の暫定設定時のみ）
   - compatibility check fail は merge 阻止の gate となる

5. **4 言語コード生成の検証**: Buf の `buf generate` を 4 言語（Rust / C# / Go / TypeScript）に対して実行し、全て成功することを確認する。生成コードの型エラーも含めてコンパイルが通ることを検証する。

6. **dual reviewer sign-off と lock ファイル更新**: 互換 CI green を確認後、dual reviewer（tier1 2 名）が sign-off する。`apicurio_gitops_sot.lock.yaml` に採用スキーマバージョンを記録する。

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | PR 確認・変更種別判定（backward 互換 / 非互換）| `buf CI fail / 変更種別: field 追加（backward 互換）/ 方針決定中` |
| 0.5 日 | tier1 担当者 | External/Internal 差分検査・互換方針確定 | `互換方針: Transport Adapter Layer のみ修正で吸収可` |
| 1 日 | tier1 担当者 | Apicurio push・compatibility check・4 言語コード生成 green | `Apicurio FULL_TRANSITIVE green / 4 言語コード生成 green` |
| 1.5 日 | dual reviewer A/B | sign-off・lock ファイル更新 | `dual sign-off 完了 / apicurio_gitops_sot.lock.yaml 更新済` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **受注**: 受注 Domain Event への新フィールド追加・型変更は受注業務の契約情報伝達に直結し、非互換変更が発生すると受注処理が停止するリスクがある。
- **SCADA テレメトリ**: 検査結果スキーマの数値精度型変更は SCADA から収集する計測値の精度に影響し、品質基準の逸脱検出に誤差が生じる可能性がある。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [スキーマ進化適合仕様](../../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)
- [apicurio_gitops_sot](../../../04_詳細設計/03_クロスカッティング適合仕様/03_apicurio_gitops_sot.md)

**関連 OSS**:
- Buf: proto lint / breaking change 検出 / コード生成
- Apicurio Schema Registry: スキーマ互換性検証（FULL_TRANSITIVE / BACKWARD）
- gRPC / Connect-RPC: Transport Adapter Layer の実装基盤

## 期待結果 / 観測指標

- Apicurio Schema Registry の compatibility check が green（FULL_TRANSITIVE）
- Buf breaking change 検出が意図した変更のみを検出している（unexpected 0 件）
- 4 言語（Rust / C# / Go / TypeScript）のコード生成が全て成功
- `apicurio_gitops_sot.lock.yaml` にスキーマバージョンが記録されている
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- tier2 / tier3 の既存 import が破壊されていない（Testcontainers 結合テスト green）

## 失敗時の挙動 / escalation

- **Apicurio compatibility check fail**: merge 阻止。変更を revert するか、互換ラッパー追加・非互換バージョン bump のいずれかの方針を選択して再 draft する。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に方針決定）。Backstage runbook `schema-compat-failure` を参照。
- **4 言語コード生成 fail**: merge 阻止。生成エラーのある言語の proto 定義を修正する。特定言語の proto プラグインバージョン不整合の場合は Buf lockfile を更新する。escalate 先: tier1 担当者 dual reviewer（SLA: 8 時間以内に修正）。
- **tier2 / tier3 の結合テスト fail**: 変更が [Transport Adapter Layer](../../../03_概要設計/02_tier1設計方針/01_Server系.md) で吸収できていない可能性がある。adapter 実装を修正して再確認する。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に adapter 修正）。
- **Buf breaking change 検出（意図外）**: PR の変更内容を精査し意図しない breaking change を revert する。escalate 先: tier1 担当者 dual reviewer（SLA: 4 時間以内に revert）。

## 失敗パターン (anti-pattern)

- **型変更を「互換あり」と誤判定して進める**: oneof 変更や field number 変更が downstream の stub を静かに壊し、本番障害で気付く。buf breaking CI を必須 gate とし、意図外の breaking を 0 件で維持する。
- **Apicurio compatibility check をスキップして緊急 push**: 後方互換性が保証されないまま consumer が新スキーマを受信し始め、全 consumer での対応が必要になる。緊急時も compatibility check を経由する運用を徹底する。
- **4 言語の一部だけでコード生成確認**: Rust は green でも C# の生成が fail していて後から発覚する。全言語 CI を一括 gate として並列実行し、部分確認を構造的に不可能にする。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [新規 OSS 採用評価](01_新規OSS採用評価.md)
