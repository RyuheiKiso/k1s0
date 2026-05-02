# E2E テスト結果サマリ（月次更新）

本書は ADR-TEST-002（E2E 自動化）の nightly workflow 実行結果を月次でサマリ化する live document。`SHIP_STATUS.md` と並列で運用し、採用検討組織が時系列で testing maturity を確認できる経路を提供する。

## 本書の位置付け

`tests/e2e/scenarios/` 配下の 3 シナリオ（tenant_onboarding / audit_pii_flow / payroll_full_flow）を `nightly.yml` で 03:00 JST に実行した結果を、月次でサマリ化する。各月の summary は workflow run の artifact（screenshot / HAR / k6-summary / cluster-logs）へのリンクと併せて記録する。

更新責務は起案者（リリース時点）+ 採用初期以降は contributor の合意で分担する。月初の最初の workflow run 完了後に、起案者または当番 SRE が前月分の summary を本書に追記する運用とする。

## 月次サマリ

### 2026-05（リリース時点 / 初月、初回 local 実走確認 — TestTenantOnboarding 完全 PASS）

- **状態**: kind cluster（k8s v1.31.4、3-worker HA）+ Dapr 1.17.5 + tier1-state（dev/CI mode）で **TestTenantOnboarding が完全 PASS**（2026-05-02 23:38 JST 実走）
- **PASS した検証（2 サブテスト）**:
  - `dapr-system_running`: dapr-system namespace で 4 Running Pod（operator / placement / sentry / sidecar-injector）
  - `tier1_state_save_get_delete`: State.Save (etag=v1) → State.Get (data="hello-from-e2e" 一致) → Delete → Get (found=false) の full cycle 成功
- **動作環境構築の経路（再現可能）**:
  1. `tools/local-stack/up.sh --role docs-writer` で kind cluster + Calico + cert-manager 起動
  2. `helm upgrade --install dapr dapr/dapr -n dapr-system --version 1.17.5` で Dapr 直接 install
  3. `docker build -f src/tier1/go/Dockerfile.state -t ghcr.io/k1s0/k1s0/tier1-state:latest .`
  4. `kind load docker-image ghcr.io/k1s0/k1s0/tier1-state:latest --name k1s0-local`
  5. `helm upgrade --install tier1-facade deploy/charts/tier1-facade --namespace tier1-state --create-namespace --set pods.secret.enabled=false --set pods.workflow.enabled=false --set image.pullPolicy=Never --set-string 'podAnnotations.dapr\.io/enabled=false'`
  6. `kubectl port-forward -n tier1-state svc/tier1-facade-state 50001:50001 &`
  7. `cd tests/e2e && K1S0_TIER1_TARGET=localhost:50001 go test -v ./scenarios/... -run TestTenantOnboarding`
- **発見した bug + 解決策**:
  - **Bug**: tier1-facade chart の default で daprd sidecar が起動するが `dapr-grpc-port=50001` が `app-port=50001` と衝突、Pod が CrashLoopBackOff
  - **解決**: `dev/CI mode` では tier1-state container が `DAPR_GRPC_ENDPOINT not set, using in-memory Dapr backend` で起動できるため、`podAnnotations.dapr.io/enabled=false` で sidecar を disable
  - **採用初期での課題**: 本番（`enabled=true`）で同 port 衝突を解消する chart 修正（dapr.io/grpc-port annotation で sidecar の listen port を 50101 等へ移すなど）
- **未実走**: TestAuditPiiFlow / TestPayrollFullFlow（Rust audit / pii Pod の image build + load + deploy が採用初期）。istio ambient mesh / mTLS 強制は SHIP_STATUS line 232 採用初期項目として残置
- **以降**: tier1-rust-service（audit / pii / decision）の image build 経路を確立し、TestAuditPiiFlow を PASS まで持っていく

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
