---
id: plan.tier1.scenario_proto_diff_pr_review
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B]
  proof_classes: []
---

# External / Internal Proto 二層 diff PR レビュー

## 一文方針

tier1 担当者が CI の `buf breaking` / API snapshot 違反検出を受けて、External Proto（公開 API）と Internal Proto（内部ストリーム用）の二層差分を審査し、breaking change の有無・後方互換性・External/Internal 境界の逸脱を PR 上でレビューして承認 or 差し戻しを完結させる。

> 朝 10 時、tier1 担当者（シニア級）が GitHub PR ダッシュボードを開くと、`buf-breaking-lint` CI ジョブが fail した tier2 担当者の Domain Event スキーマ更新 PR が赤く点灯している。手元は WSL2 terminal + VS Code、Mattermost では tier2 担当者がレビュー依頼コメントを送っている。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: buf breaking 検出を起点に External/Internal 二層判定と breaking 有無を PR 上でレビューして承認 or 差し戻しを完結させる

## 現状業務での痛み

- buf breaking 検出後の External/Internal 二層判定を人手で毎回実施しており、判断がレビュアーごとにブレる
- field 番号変更や型変更が「backward 互換あり」と誤判定されて merge され、downstream の stub が壊れる
- tier1 担当者 2 名がいずれも利用不能な on-call 不在時にレビューが詰まり、tier2 の作業がブロックされる
- External Proto が Internal 実装詳細を露出していても気付かれずに merge されるケースがある

## k1s0 でこう変わる

- External/Internal 二層判定の decision tree を tier1 README に明文化し、dual reviewer が同一基準で物理 apply する
- `buf breaking` CI gate が fail した PR は dual reviewer 2 名の approve なしに merge 不可となる
- `api_snapshot.lock.yaml` との diff 評価で breaking reason を PR comment に明記する手順が標準化される
- dual reviewer 不在時の代替 reviewer 依頼フロー（Mattermost `#tier1-oncall`）が明文化されている

## Trigger（発火条件）

tier2 / tier3 担当者が Domain Event または API の `.proto` ファイルを変更した PR を提出し、CI の `buf breaking` チェックまたは API_snapshot 違反チェックが fail または warning を返した時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 週次〜月次（tier2 が Domain Event schema を追加・変更するたびに発火）
- 典型きっかけ: 「tier2 担当者が受注 Domain Event に `shipment_ref` フィールドを追加したが、既存の Internal Proto stream の field number が重複し `buf breaking` が fail した」「External Proto の `OrderCreated` message を変更した PR で API snapshot 違反が検出され、downstream の tier3 / Companion への breaking impact を評価する必要が生じた」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級、proto 二層化 + buf 運用のオーナー）
- 関与: tier2 担当者（変更 PR の author）
- 関与: tier3 担当者（External Proto 変更で影響を受ける downstream の確認）
- 承認: dual reviewer（tier1 担当者 2 名、PR author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 / リモート | GitHub PR + `buf-breaking` CI ジョブ | breaking 判定 / field 番号修正指示 / sign-off |
| 関与（tier2）| 中堅 | 本社 IT 室 | Backstage Software Catalog | proto 差分の意図説明 / 修正 PR 提出 |
| 関与（tier3）| ジュニア | 本社 / 工場 IT 室 | Backstage Docs | 変更が自分の stub に影響するか確認 |

## 個人 KPI / 達成感

- `buf-breaking` CI all pass 維持（unexpected breaking change 0 件）
- `buf-generate` CI 4 言語 stub 生成 green 率
- dual reviewer 応答時間 ≤ 24h（on-call 不在時含む）
- breaking change 誤 approve 0 件

## 工数 / 関与人数 / コスト感

- 初回（non-breaking と判定 → approve）: 1〜2h、関与 3〜4 名（主役 + tier2 担当者 + dual reviewer 2 名）
- 平常（breaking と判定 → 修正方針指示）: 2〜4h、関与 4〜5 名（+ tier3 担当者 1 名）
- 失敗時（major version bump 必要・移行計画作成）: +1〜2 日、関与 5〜6 名

## 前提

- `buf.yaml` と `buf.gen.yaml` が tier1 レポジトリに存在し、CI で `buf breaking` が自動実行される
- External Proto と Internal Proto の二層が明確に分離されている（External = 外部公開 API / Internal = tier1-tier2 間の内部ストリーム）
- API snapshot（`api_snapshot.lock.yaml`）が最新の External Proto の状態を記録している
- tier2 generated stub が TypeScript / C# / Go / Rust の 4 言語で Buf 管理下で自動生成されている

## 流れ

1. CI fail を確認する: `buf breaking` が fail している場合は report 内容（削除フィールド / 型変更 / field 番号変更）を読み取る
2. 二層判定を行う:
   - **External Proto 変更**: downstream（tier3 / Companion / 外部パートナー）への breaking impact が最大。外部 API 変更は `v2` namespace 追加 / deprecation 宣言が原則
   - **Internal Proto 変更**: tier1-tier2 間のみ影響。field 番号重複 / 型変更は reject。新フィールドの optional 追加は許容
3. 後方互換性の評価: 削除・型変更・field 番号変更は breaking。新 optional フィールド追加は non-breaking
4. External/Internal 境界逸脱の確認: Internal Proto の定義が External Proto に漏れていないか / External Proto が Internal の実装詳細を露出していないか
5. `api_snapshot.lock.yaml` との diff 評価: External Proto 変更が snapshot 違反の場合、breaking reason を PR comment に明記し差し戻す（breaking が意図的なら major version bump と snapshot 更新が必要）
6. 承認 or 差し戻し: non-breaking と判定した場合は tier1 担当者 2 名の dual reviewer sign-off で approve。breaking の場合は修正方針（field 番号の保持 / optional 化 / v2 namespace 追加）を PR comment で指示する
7. approve 後: `buf generate` の CI が自動実行され、4 言語 stub が生成されることを確認する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | `buf-breaking` fail 確認・二層判定（External/Internal）| `External Proto 変更 / 削除フィールド検出 → breaking と判定` |
| 1h | tier1 担当者 | 後方互換性評価・External/Internal 境界逸脱確認 | `backward 非互換 / v2 namespace 追加方針 / tier2 担当者に方針通知` |
| 2〜3h | tier1 担当者 | `api_snapshot.lock.yaml` diff 評価・PR comment で修正方針明記 | `breaking reason: field 削除 / 修正: v2 namespace 追加 or reserved 宣言` |
| 1 日 | dual reviewer A/B | approve（non-breaking）or 差し戻し確認 | `dual sign-off 完了 / buf-generate: 4 言語 stub 生成 green` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（External Proto が API の公開契約、Internal Proto が tier1-tier2 間のストリーム仕様のため）。特に影響度が高い 2 業務:

- **受注管理**: `OrderCreated` / `OrderUpdated` Domain Event の proto schema 変更が直接 downstream に波及する。breaking change が混入すると受注 sub が停止する
- **警報配信**: 警報 Domain Event の field 変更（例: severity フィールドの型変更）は tier3 の警報 UI および Companion 経由の外部通知に即時 breaking impact を与える

## 関連適合仕様 / 関連 OSS

- 19 検証規律適合仕様: [19_検証規律適合仕様.md](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md)
- 関連 OSS: Buf（proto lint / breaking check）/ TypeScript / C# / Go / Rust（4 言語 stub 生成）/ Apicurio Registry

## 期待結果 / 観測指標

- ci: `buf-breaking` CI ジョブが all pass（GitHub PR checks 画面）
- ci: `buf-generate` CI ジョブが 4 言語 stub 生成完了（green）
- artifact: breaking change が意図的な場合は `api_snapshot.lock.yaml` が更新済み
- sign-off: dual reviewer（tier1 担当者 2 名）approve 完了

## 失敗時の挙動 / escalation

- **breaking change が意図的で major version bump が必要**: tier2 担当者と調整し、External Proto の `v2` namespace 追加または `api_snapshot.lock.yaml` の update PR を別途作成（**SLA: 48h 以内**）。downstream の tier3 / Companion への移行計画を tier2 担当者が作成する
- **4 言語 stub 生成が一部 fail**: 生成失敗言語のコンパイルエラーを確認し、proto 定義の型互換性問題を tier2 担当者にフィードバック（**SLA: 24h 以内**）
- **tier1 担当者 2 名がいずれも利用不能（on-call 不在）**: アーキテクト / tier1 リードに Mattermost `#tier1-oncall` で代替 reviewer 依頼（**SLA: 4h 以内**）。LLM 単独 sign-off は禁止

## 失敗パターン (anti-pattern)

- **新 optional フィールド追加を「breaking」と誤判定して差し戻す**: 不要な修正ラウンドが発生し tier2 担当者の作業が遅延する。backward 互換性の判定ルール（追加は non-breaking / 削除・型変更は breaking）を decision tree に明文化し、誤判定を排除する。
- **LLM 単独 sign-off を許可**: AI の判断に依存した approve が積み重なり、重大な breaking change が見落とされる。dual reviewer（tier1 担当者 2 名の人間）の approve を merge の物理前提条件とし、AI 単独 sign-off を構造的に禁止する。
- **breaking と判定した PR を「方針は後で決める」として長期放置**: PR が数週間 open のまま tier2 の作業がブロックされ続ける。方針決定 SLA（48h 以内）を明文化し、Backstage で自動 escalate を発火させる。

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成と担当者プロフィール
- [proto 進化 / schema 互換管理（tier1-03）](./03_proto二層化スキーマ進化.md) — proto 変更の expand-contract 手順と互換性保持の設計原則
- [API snapshot 違反対応（tier1-08）](./08_公開API_snapshot違反対応.md) — snapshot 違反を検出した際の version bump 手順
- [L1+ migration dry-run（tier1-12）](./12_L1plus_migration_dryrun.md) — proto 二層を含む L1+ OSS の年次移行検証
- [19 検証規律適合仕様](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md) — buf breaking check の検証規律と CI 組込み規則
