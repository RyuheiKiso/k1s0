# E2E テスト結果サマリ（月次更新）

本書は ADR-TEST-002（E2E 自動化）の nightly workflow 実行結果を月次でサマリ化する live document。`SHIP_STATUS.md` と並列で運用し、採用検討組織が時系列で testing maturity を確認できる経路を提供する。

## 本書の位置付け

`tests/e2e/scenarios/` 配下の 3 シナリオ（tenant_onboarding / audit_pii_flow / payroll_full_flow）を `nightly.yml` で 03:00 JST に実行した結果を、月次でサマリ化する。各月の summary は workflow run の artifact（screenshot / HAR / k6-summary / cluster-logs）へのリンクと併せて記録する。

更新責務は起案者（リリース時点）+ 採用初期以降は contributor の合意で分担する。月初の最初の workflow run 完了後に、起案者または当番 SRE が前月分の summary を本書に追記する運用とする。

## 月次サマリ

### 2026-05（リリース時点 / 初月、初回 local 実走確認）

- **状態**: kind cluster（k8s v1.31.4、3-worker HA）+ Dapr 1.17.5 layer install 済の環境で **TestTenantOnboarding/dapr-system_running が PASS**（dapr-system namespace で 4 Running Pod 確認、2026-05-02 23:23 JST 実走）
- **PASS した検証**:
  - `helpers.SetupCluster`: kind cluster の kubeconfig 経由接続
  - `dapr-system_running` サブテスト: dapr-operator / dapr-placement-server / dapr-sentry / dapr-sidecar-injector が Running
- **Skip された検証**: `tier1_state_save_get_delete`（K1S0_TIER1_TARGET 未指定、tier1-facade deploy が採用初期）
- **未実装層**: tier1-facade / tier1-rust-service の image build + kind load + helm install。Workflow.Start 等の他 service の gRPC 疎通拡張
- **完了済**: tests/e2e/scenarios/ 3 シナリオ実装 / `_reusable-e2e.yml` / `nightly.yml` 配置 / `Makefile verify-e2e` target 整備 / **kind cluster 上での初回実走で 1 件 PASS 取得**
- **失敗事象**: `tools/local-stack/up.sh --no-cluster --role tier1-go-dev` が istio ambient install で timeout（istio-cni-node / ztunnel が CrashLoopBackOff、SHIP_STATUS line 232「Istio Ambient mesh / mTLS 強制」未検証扱いと整合）。本実走では istio ambient install を skip し dapr layer を helm で直接 install して進めた。istio ambient の安定化は SHIP_STATUS の採用初期項目として残置し、本書 entry の前提条件に「istio skip」を明記する
- **以降**: tier1-facade image build + helm install 経路を採用初期で確立し、tier1_state_save_get_delete を PASS まで持っていく

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
