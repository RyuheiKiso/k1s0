---
id: plan.tier3.scenario_npm_vuln_response
axis: tier3
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# npm vuln 対応

## 一文方針

tier3 担当者が npm audit / GitHub Dependabot が検出した脆弱性を評価し、依存 library を安全に upgrade して 13 層強制機構の supply chain 要件を維持する。

> 午前 9 時、本社 IT 室の tier3 担当者（ジュニア級）が Mattermost `#tier3-dev` で「GitHub Dependabot: vite に CVSS 8.7 の CVE が公開」という通知に気付き、リリース前日の緊張感の中で影響範囲確認を開始する。手元には npm audit レポートと Harbor internal registry のコンソール、Mattermost 越しに tier1 担当者（Harbor mirror 更新）と dual reviewer がいる。

## Trigger（発火条件）

GitHub Dependabot のアラートが発生した時 / npm audit で CVSS 7.0 以上の脆弱性が検出された時 / 週次の依存チェック cadence 到来時

## 想定頻度 / 典型きっかけ

想定頻度: 週次〜月次（Dependabot alert は随時）。典型きっかけ: 「`vite` に CVE-2024-45812 が公開され CVSS 8.7、影響範囲が tier3 の SPA 全体になることを確認した」「月次の npm audit で `axe-core` の patch 更新が必要になった」

## 主役 / 関与者

- **主役**: tier3 担当者（ジュニア級）
- **関与**: tier1 担当者（Harbor mirror 更新が必要な場合）
- **承認**: dual reviewer（tier3 担当者 1 名 + tier2 担当者 1 名）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier3）| ジュニア | 本社 IT 室 / リモート | Mattermost `#tier3-dev` / GitHub PR | CVSS 評価 / Harbor mirror 確認 / npm update / E2E 再実行 / PR 提出 |
| 関与（tier1）| シニア | 本社 IT 室 | Backstage Catalog | Harbor mirror に upgrade 対象バージョンを追加 / supply chain 確認 |
| 承認（dual reviewer）| 中堅〜シニア | 本社 IT 室 / リモート | GitHub PR | 全 E2E + smoke + a11y green / Web Vitals 劣化なし / sign-off |

## 前提

- 依存 package は Harbor mirror の internal npm registry からのみ pull（13 層強制機構 #12: 内部レジストリ唯一化）
- 公開 npm registry から直接 pull する経路は Kyverno / lint で禁止
- （責務境界: [README 重要原則](./README.md#重要原則-tier3-が所有するもの--所有しないもの) を参照）

## 流れ

1. Dependabot アラートまたは `npm audit` の結果を確認し、脆弱性の CVSS スコアと影響範囲を評価する
   - CVSS 7.0 未満: 週次 batch として対応
   - CVSS 7.0-8.9: 48h 以内に対応 PR を作成
   - CVSS 9.0 以上: 即日対応（tier1 に supply chain 障害対応を依頼することを検討）
   - CVSS 9.0 以上: tier1 担当者に Mattermost `#tier1-harbor` 経由で tier1-09（supply_chain 障害対応）の起動を依頼
   - CVSS 7.0〜8.9: tier1-10（SBOM CVE 月次トリアージ）に本件を合流させる形で連絡し、tier1 との共同対応とする
   - CVSS 7.0 未満: 本シナリオで自律的に対応
2. tier2 generated stub の依存でないことを確認する（stub は tier2 が管理するため tier3 から直接変更しない）
3. Harbor mirror の internal npm registry に upgrade 対象バージョンが存在するか確認する
   - 存在しない場合: tier1 担当者に Mattermost `#tier1-harbor` で Harbor mirror 更新を依頼してから upgrade
4. `npm update <package>@<safe-version>` を実行し `package.json` / `package-lock.json` を更新する
5. 全ての Playwright E2E test + smoke test + axe-core a11y test を local で実行し green を確認する
6. Web Vitals（LCP ≤ 2.5s / INP ≤ 200ms / CLS ≤ 0.1）が劣化していないことを確認する
7. CI での green を確認し PR を作成する
8. dual reviewer sign-off を取得する

## 業界 9 業務との紐付け

- **FA 生産指示・設備操作**: SPA（vite ビルド）の脆弱性が生産指示画面の XSS リスクに直結するため、supply chain の安全性が FA 操作の信頼性を担う
- **受注**: 受注 SPA の依存 library の脆弱性を放置すると受注データの改ざんリスクが生じるため、npm vuln 対応が受注業務の安全性を底支えする
- **ライン稼働監視・進捗実績**: 監視画面を提供する SPA の supply chain 整合性が、リアルタイム表示の信頼性の前提となる

## 関連適合仕様 / 関連 OSS

- 検証規律適合仕様: [../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md)
- tier3 強制機構: [../../../04_詳細設計/02_強制機構/03_tier3強制機構.md](../../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
- 関連 OSS: npm audit / GitHub Dependabot / Harbor（internal npm registry）/ Playwright（regression test）

## 期待結果 / 観測指標

- ci: `npm install` が Harbor mirror の internal npm registry から全 package を pull（`npm audit --registry <harbor-url>` 0 件の新規 CVSS 7.0 以上）
- ci: CI job `tier3-e2e-tests` + `tier3-smoke-tests` all green
- web-vitals: LCP ≤ 2.5s / INP ≤ 200ms / CLS ≤ 0.1（Lighthouse CI で計測）
- sign-off: dual reviewer（tier3 担当者 1 名 + tier2 担当者 1 名）sign-off 完了

## 失敗時の挙動 / escalation

- **upgrade 後に E2E test / smoke test が fail**: 原因を特定し修正。CVSS 9.0 以上で 24h 以内に修正不可の場合 → tier1 担当者に Mattermost `#supply-chain-incident` で即時エスカレーション（**SLA: 1h 以内**）。Backstage runbook `critical-npm-vuln` を参照。
- **Harbor mirror に upgrade 対象バージョンが存在しない**: tier1 担当者に Mattermost `#tier1-harbor` で Harbor mirror 更新を依頼（**SLA: tier1 担当者が 4h 以内に mirror 更新**）。それまでは脆弱性バージョンのまま運用継続（CVSS 9.0 以上の場合は機能凍結も検討）。
- **upgrade で Web Vitals が劣化**: tier2 担当者に影響のある API との通信コストを確認。劣化許容外の場合は upgrade を見送り、alternative package の調査を実施（**SLA: 5 営業日以内**に代替案提示）。

## 関連参照

- [tier3 担当者シナリオ index](./README.md) — tier3 担当者シナリオ全体の構成と dual reviewer 規約
- [tier2 generated stub 経由 refactor](./09_tier2_generated_stub経由refactor.md) — tier2 generated stub 自体の依存更新が必要な場合のシナリオ
- [pre-release smoke test](./10_pre_release_smoke_test.md) — upgrade 後の regression を smoke test で検出するシナリオ
- [tier3 設計方針](../../../03_概要設計/04_tier3設計方針/README.md) — 13 層強制機構 #12（内部レジストリ唯一化）の設計指針
