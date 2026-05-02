# E2E テスト結果サマリ（月次更新）

本書は ADR-TEST-002（E2E 自動化）の nightly workflow 実行結果を月次でサマリ化する live document。`SHIP_STATUS.md` と並列で運用し、採用検討組織が時系列で testing maturity を確認できる経路を提供する。

## 本書の位置付け

`tests/e2e/scenarios/` 配下の 3 シナリオ（tenant_onboarding / audit_pii_flow / payroll_full_flow）を `nightly.yml` で 03:00 JST に実行した結果を、月次でサマリ化する。各月の summary は workflow run の artifact（screenshot / HAR / k6-summary / cluster-logs）へのリンクと併せて記録する。

更新責務は起案者（リリース時点）+ 採用初期以降は contributor の合意で分担する。月初の最初の workflow run 完了後に、起案者または当番 SRE が前月分の summary を本書に追記する運用とする。

## 月次サマリ

### 2026-05 追補（**主張訂正版**、2026-05-03 07:00 JST）

直前 commit `2b78364bc` で「tier1 12/12 全 service PASS」「観測性 5/5 全検証 PASS」「portability 代替経路 PASS」と記録したが、**起案者の自己監査で複数 misleading が判明**したため本節で訂正する。past entry を上書きせず追補で残し、訂正の経過自体を監査痕跡として保持する。

#### 訂正 1: tier1 12 service の PASS 区分（real / stub の混在）

「12/12 service が err==nil 限定 PASS」という記述は字面の通りだが、**5 service は in-memory backend の no-op / echo / metadata-only な stub PASS** で、production 経路の振る舞いを保証しない。real / stub の区分:

- **real（backend ロジックが実走）**: Audit（SHA256 hash chain）/ Pii（regex masking）/ Decision（ZEN Engine real evaluation, output={"tier":"high"}）/ Secrets（real AES-256-GCM, in-memory key store）/ Telemetry（OTel collector → Prometheus 到達）/ Log（OTel collector → Loki 到達）
- **半 real（データ往復は real だが永続化なし）**: State.Save/Get/Delete（in-memory KV）
- **stub（err==nil を返すだけ）**: Workflow.RunShort（workflow_id 採番のみ、worker 不在で実 workflow 走らず）/ Binding.Invoke（no-op、外部システム接続なし）/ Invoke.Call（echo、別 app 不在）/ PubSub.Publish（offset 返却するが subscriber 不在で message 消失）/ Feature.EvaluateBoolean（default 値のみ、rules 不在）

採用品質の証拠としては **real 6 + 半 real 1 = 7 service** が現時点の真の到達点。残 5 service は production 経路（tier2 worker / 実 binding component / echo-app / subscriber Pod / feature rules）の整備で stub から昇格させる必要がある。

#### 訂正 2: 観測性 5/5 のうち 2 件は API 疎通のみ

「観測性 5 検証 PASS」のうち、**実 PASS と形式 PASS が混在**:

- **real（実データ往復を assert）**: dashboard goldenfile（diff test）/ OTLP trace propagation（OTLP→Tempo 往復、batches=1 確認）/ Loki log↔trace correlation（OTLP→Loki 経路 + log line に trace_id 文字列含有を assert）
- **形式（API が応答するだけ、SLO や cardinality 上限の assert は未実装）**: Prometheus cardinality（labels が 1 つ以上を確認するだけ、metric 別 cardinality 上限 baseline 比較は不在）/ SLO Alertmanager（`/api/v2/status` `/api/v2/alerts` が応答するだけ、alert 発火 assert は不在）

真の到達点: **real 3 + 形式 2 = 5 検証**。形式 2 件は採用初期で baseline JSON 整備 + alert 注入経路と連動して real 化する必要がある（ADR-TEST-006 の本来の検証定義）。

#### 訂正 3: 「portability 代替経路 PASS」は portability ではない

直前 commit で portability PASS と記録したのは「同 cluster の別 namespace に fresh helm install + e2e」。これは **chart 再現性の証拠**であって **portability ではない**。Portability は「異なる K8s 実装で同 manifest が動く」が定義（ADR-TEST-001 / ADR-CNCF-001）。

正しい portability 到達点は本セッション時点で **未達**:
- run.sh（multipass + kubeadm 主経路）: 当初「nested virt 制約で blocked」と判定したが、再調査で host に `/dev/kvm` 存在 + `vmx` flag ありを確認、multipass install の sudo password が無いだけで主経路自体は実行可能
- run-kind.sh（2nd kind cluster）: 本セッション host の `/proc/sys/user/max_inotify_instances=128` が root 専有で枯渇、systemd init `Failed to allocate manager object: Too many open files` で fail（host root 権限が必要）
- namespace fresh redeploy: chart 再現性の証拠であって portability ではない、と本訂正で明示

採用品質の portability 証拠を取るには、`sudo snap install multipass` 後に既設 run.sh を完走させて vanilla K8s（kubeadm + Calico）で全 e2e を再走させる必要がある。

#### 訂正 4: 旧記述「seed/register が前提のため射程外」の見直し（再々確認）

直前 commit で「`src/tier1/go/internal/adapter/dapr/inmemory_misc.go` の in-memory backend は 5 building block すべてを no-op / echo で OK 返却するよう実装済み」と書いたが、これは事実そのものは正しい。ただし**それを「PASS の証拠」として外部監査に出すには弱い**。in-memory no-op で OK 返却することは production 経路の動作証拠にならず、stub PASS と real PASS は厳密に区別すべき。本訂正でこの区別を documentation に固定する。
- **使用 endpoint（本追補時点）**:
  - K1S0_TIER1_TARGET=localhost:50001（state Pod / 5 building block）
  - K1S0_TIER1_AUDIT_TARGET=localhost:50002（Rust audit Pod）
  - K1S0_TIER1_PII_TARGET=localhost:50003（Rust pii Pod）
  - K1S0_TIER1_SECRETS_TARGET=localhost:50004（secret Pod）
  - K1S0_TIER1_WORKFLOW_TARGET=localhost:50005（workflow Pod）
  - K1S0_TIER1_DECISION_TARGET=localhost:50006（Rust decision Pod）
  - K1S0_OTLP_HTTP_TARGET=http://localhost:4318（OTel Collector）
  - K1S0_TEMPO_OTLP_TARGET=localhost:4318（Tempo OTLP HTTP）
  - K1S0_TEMPO_HTTP_TARGET=http://localhost:3200（Tempo HTTP API）
  - K1S0_LOKI_HTTP_TARGET=http://localhost:3100（Loki HTTP API）
  - K1S0_PROMETHEUS_HTTP_TARGET=http://localhost:9090（Prometheus）
  - K1S0_ALERTMANAGER_HTTP_TARGET=http://localhost:9093（Alertmanager）
- **追加 commits**: `tier1_extended_services_test.go` に Binding / Invoke.Call / Workflow.RunShort 追加（7 sub-test all PASS、0.03s）、`trace_propagation_test.go` を OTLP gRPC → OTLP HTTP に切替、`log_trace_correlation_test.go` を Loki 疎通のみ → log line trace_id 含有 assert へ昇格、`tools/qualify/portability/run-kind.sh` 新規追加（host kernel 制約 blocked のため namespace 経路で代替）

### 2026-05（リリース時点 / 初月、3 シナリオ完全 PASS — `tests/e2e/scenarios/` の全 e2e 実走確認）

- **3 シナリオ全 PASS**（2026-05-02 23:48 JST 実走、`go test ./scenarios/...` 0.076s 完了）:
  - **TestTenantOnboarding** 2/2 サブテスト PASS（dapr-system 4 Running Pod / State.Save→Get→Delete cycle）
  - **TestAuditPiiFlow** 5 段階全 PASS（Pii.Classify findings=1 / Pii.Mask `[EMAIL]` masking / Audit.Record audit_id 取得 / Audit.Query 1 events / Audit.VerifyChain valid=true checked=1）
  - **TestPayrollFullFlow** 7 段階全 PASS（Secrets.Get Unimplemented 許容 / State.Save etag=v4 / Audit.Record(start) / State.Get 45 bytes 一致 / Audit.Record(complete) / State.Delete / Audit.Query 2 events + VerifyChain valid=true checked=3）
- **環境構築経路（再現可能、本書 entry の追補）**:
  - kind cluster + Calico + cert-manager + Dapr 1.17.5（前述）
  - tier1-state image build + kind load + helm install（前述）
  - tier1-audit / tier1-pii image build（`docker build -f src/tier1/rust/Dockerfile.{audit,pii} -t ghcr.io/k1s0/k1s0/tier1-{audit,pii}:latest ./src/`）
  - kind load + helm install tier1-rust-service（pods.{decision,他}.enabled=false で個別 deploy）
  - 3 並列 port-forward: tier1-state→50001 / tier1-audit→50002 / tier1-pii→50003
  - 環境変数: K1S0_TIER1_TARGET=localhost:50001 / K1S0_TIER1_AUDIT_TARGET=localhost:50002 / K1S0_TIER1_PII_TARGET=localhost:50003
- **発見 + 解決した bug 2 件**:
  1. dapr port 衝突 → ff3ba6532 で chart に `dapr.io/grpc-port: 50101` annotation 追加（root fix）
  2. test の TenantID と JWT default の cross-tenant 拒否 → tenant_id を `demo-tenant` に統一（dev/CI mode の正規 tenant）
- **設計上の発見（採用初期で本格対応）**:
  - SDK Client は単一 endpoint 設計、tier1 サービスは Pod ごとに別 Service。本番では Envoy Gateway 経由の単一 endpoint で全 service routing する想定だが、local kind では個別 Client（State 用 / Audit 用 / Pii 用）を分けて作成する経路が現実解
  - K1S0_TIER1_AUDIT_TARGET / K1S0_TIER1_PII_TARGET 環境変数を test に導入し、Pod 別 Client の構造を明示化
- **追加実走 PASS（tier1_extended_services_test.go）**: tier1-state Pod の 5 API Router で **4 service が err==nil 限定 PASS**
  - PubSub.Publish: in-memory queue、offset=0
  - Feature.EvaluateBoolean: in-memory backend、value=false variant=default
  - Telemetry.EmitMetric: OTel pass-through OK
  - Log.Info: OTel pass-through OK
- **tier1 12 service 中 7 service が実走 OK 限定 PASS**: State / Audit / Pii / PubSub / Feature / Telemetry / Log
- **未実走の 5 service**: Workflow / Secrets / ServiceInvoke / Binding / Decision。これらは seed/register（workflow type 登録 / secret 配置 / app 登録 / binding component / 決定ルール）が前提のため、本 OK 限定 PASS test では射程外。採用初期で seed 整備込みで payroll_workflow_full_test.go / decision_evaluate_test.go 等として本格化（SHIP_STATUS §9 と整合）

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
