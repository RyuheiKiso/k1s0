# ADR-TEST-002: E2E テストを kind cluster + tools/local-stack + reusable workflow で自動化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / QA リード / 開発者体験チーム

## コンテキスト

ADR-TEST-001 で Test Pyramid（UT 70 / 契約 5 / 結合 20 / E2E 5）+ testcontainers を確定したが、E2E 層（Playwright UI シナリオ / k6 API 負荷）の **CI 自動化経路** は未定義のままである。具体的には ① E2E が走る Kubernetes cluster をどう調達するか、② 本番依存（Dapr / CNPG / Kafka / Valkey / OpenBao / Keycloak / Backstage / Grafana 等）をどう束ねるか、③ 既存 reusable workflow 4 本構成（IMP-CI-RWF-010）にどう接続するか、④ 採用検討者がローカルで再現する経路は何か、の 4 点が未確定である。

既存 `tests/e2e/scenarios/tenant_onboarding_test.go` は雛形 stub（`t.Skip("PHASE: release-initial")`）のみで、`tests/e2e/README.md` も「kind cluster 上で `infra/environments/dev/` を apply」と記述するに留まり、CI から呼ばれる経路や failure artifact の取り扱いが未定義。これは Phase 0（リリース時点）で Test Pyramid を回すための致命的欠落である。

E2E cluster 調達の選択肢は実用上以下のとおり:

- **既存 kind + `tools/local-stack/up.sh`**（ADR-POL-002 で local-stack SoT 制定済）
- **testcontainers ベースの単一プロセス E2E**（K8s 不要、シナリオを Go プロセス内で完結）
- **multipass kubeadm cluster**（本番 fidelity は最大、起動時間が長い）
- **CI 専用 ephemeral cluster**（GKE / EKS 等のクラウド K8s）

E2E の CI 統合経路の選択肢:

- **既存 reusable workflow と同様のパターンで `_reusable-e2e.yml` を新設**（IMP-CI-RWF-010 と整合）
- **`pr.yml` から直接 E2E job を呼ぶ**（reusable workflow を経由しない、ad-hoc 構成）
- **GitHub Actions schedule trigger で nightly workflow を別建て**

既存環境 / ADR との制約は以下を満たす必要がある:

- **ADR-POL-002（local-stack SoT）** との整合 — kind cluster の構成 SoT は `tools/local-stack/up.sh`、E2E もこれを使うべき
- **ADR-TEST-001（Test Pyramid + 時間予算）** との整合 — E2E は PR 時に実行しない、夜間バッチで 30 分以内
- **ADR-DEV-002（WSL2 + Docker runtime）** との整合 — ローカル開発機で E2E が再現可能であること
- **IMP-CI-RWF-010（reusable workflow 4 本）** の追加新設は許容、既存 4 本の責務分界を侵さない
- **既存 `tests/02_tests配置.md`（IMP-DIR-COMM-112）** の物理配置（E2E は Go 統一・独立 module）を尊重
- **採用検討者の再現容易性** — ローカルで `make verify-e2e` 相当 1 コマンドで再現可能

## 決定

**E2E テストの CI 自動化経路は、既存 `tools/local-stack/up.sh --role e2e` で起動する kind cluster + 本番再現スタック上で `tests/e2e/scenarios/` の Go テストを実行し、`.github/workflows/_reusable-e2e.yml`（新設）を nightly workflow（`nightly.yml`、新設）から呼ぶ構造で確定する。**

### 1. cluster 調達

`tools/local-stack/up.sh` を E2E にも拡張する。具体的には `--role e2e` を追加し、kind cluster（control-plane 1 + worker 3、Calico CNI）+ Argo CD + cert-manager + MetalLB + Istio Ambient + Kyverno + SPIRE + Dapr operator + flagd + CNPG + Strimzi Kafka + MinIO + Valkey + OpenBao + Backstage + 観測性スタック（Grafana / Loki / Tempo）+ Keycloak の役別フルスタックを起動する。`infra/environments/dev/` の overlay と整合する。

ADR-POL-002 の local-stack SoT 思想を E2E に拡張することで、開発者ローカル（devcontainer 起動時に postCreate で立ち上がる）と CI（nightly workflow で立ち上げる）が **同じ shell script で同じスタック** を起動する。SoT が割れず、ローカル再現性が原理的に担保される。

### 2. E2E test 実装

既存 `tests/02_tests配置.md`（IMP-DIR-COMM-112）の物理配置を踏襲する:

- `tests/e2e/go.mod` — 独立 Go module（`github.com/k1s0/k1s0/tests/e2e`、go 1.22+）
- `tests/e2e/scenarios/<scenario>_test.go` — 各 E2E シナリオの Go test（既存 `tenant_onboarding_test.go` を含む）
- `tests/e2e/helpers/` — cluster setup / auth / API client の共通ヘルパ
- `tests/e2e/testdata/` — シナリオ固有 fixtures（`tests/fixtures/` 共有でないもの）

シナリオは「ログイン → 業務画面遷移 → データ入力 → 保存 → 確認」のような 5〜10 ステップを 1 シナリオとし、Playwright（UI シナリオ）/ k6（API 負荷）/ Go test（tier1 公開 API 直叩き）の 3 系統で記述する。Go test は Playwright / k6 のシナリオを起動するラッパとしても機能し、CI から見ると「Go test 1 系統」で統一される。

failure artifact は以下を必ず保存する:

- `tests/e2e/.artifacts/<scenario>/screenshot/` — Playwright のスクリーンショット
- `tests/e2e/.artifacts/<scenario>/har/` — Playwright の HAR ファイル
- `tests/e2e/.artifacts/<scenario>/k6-summary.json` — k6 の負荷結果サマリ
- `tests/e2e/.artifacts/<scenario>/cluster-logs/` — `kubectl logs --all-namespaces` の dump

これらは GitHub Actions の `actions/upload-artifact@v4` で 14 日間保存。

### 3. CI 統合

`.github/workflows/_reusable-e2e.yml`（新設）を以下構造で実装:

```yaml
name: _reusable-e2e
on:
  workflow_call:
    inputs:
      role:
        description: "tools/local-stack/up.sh の --role 引数"
        required: false
        type: string
        default: "e2e"
      timeout_minutes:
        required: false
        type: number
        default: 45
jobs:
  e2e:
    runs-on: ubuntu-latest
    timeout-minutes: ${{ inputs.timeout_minutes }}
    steps:
      - uses: actions/checkout@v4
      - name: setup go
        uses: actions/setup-go@v5
        with:
          go-version: "1.22"
      - name: setup kind + local-stack
        run: ./tools/local-stack/up.sh --role ${{ inputs.role }}
      - name: run e2e
        working-directory: tests/e2e
        run: go test ./scenarios/... -v -timeout=30m
      - name: collect artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-artifacts
          path: tests/e2e/.artifacts/
          retention-days: 14
```

呼び出し元は新設 `nightly.yml`:

```yaml
name: nightly
on:
  schedule:
    - cron: "0 18 * * *"  # 03:00 JST = 18:00 UTC
  workflow_dispatch:      # 手動トリガも許容
permissions:
  contents: read
jobs:
  e2e:
    uses: ./.github/workflows/_reusable-e2e.yml
    with:
      role: e2e
      timeout_minutes: 45
```

PR ゲート（`pr.yml`）からは E2E を呼ばない（ADR-TEST-001 で「PR では実行しない、夜間バッチで全シナリオ」と決定済）。`workflow_dispatch` で起案者 / 採用初期協力者が必要時に手動起動可能。

### 4. ローカル再現

ローカルでの E2E 実行は以下 2 コマンドで完結:

```bash
./tools/local-stack/up.sh --role e2e   # devcontainer 起動時に postCreate で自動実行も可
cd tests/e2e && go test ./scenarios/... -v -timeout=30m
```

`Makefile` に `verify-e2e` target を追加し、上記 2 コマンドを 1 コマンドにまとめる。これにより採用検討者が `make verify-e2e` 1 コマンドでローカル再現できる。

## 検討した選択肢

### 選択肢 A: kind + tools/local-stack + reusable workflow（採用）

- 概要: 既存 `tools/local-stack/up.sh` を `--role e2e` で拡張、kind cluster + 本番再現スタック上で Go test 実行、`_reusable-e2e.yml` 新設で `nightly.yml` から呼ぶ
- メリット:
  - **ADR-POL-002 の local-stack SoT を E2E まで拡張**、構成 SoT が割れない
  - 既存 IMP-CI-RWF-010 の reusable workflow パターンと整合、責務分界が明確
  - 開発者ローカル / CI nightly が **同一 shell script** で同一スタック起動、再現性が原理的に担保
  - 採用検討者が `make verify-e2e` 1 コマンドでローカル再現可能
  - ADR-TEST-001 の CI 時間予算（夜間バッチ 30 分）と整合
- デメリット:
  - kind cluster 起動 + 全スタック helm install で nightly workflow の所要時間が約 15〜30 分かかる
  - GitHub Actions runner のリソース上限（4 core / 14 GB）で kind cluster + 全スタックを動かすため、リソース制約が tight
  - kind cluster の CNI（Calico）と本番 CNI（Cilium、ADR-NET-001）が異なるため、ネットワーク層の本番 fidelity は L5 conformance（ADR-TEST-003 で別途扱う）に依存

### 選択肢 B: testcontainers ベースの単一プロセス E2E

- 概要: E2E を K8s cluster 上ではなく testcontainers で個別依存を立てて Go プロセス内で完結させる
- メリット:
  - K8s cluster 不要、起動時間が短い（< 5 分）
  - PR 時の実行も可能になる（CI 時間予算超過しない）
  - testcontainers が ADR-TEST-001 で結合テスト用に既に採用済、追加学習不要
- デメリット:
  - **本番乖離が大きい**: K8s 上の Service Mesh / Network Policy / Operator 連携 / RBAC / SPIRE workload identity 等が再現できず、E2E の意味が失われる
  - 結合テスト（ADR-TEST-001 で testcontainers 採用）と E2E の責務分界が崩壊、Test Pyramid の層構成が形骸化
  - Playwright / k6 のシナリオが Go プロセスから切り離せず、UI / 負荷検証が成立しない

### 選択肢 C: multipass kubeadm + 別 workflow

- 概要: kind ではなく multipass で立てる Ubuntu VM 3 台 + kubeadm cluster で E2E を実行
- メリット:
  - 本番 fidelity が最大（ADR-INFRA-001 の本番ブートストラップと一致）
  - L5 conformance とクラスタ実装を統一できる（ADR-TEST-003 と統合可能）
- デメリット:
  - **GitHub Actions runner で multipass が動かない**（nested virtualization 制約）
  - 起動時間が 5〜10 分長く、nightly workflow の総時間が 1 時間超
  - **ADR-POL-002 の local-stack SoT を E2E に拡張する正攻法から外れる**（multipass は local-stack 範疇外）
  - 開発者ローカルで multipass を立ち上げる学習コストが追加発生、devcontainer の docker-outside-of-docker 経路と乖離

### 選択肢 D: CI 自動化なし（手動のみ）

- 概要: E2E を CI から実行せず、起案者 / 協力者が手動でローカル実行する運用に留める
- メリット:
  - 実装工数ゼロ
  - GitHub Actions runner のリソース消費なし
- デメリット:
  - **継続的な E2E 検証が成立しない**: PR ごと / 日次の自動検証が無いため、本番乖離バグが採用組織のフィードバックまで検出されない
  - 採用検討者が「k1s0 は E2E を CI で回していない」と判定し、testing maturity 評価が低下
  - ADR-TEST-001 の Test Pyramid 比率（E2E 5%）が「概念上だけ存在し実行されない」状態になり、ADR の決定が空洞化
  - ADR-TEST-001 で「E2E は夜間バッチで全シナリオ実行」と決定済の運用と矛盾

## 決定理由

選択肢 A（kind + tools/local-stack + reusable workflow）を採用する根拠は以下。

- **ADR-POL-002（local-stack SoT）の自然な拡張**: `tools/local-stack/up.sh` は既に開発者ローカル（devcontainer）/ 構成 SoT として制定済。E2E 用の `--role e2e` を追加するだけで CI と開発者ローカルが同一スタックを起動し、SoT が割れない。選択肢 C（multipass）は別 SoT を新設することになり、構成管理の二重化を生む
- **既存 reusable workflow パターンとの整合**: IMP-CI-RWF-010 の 4 本構成（lint / test / build / push）に E2E 専用 5 本目を新設する形で拡張、既存 4 本の責務分界を侵さない。`_reusable-e2e.yml` は同一 schema（`workflow_call` + `inputs`）で書かれ、開発者 / 採用検討者の認知負荷が最小
- **ローカル再現性の構造的担保**: 開発者が devcontainer 内で `make verify-e2e` を実行すると、CI の nightly workflow と **同じ shell script** が呼ばれ、同じ kind cluster + 同じスタックが起動する。「ローカルで動いたが CI で落ちる」「CI で通ったがローカルで再現できない」が原理的に発生しない。選択肢 B（testcontainers）は K8s 抽象化レイヤを欠き、選択肢 C（multipass）は CI / ローカルで cluster 実装が分かれる
- **CI 時間予算との整合**: ADR-TEST-001 で「E2E は PR 時に実行しない、夜間バッチで 30 分以内」と決定済。選択肢 A は nightly workflow（schedule trigger 03:00 JST）で実行し、所要時間 30〜45 分（kind 起動 15〜20 分 + E2E 実行 15〜20 分 + artifact 収集 5 分）が朝会前（07:00 JST）に完了する。選択肢 B は PR 時実行可だが本番乖離、選択肢 C は所要時間が予算超過
- **採用検討者への低再現コスト**: 選択肢 A は VSCode + Docker（業界標準）+ devcontainer で 5 分以内に環境到達後、`make verify-e2e` 1 コマンドで E2E が走る。選択肢 D（手動）は採用検討者が「自分で手順を組み立てる」必要があり、評価放棄リスク
- **退路の確保**: kind cluster の CNI（Calico）と本番 CNI（Cilium、ADR-NET-001）が異なる本番 fidelity 不足は、ADR-TEST-003（CNCF Conformance / Sonobuoy）で別途扱う構造的分離。L4 standard E2E（kind）と L5 conformance（kind multi-node + Sonobuoy）が層別に責務分界され、選択肢 C の「全部 multipass」のような重量化を避けながら fidelity を取れる

## 影響

### ポジティブな影響

- ADR-POL-002 の local-stack SoT が E2E にまで拡張され、構成 SoT が単一になる
- nightly workflow が朝会（07:00 JST）前に完了し、E2E 結果が日次で議論可能
- 採用検討者が `make verify-e2e` 1 コマンドで E2E をローカル再現でき、release artifact 中心の品質公開（採用検討者向け）が補強される
- failure artifact（screenshot / HAR / k6-summary / cluster-logs）が自動収集され、夜間に発生した E2E failure の翌朝トリアージが容易
- 既存 IMP-CI-RWF-010 のパターンに沿うため、Phase 1 以降に「path-filter で contracts / sdk 変更時のみ E2E を PR で起動」のような拡張が low-cost で可能
- 既存 `tests/e2e/scenarios/tenant_onboarding_test.go` の雛形 stub が本決定により採用初期で実装に置き換わる経路が確立

### ネガティブな影響 / リスク

- nightly workflow の GitHub Actions runner リソース消費（4 core / 14 GB / 30〜45 分）が継続的に発生し、Actions free tier の月間制限を消費する。Public repo では無制限だが、private repo に切り替える場合は予算管理が必要
- kind cluster + 全スタック helm install で nightly 所要時間 15〜30 分が固定費として発生、E2E シナリオが増えるごとに workflow 時間が長期化する。Phase 1 で「シナリオ並列実行」の最適化が要る
- E2E failure の検出が最大 24 時間遅延する（前日夜の break が翌朝発覚）。緊急修正時は `workflow_dispatch` の手動起動 + ローカル `make verify-e2e` で代替する Runbook 化が必要
- kind cluster の CNI が Calico / 本番が Cilium（ADR-NET-001）と異なるため、ネットワーク層 fidelity 不足が L4 standard E2E では検出されない。これは ADR-TEST-003（CNCF Conformance / Sonobuoy）で別途扱うが、L4 / L5 の責務分界を `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` で明文化する必要
- `tools/local-stack/up.sh --role e2e` のメンテが起案者 / SRE に集中、`infra/environments/dev/` の overlay と整合し続ける継続コストが発生

### 移行・対応事項

- `tools/local-stack/up.sh` に `--role e2e` を追加し、Argo CD + cert-manager + MetalLB + Istio Ambient + Kyverno + SPIRE + Dapr operator + flagd + CNPG + Strimzi Kafka + MinIO + Valkey + OpenBao + Backstage + 観測性スタック + Keycloak のフルスタック起動経路を整備
- `.github/workflows/_reusable-e2e.yml` を新設し、`workflow_call` で `role` / `timeout_minutes` を inputs として受け取る構造で実装
- `.github/workflows/nightly.yml` を新設し、`schedule: 0 18 * * *`（03:00 JST）+ `workflow_dispatch` で `_reusable-e2e.yml` を呼ぶ
- `tests/e2e/scenarios/` の `tenant_onboarding_test.go` 以外のシナリオ（`payroll_full_flow_test.go` / `audit_pii_flow_test.go` の雛形が `02_tests配置.md` で計画済）を採用初期で実装
- `tests/e2e/helpers/cluster_setup.go` で `tools/local-stack/up.sh --role e2e` の `--verify` 結果を取り込み、cluster ready の判定を実装
- `tests/e2e/.artifacts/` のディレクトリ構造を整備し、`actions/upload-artifact` の retention-days 14 と整合
- `Makefile` に `verify-e2e` target を追加（`tools/local-stack/up.sh --role e2e && cd tests/e2e && go test ./scenarios/... -v -timeout=30m`）
- `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` を ADR-TEST-003（CNCF Conformance / Sonobuoy）起票時に同時整備し、「L4 standard E2E は kind / Calico、L5 conformance は別 cluster で本番 fidelity」の責務分界を散文で明文化
- `tests/e2e/README.md` を本 ADR の決定に合わせて改訂（既存記述「kind cluster 上で `infra/environments/dev/` を apply」を `tools/local-stack/up.sh --role e2e` 経路に置き換え）
- ADR-POL-002 の「帰結」セクションに「E2E 用 `--role e2e` も local-stack SoT 配下とする」を追記する relate-back 作業
- 採用初期で nightly workflow の monthly 結果を `docs/40_運用ライフサイクル/e2e-results.md` で月次サマリ公開（採用検討者向け）

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— E2E 5% 比率と CI 時間予算の前提
- ADR-POL-002（local-stack を構成 SoT に統一）— 本 ADR の `--role e2e` 拡張の根拠
- ADR-DEV-002（WSL2 + Docker runtime）— 開発者ローカルでの E2E 再現経路
- ADR-INFRA-001（Cluster API + kubeadm）— 本番 cluster 構成、kind との fidelity 差の認識
- ADR-NET-001（CNI 選定）— kind Calico と本番 Cilium の差異
- ADR-DIR-002（infra 分離）— `infra/environments/dev/` の役割
- IMP-CI-RWF-010（reusable workflow 4 本）— 本 ADR が 5 本目を追加する根拠
- IMP-CI-PF-031（path-filter 単一真実源）— Phase 1 以降の差分実行最適化の前提
- IMP-DIR-COMM-112（tests 配置）— `tests/e2e/` の物理配置
- DS-DEVX-TEST-006（Playwright E2E）/ DS-DEVX-TEST-007（k6 API 負荷）— E2E 層の概要設計
- 関連 ADR（採用検討中）: ADR-TEST-003（CNCF Conformance / Sonobuoy）/ ADR-TEST-004（Chaos ツール選定）/ ADR-TEST-005（Upgrade / DR drill）/ ADR-TEST-006（観測性 E2E）/ ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）
