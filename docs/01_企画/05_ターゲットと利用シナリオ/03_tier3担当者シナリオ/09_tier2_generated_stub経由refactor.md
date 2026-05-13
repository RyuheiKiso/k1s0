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
  defense_in_depth_layers: [A, B, C]
  proof_classes: []
---

# tier2 generated stub 経由 refactor

## 一文方針

13 層強制機構 lint が検出した禁止 import（tier1 Library / OSS 直接 / 業務管理 API / 独自型定義）を tier2 generated stub 経由に置き換えることで、tier3 の責務境界を回復する。

> 朝 9 時、自宅リモートの tier3 担当者（ジュニア級）が Jest CI の結果で「13 層強制機構 lint fail: 禁止 import `kafka-node` を検出」というエラーに気付く。手元には tier2 generated stub の TypeScript 定義と lint レポート、Mattermost 越しに tier2 担当者（新 API 追加依頼の受け手）と dual reviewer がいる。

## ペルソナ要約

主役: tier3 担当者（ジュニア級）、目的: tier2 直接呼び出しを generated stub 経由に refactor して境界違反を解消する

## 現状業務での痛み

- tier2 API を tier3 から直接呼び出して responsibility boundary を侵犯し、tier2 変更のたびに tier3 が壊れる
- 直接呼び出しにより contract test が存在せず、API の非互換変更が本番デプロイ後に発覚する
- 境界違反コードが散在しており、refactor の影響範囲が把握できない

## k1s0 でこう変わる

- generated stub 経由に統一することで tier2 の実装変更が tier3 に影響しなくなる
- stub を通じた contract test が CI で強制され、API 非互換変更が merge 前に検知される
- 境界違反コードが CI の boundary check で定量検出され、refactor 完了を数値で確認できる

## Trigger（発火条件）

既存 tier3 実装が tier1 Library または独自 type を直接 import していることが 13 層強制機構 lint で検出された時。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（lint fail 時）。典型きっかけ: 「tier3 担当者が Kafka client を直接 import し強制機構 lint が fail、tier2 generated stub 経由に refactor が必要になった」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier2 担当者（新 API 追加依頼の受け手）/ dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier3）| ジュニア | 本社 IT 室 / リモート | Jest CI / GitHub PR | lint 違反特定 / stub 経由に置換 / 独自型定義削除 / contract test / E2E 確認 |
| 関与（tier2）| 中堅 | 本社 IT 室 | Backstage Catalog | stub に等価 API がない場合の新 API 追加 / stub 最新化 / sign-off |
| 承認（dual reviewer）| 中堅〜シニア | 本社 IT 室 / リモート | GitHub PR | lint 違反 0 件 / contract test pass / E2E 退行なし / sign-off |

## 個人 KPI / 達成感

- tier2 直接呼び出し 0 件を boundary check CI で確認でき、refactor 完了の達成感を定量的に得られる
- contract test 全 pass で境界が正しく定義されていることを確認できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜3 日（境界違反箇所の特定 2h + stub 切り替え 4〜12h + contract test 追加 4h）
- 関与人数: 2〜3 名（tier3・tier2・dual reviewer）
- コスト感: 中〜高。違反箇所が多い場合は工数が増加するが boundary check CI が進捗を可視化する

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier3 担当者 | boundary check CI で tier2 直接呼び出し一覧を抽出 | `境界違反 N 件検出 / refactor 計画作成` |
| 4h | tier3 担当者 | 直接呼び出しを generated stub 経由に順次切り替え | `stub 切り替え進行中 / 残 N 件` |
| 1d | tier3 担当者 | 全違反を解消し boundary check + contract test green を確認して PR 提出 | `boundary check 0 violations / PR #NNN 提出` |
| 1d+4h | dual reviewer | boundary check green + contract test を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

- **FA 生産指示・設備操作**: 禁止 import を排除して tier3 の責務境界を回復することで、FA 生産指示画面が tier2 の認可制御を正しく受けるようになり、設備操作の安全性が担保される
- **受注**: stub 経由 refactor により受注 API の contract が明確化され、受注フロー全体の型安全性と契約的整合が確立される
- **在庫**: 在庫照会 API の直接 import を stub 経由に置き換えることで、在庫データの読み取りが tier2 の認可スコープ内に収まる

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

## 失敗パターン (anti-pattern)

- stub を迂回した直接 fetch: axios 等で tier2 URL を直接呼ぶと boundary check CI が merge 阻止する
- stub を薄い wrapper にして型なし: TypeScript の型を any にすると contract test が型不整合を検知する

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と担当者プロフィール
- [01_新業務画面追加.md](./01_新業務画面追加.md) — refactor 完了後の新規画面実装で踏むべき基本フロー
- [02_業務エラーUX_1対1分岐.md](./02_業務エラーUX_1対1分岐.md) — stub 経由 refactor 後に BusinessConflict subtype の UI 分岐が必要な場合の実装手順
- [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) — 禁止 import を stub 経由に置換した後の状態管理整合確認の SoT
- [13 層強制機構](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md) — 禁止 import lint の定義・適用範囲と refactor トリガーの仕様一覧
