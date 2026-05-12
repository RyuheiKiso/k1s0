---
id: plan.tier3.scenario_index
axis: tier3
phase: plan
kind: index
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier3 担当者シナリオ index

## 一文方針

tier3 担当者（ジュニア級）が日常的に踏む 15 シナリオを 1 ファイル 1 シナリオで列挙する。個別業務 UI の開発に集中し、業務不変条件は 13 層強制機構が保護するため tier3 担当者は UI 層の業務シナリオ表現のみを責務とする。

## 担当者プロフィール

- 級: ジュニア
- 想定人数: 業界 pack × 業務領域 × tenant 数で増減
- 必須スキル: TypeScript + React / C# WPF / .NET Framework + WinForms / Tauri

詳細は [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md) を参照。

## dual reviewer 規約
- tier3 シナリオにおける dual reviewer = **tier3 担当者 1 名 + tier2 担当者 1 名**（tier2 担当者が business logic の整合を確認）
- ジュニア級 tier3 担当者の単独 sign-off は禁止。必ず tier2 担当者の確認を経る。
- 詳細: [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)

## 本シナリオ群での追加役職定義
- **アーキテクト**: 業界 / 業務領域横断の設計をレビューするロール。tier1 または tier2 のシニア担当者が担当。Mattermost `#tier3-arch-review` で相談。
- **i18n 担当者**: ローカライゼーション / 翻訳品質を担当するロール。Mattermost `#i18n-review` で翻訳依頼。
- **デザインシステム担当者**: design token / a11y ガイドライン / コンポーネントライブラリを管理するロール。Mattermost `#design-review` で確認。

## 重要原則: tier3 が所有するもの / 所有しないもの

| 所有する | 所有しない |
|---|---|
| 画面遷移シナリオ | 業務不変条件 / 整合性境界 |
| フォームレイアウト | 業務資産（Domain Event / Workflow / 決定表）|
| 業務エラーの UI 表現 | infra への直接アクセス |
| 帳票レイアウト | tier1 Library / Companion の直接使用 |
| アクセシビリティ補助 | 業界横断概念の独自定義 |

→ **業務ロジックは tier2 が持つ。tier3 は「同じ業務処理を操作シナリオで表現する場」**

## シナリオ一覧

| # | シナリオ名 | trigger 概要 | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|-------------|---------|-----------------|------|
| 01 | [新業務画面追加](./01_新業務画面追加.md) | 業務担当者要求または tier2 新 API を受けて新業務画面を追加する | 月次〜週次 | [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [計画] |
| 02 | [業務エラー UX 1 対 1 分岐](./02_業務エラーUX_1対1分岐.md) | tier2 が BusinessConflict subtype を追加した通知を受けて UI 分岐を実装する | 四半期（新 subtype 追加時）| [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [計画] |
| 03 | [リアルタイム更新 UX](./03_リアルタイム更新UX.md) | ラインライブ監視・警報配信などリアルタイム更新 UX を要求する画面を実装する | 月次 | [Bidi 適合仕様](../../../04_詳細設計/01_適合仕様/01_Bidi適合仕様.md) | [計画] |
| 04 | [オフライン業務記録](./04_オフライン業務記録.md) | 工場現場など 25h オフライン業務に対応する記録機能を実装する | 四半期〜年次 | [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [計画] |
| 05 | [レガシー .NET Framework Companion 統合](./05_レガシー_NetFx_Companion統合.md) | .NET Framework 4.8 ERP 等のレガシーシステムを k1s0 に統合する | 不定期（新レガシーシステム統合時）| [Tauri companion sidecar](../../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md) | [計画] |
| 06 | [帳票添付 UX](./06_帳票添付UX.md) | 検査票・図面・受注書などの upload / download 帳票 UX を実装する | 月次 | [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [計画] |
| 07 | [WebAuthn step_up + IdP federation](./07_WebAuthn_step_up_federation.md) | high-risk 操作の WebAuthn step_up または親会社 IdP federation を実装する | 四半期〜年次 | [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) | [計画] |
| 08 | [a11y / i18n 修正](./08_a11y_i18n修正.md) | CI の axe-core fail または i18n 翻訳キー未定義を検出して修正する | イベント駆動（CI fail 時）| [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [緊急] |
| 09 | [tier2 generated stub 経由 refactor](./09_tier2_generated_stub経由refactor.md) | 禁止 import を 13 層強制機構 lint で検出し tier2 generated stub 経由にリファクタする | イベント駆動（lint fail 時）| [クライアント状態適合仕様](../../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md) | [緊急] |
| 10 | [pre_release_smoke_test](10_pre_release_smoke_test.md) | 新業務画面追加完了時またはリリース milestone 到来時 | 月次 | クライアント状態適合仕様 | [計画] |
| 11 | [npm_vuln_対応](11_npm_vuln_対応.md) | Dependabot アラートまたは npm audit で CVE 検出時 | 週次〜月次 | 検証規律適合仕様 | [緊急]+[周期] |
| 15 | [Backstage 業務管理 UI マスタ更新](15_業務管理UI_マスタ更新.md) | 業務管理者のマスタ更新で validation エラー / 新規マスタ種別追加依頼があった時 | 週次〜月次 | 13 層強制機構 / クライアント状態適合仕様 | [計画] |

## 13 層強制機構（tier3 を守るガードレール）

> **注意**: 「13 層」という数字は **tier3 軸固有の enforcement 層数**（lint / runtime / contract test 等 13 種類）を指します。infra/data の「13 cross-cutting 適合仕様」（全 19 軸横断の適合仕様ファイル数）とは別概念です。混同しないよう注意してください。

tier3 担当者（ジュニア級）が業務不変条件を踏み抜かないよう、以下 13 層が物理的に保護します。

1. tier1 / OSS クライアントの直接 import 禁止（lint）
2. 業務管理 API の業務 UI への import 禁止（lint）
3. tenant_id を form / URL で受け付けることを禁止（lint）
4. 認可結果の client-side cache 禁止（runtime）
5. localStorage / sessionStorage / cookie への PII 平文保管禁止（lint）
6. 監査 emit を抑制する経路の禁止（lint）
7. 翻訳キー未定義の検出（CI）
8. 公開 type の独自生成禁止（contract test）
9. contract test の必須実行（CI merge 条件）
10. a11y 自動検査（axe-core / CI merge 条件）
11. CSP / セキュリティヘッダ宣言（CI merge 条件）
12. 内部レジストリ唯一化（Harbor mirror のみから pull）
13. サプライチェーン署名（cosign）

詳細: [tier3 強制機構](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md)

## 新規参画者向けオンボーディング（ジュニア級）

**難易度別推奨順序（入門 → 上級の順に着手）**:

- **入門（Day 1-5）**: 01（新業務画面追加）→ 08（a11y/i18n 修正）  
  → まず「画面を 1 つ作ってみる」ことと「lint fail を直す」ことから始める
- **基礎（Week 2）**: 06（帳票添付）→ 09（stub 経由 refactor）→ 10（smoke test）  
  → tier2 との連携パターンと強制機構を身体で覚える
- **応用（Week 3-4）**: 02（業務エラー UX）→ 03（リアルタイム）→ 11（npm vuln 対応）
- **上級（ペアプロ必須）**: 04（オフライン）→ 05（レガシー統合）→ 07（WebAuthn）  
  → 単独着手しないこと。必ず tier2 担当者 or シニアがペアについた状態で行う

## シナリオ間の依存関係

- **01（新業務画面）→ 10（smoke test）**: 新画面追加後に 10 で smoke test を追加する。実施順序: 01 → 10 → release。
- **02（業務エラー UX）の先行条件**: tier2-07（BusinessConflict subtype 追加）が完了し subtype の contract が宣言済みであること。
- **09（stub refactor）→ 01（新画面）**: lint fail 検出後に refactor（09）を先行し、その後新機能追加（01）を進める。
- **11（npm vuln）の判断ツリー**: CVSS 9.0 以上 → tier1-09 supply_chain 障害対応を起動依頼 / 7.0〜8.9 → tier1-10 月次トリアージに合流 / 7.0 未満 → 11 で自律対応。

## 関連参照

- tier3 設計方針: [../../../03_概要設計/04_tier3設計方針/README.md](../../../03_概要設計/04_tier3設計方針/README.md)
- ターゲットと利用シナリオ index: [../README.md](../README.md)
