---
id: plan.tier3.scenario_state_transition_ui
axis: tier3
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C]
---

# 状態遷移 UI FSM 実装

## 一文方針

tier3 担当者が tier2 の状態遷移 FSM（protoc-gen-go FSM 方式 / 4 言語等価強度）と連携し、許可遷移のみ button を activate、禁止遷移は deactivate する状態遷移 UI を実装する。

## Trigger（発火条件）

業務フローに FSM（Finite State Machine）が必要な画面（受注ステータス / 設備稼働状態 / 検査フロー）の実装要求が来た時

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期。典型きっかけ: 「受注の状態遷移（受注確認中 → 確定 → 製造指示中 → 完了 / キャンセル）をユーザーが操作できる画面が必要になった。tier2 が宣言する FSM に従い、現在の状態から可能な遷移のみ button を有効化する UI を実装する」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier2 担当者（FSM の遷移定義確認）
- **承認**: dual reviewer（tier3 担当者 1 名 + tier2 担当者 1 名）

## 前提

- tier2 が [protoc-gen-go FSM](../../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)（4 言語等価強度）で状態遷移を宣言済み。
- tier2 API が「現在状態で可能な遷移のリスト」を返す endpoint を提供している。
- 認可 cache 禁止（13 層強制機構 #4）。
- （責務境界: [README 重要原則](./README.md#重要原則-tier3-が所有するもの--所有しないもの) を参照）

## 流れ

1. tier2 generated stub で「可能な遷移リスト」を返す API エンドポイントを確認する（例: `GET /v1/orders/{id}/state-transitions`）
2. 状態遷移 button コンポーネントを実装する:
   - tier2 API から現在状態と可能遷移リストを取得（**認可 cache 禁止: 毎回 API から取得**）
   - 可能遷移 = button `active`（クリック可能）/ 不可遷移 = button `disabled`（`aria-disabled="true"` + CSS グレーアウト）
3. 遷移実行: button クリック → 確認ダイアログ（「受注を確定しますか?」等）→ tier2 の状態遷移 API を呼び出す
4. 状態遷移後の画面更新: 遷移後の新状態と可能遷移リストを再取得して UI を更新（楽観的更新は禁止、tier2 の確定後に反映）
5. エラー処理: 遷移先状態が既に変わっていた場合（競合）は BusinessConflict UX（[02_業務エラーUX_1対1分岐.md](./02_業務エラーUX_1対1分岐.md)）を表示
6. a11y: 現在状態と可能遷移を `aria-label` で screen reader に通知する
7. Playwright E2E test: 各状態から可能な遷移を実行し、不可能な遷移が disabled になっていることを確認
8. dual reviewer sign-off を取得する

## 関連適合仕様 / 関連 OSS

- クライアント状態適合仕様: [../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)
- tier3 強制機構: [../../../04_詳細設計/02_強制機構/03_tier3強制機構.md](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
- protoc-gen-go FSM: [../../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md](../../../04_詳細設計/03_クロスカッティング適合仕様/04_protoc_gen_go_fsm.md)
- 関連 OSS: Playwright（E2E）/ axe-core（a11y）/ tier2 generated TypeScript stub

## 期待結果 / 観測指標

- ci: Playwright E2E test（各状態 × 可能遷移 / 不可遷移の網羅テスト）all green
- ci: axe-core a11y test all green
- ux: 認可 cache を持たず、毎 API call で最新の可能遷移リストを取得（強制機構 #4 準拠）
- sign-off: dual reviewer（tier3 + tier2）sign-off 完了

## 失敗時の挙動 / escalation

- **認可 cache を持つ実装が 13 層強制機構 lint で検出**: merge 阻止。tier2 担当者と実装方針を再確認（**SLA: 24h 以内**）。
- **状態遷移 API が競合エラーを返す（409）**: BusinessConflict UI（concurrent_edit / stale_write）を表示して解決を促す。

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と dual reviewer 規約
- [業務エラー UX 1対1 分岐](./02_業務エラーUX_1対1分岐.md) — 状態遷移で競合が発生した場合の BusinessConflict UX
- [tier3 設計方針 状態遷移 UI](../../../03_概要設計/04_tier3設計方針/22_状態遷移UI.md) — 許可遷移 activate / 4 言語等価強度の設計指針
