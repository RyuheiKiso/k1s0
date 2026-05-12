---
id: plan.tier3.scenario_tier2_stub_refactor
axis: tier3
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier2 generated stub 経由 refactor

## 一文方針

13 層強制機構 lint が検出した禁止 import（tier1 Library / OSS 直接 / 業務管理 API / 独自型定義）を tier2 generated stub 経由に置き換えることで、tier3 の責務境界を回復する。

## Trigger（発火条件）

既存 tier3 実装が tier1 Library または独自 type を直接 import していることが 13 層強制機構 lint で検出された時。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（lint fail 時）。典型きっかけ: 「tier3 担当者が Kafka client を直接 import し強制機構 lint が fail、tier2 generated stub 経由に refactor が必要になった」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier2 担当者（新 API 追加依頼の受け手）/ dual reviewer

## 前提

- [13 層強制機構](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md)の lint が CI に組込済み
- tier2 generated stub（TypeScript）が最新の状態でリポジトリに存在する
- （責務境界: [README 重要原則](./README.md#重要原則-tier3-が所有するもの--所有しないもの) を参照）

## 流れ

1. 違反 import 箇所を lint レポートから特定（tier1 Library / OSS / 業務管理 API / 独自型定義）
2. tier2 generated stub（TypeScript）で等価な API が提供されているか確認
3. stub が提供していない場合: tier2 担当者に新 API 追加を依頼（tier3 から直接 tier1 を呼ぶのは禁止）
4. 独自型定義: tier2 generated type に置換し型定義ファイルを削除
5. contract test: 置換後も tier2 API との contract が成立することを確認
6. 全 Playwright E2E test green を確認してから旧コードを削除
7. dual reviewer sign-off

## 関連適合仕様 / 関連 OSS

- クライアント状態適合仕様: [../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)
- 関連 OSS: Playwright / TypeScript / Pact（contract test）

## 期待結果 / 観測指標

- 13 層強制機構 lint: 違反 0 件
- contract test: 全 pass
- Playwright E2E: 全 green（機能退行なし）
- 独自型定義ファイルが削除されている
- dual reviewer 2 名 sign-off 完了

## 失敗時の挙動 / escalation

- 強制機構 lint fail → merge 阻止。**escalate 先**: tier2 担当者に Mattermost `#tier3-contract-fail` で stub 追加依頼（**SLA**: 24h 以内）。**runbook**: Backstage runbook `forbidden-import-refactor` を参照
- contract test fail → merge 阻止。**escalate 先**: tier2 担当者に Mattermost `#tier3-contract-fail` で API 仕様確認を依頼（**SLA**: 24h 以内）
- tier2 stub に等価 API がなく tier2 担当者が追加を拒否 → tier2 / tier3 間の責務境界を設計レビューにエスカレーション（escalate）。**escalate 先**: アーキテクト / tier1 担当者に Mattermost `#tier3-arch-review` で相談（**SLA**: 48h 以内）
- contract test が白紙になっている場合（改変疑い）→ **escalate 先**: security / test 担当者に即時 escalate（**SLA**: 1h 以内）。**runbook**: Backstage runbook `contract-test-integrity-check` を参照

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と担当者プロフィール
- [01_新業務画面追加.md](./01_新業務画面追加.md) — refactor 完了後の新規画面実装で踏むべき基本フロー
- [02_業務エラーUX_1対1分岐.md](./02_業務エラーUX_1対1分岐.md) — stub 経由 refactor 後に BusinessConflict subtype の UI 分岐が必要な場合の実装手順
- [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) — 禁止 import を stub 経由に置換した後の状態管理整合確認の SoT
- [13 層強制機構](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md) — 禁止 import lint の定義・適用範囲と refactor トリガーの仕様一覧
