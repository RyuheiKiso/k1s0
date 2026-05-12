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

## 関連参照

- [tier1 担当者シナリオ index](./README.md) — tier1 担当者シナリオ全体の構成と担当者プロフィール
- [proto 進化 / schema 互換管理（tier1-03）](./03_proto二層化スキーマ進化.md) — proto 変更の expand-contract 手順と互換性保持の設計原則
- [API snapshot 違反対応（tier1-08）](./08_公開API_snapshot違反対応.md) — snapshot 違反を検出した際の version bump 手順
- [L1+ migration dry-run（tier1-12）](./12_L1plus_migration_dryrun.md) — proto 二層を含む L1+ OSS の年次移行検証
- [19 検証規律適合仕様](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md) — buf breaking check の検証規律と CI 組込み規則
