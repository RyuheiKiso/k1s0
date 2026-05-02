# E2E テスト結果サマリ（月次更新）

本書は ADR-TEST-002（E2E 自動化）の nightly workflow 実行結果を月次でサマリ化する live document。`SHIP_STATUS.md` と並列で運用し、採用検討組織が時系列で testing maturity を確認できる経路を提供する。

## 本書の位置付け

`tests/e2e/scenarios/` 配下の 3 シナリオ（tenant_onboarding / audit_pii_flow / payroll_full_flow）を `nightly.yml` で 03:00 JST に実行した結果を、月次でサマリ化する。各月の summary は workflow run の artifact（screenshot / HAR / k6-summary / cluster-logs）へのリンクと併せて記録する。

更新責務は起案者（リリース時点）+ 採用初期以降は contributor の合意で分担する。月初の最初の workflow run 完了後に、起案者または当番 SRE が前月分の summary を本書に追記する運用とする。

## 月次サマリ

### 2026-05（リリース時点 / 初月、E2E 実走前）

- **状態**: skeleton 配置完了、nightly workflow 起動条件未充足（OSS 公開後の contributor 1 名以上の参画が起動 trigger）
- **完了済**: tests/e2e/scenarios/ 3 シナリオ実装 / `_reusable-e2e.yml` / `nightly.yml` 配置 / `Makefile verify-e2e` target 整備
- **次月**: OSS 公開後、初回 nightly run の completion を待って 2026-06 entry を追記

## 月次サマリ template（採用初期で本テンプレに従って追記）

```markdown
### YYYY-MM

- **対象期間**: YYYY-MM-01 〜 YYYY-MM-末日
- **nightly 実行回数**: N 回（うち success M 回 / failure K 回）
- **fail 率**: X.X%（5% 超で警告）
- **代表 failure**: <run URL> — <概要>
- **修正対応**: <commit hash> <概要>
- **flaky 候補**: <test 名>（quarantine 状態）
```

## 関連

- ADR-TEST-002（E2E 自動化）
- `.github/workflows/nightly.yml` / `_reusable-e2e.yml`
- `tools/qualify/flaky-detector.py`
