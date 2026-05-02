# 02. テスト層責務分界（L4 standard E2E と L5 conformance）

本ファイルは ADR-TEST-002（E2E 自動化）と ADR-TEST-003（CNCF Conformance / Sonobuoy）で決定されたテスト層の責務分界を、実装段階の運用契約として固定する。`01_quality_gate.md` で扱う 4 ゲート（fmt / lint / unit-test / coverage）が L0–L2（Test Pyramid 下層、ADR-TEST-001）の品質を保証する一方、本ファイルは **kind cluster を使う 2 つの上位層** の cluster 構成・実行頻度・本番 fidelity 目標を区別する。

## なぜ層を分けるのか

ADR-TEST-002 で確定した E2E 自動化経路は、`tools/local-stack/up.sh --role e2e` で起動する **kind cluster + 本番再現フルスタック**（Argo CD / Istio Ambient / Dapr / CNPG / Strimzi / MinIO / Valkey / OpenBao / Backstage / Grafana LGTM / Keycloak）の上で Go test を nightly に走らせる。これは「採用組織の業務フローが tier1→2→3 を貫通して動くか」を検証する層であり、本番再現スタックが起動している前提で初めて意味を持つ。

一方 ADR-TEST-003 で確定した CNCF Conformance は、Sonobuoy が **vanilla Kubernetes 機能のみ** を 500+ テストで検証する層であり、Argo CD / Istio / Dapr 等のスタック起動は **不要**（むしろテスト namespace を汚染するため避ける）。実行頻度も nightly ではなく月次で、ADR-CNCF-001 の「移行・対応事項」を充足する位置づけである。

両層を「kind cluster を使う E2E」として一括扱うと、cluster 起動コスト・実行時間予算・failure 時のトリアージ動線が混線する。本ファイルで責務分界を明文化することで、`tools/local-stack/up.sh --role` の引数体系・reusable workflow の分離・artifact 保管経路が ADR と整合する。

## L4 と L5 の対比

責務分界を以下表で固定する。表の各行は ADR-TEST-002 / 003 で確定済の決定事項であり、本ファイルでの独自決定はない。

| 軸 | L4 standard E2E | L5 conformance |
|----|----------------|----------------|
| 起源 ADR | ADR-TEST-002 | ADR-TEST-003 |
| cluster 起動 | `tools/local-stack/up.sh --role e2e` | `tools/local-stack/up.sh --role conformance`（新設） |
| node 構成 | control-plane 1 + worker 3（Calico CNI） | control-plane 1 + worker 3（Calico CNI） |
| 起動コンポーネント | フルスタック（Argo CD / Istio Ambient / Dapr / CNPG / Strimzi / MinIO / Valkey / OpenBao / Backstage / Grafana LGTM / Keycloak） | vanilla K8s のみ（Sonobuoy が実行する Conformance テスト用） |
| 実行ツール | Go test（`tests/e2e/scenarios/`、Playwright / k6 を Go test がラップ） | Sonobuoy v0.57+（`--mode certified-conformance`） |
| reusable workflow | `_reusable-e2e.yml` | `_reusable-conformance.yml` |
| trigger workflow | `nightly.yml`（cron 03:00 JST） | `conformance.yml`（cron 月初 03:00 JST） |
| 実行頻度 | 毎晩 | 月次 |
| 所要時間 | 約 30〜45 分 | 約 60〜120 分 |
| failure artifact | screenshot / HAR / k6-summary / cluster-logs（14 日保存） | sonobuoy results.tar.gz / summary.md（90 日保存 + 12 ヶ月版管理） |
| fidelity 目標 | 業務シナリオ（tier1→2→3）の貫通 | vanilla K8s API surface の準拠（CNCF 認証経路） |

## kind / Calico を共有する理由と限界

両層は cluster 実装（kind）と CNI（Calico）を共有する。これは ADR-NET-001（kind multi-node = Calico）の決定が両層に等しく適用されるためで、cluster 実装を分けるとローカル開発機の起動経路が散逸する。一方で本番 cluster は kubeadm + Cilium（ADR-INFRA-001 / ADR-NET-001）であり、kind / Calico では以下の本番 fidelity が取れない:

- **CSI**: kind は local-path-provisioner、本番は Longhorn（ADR-STOR-001）。PV snapshot / replication の検証は不可
- **LB**: kind は extraPortMappings、本番は MetalLB（ADR-STOR-002）の L2 / BGP モード。LB 挙動の検証は不可
- **CNI eBPF**: kind は Calico iptables、本番は Cilium eBPF。NetworkPolicy 強制 / mTLS / Hubble 観測性の検証は不可
- **multi-AZ**: kind は単一 host 内、本番は multi-AZ。topology spread / zone failure の検証は不可

これらは L4 / L5 の射程外として `docs/40_運用ライフサイクル/conformance-results.md`（採用初期で初回作成）の skip 項目に明記する。本番 fidelity の完全検証は採用組織の production cluster 上で取得する設計となり、本リポジトリの CI 経路では取得しない（採用組織側で取得した結果を README に背景情報として cite する経路は採用初期で別途整備）。

## ローカル再現と CI の同一経路

両層とも **同一の `tools/local-stack/up.sh` を `--role` 引数で切り替える** 設計のため、開発者が devcontainer 内で実行する経路と CI workflow が呼ぶ経路が機械的に一致する。これは ADR-POL-002（local-stack を構成 SoT に統一）の延長で、cluster 構成が割れない構造的担保である。

`Makefile` には `verify-e2e`（L4）/ `verify-conformance`（L5）の 2 target を追加し、開発者が `make verify-e2e` / `make verify-conformance` の 1 コマンドでローカル再現できる。CI workflow は同一 shell script を呼ぶため、「ローカルで動いたが CI で落ちる」/「CI で通ったがローカルで再現できない」が原理的に発生しない。

## 拡張余地

本ファイルは L4 / L5 の責務分界に射程を絞っており、以下は後続 ADR-TEST-* で順次拡張する:

- L7 chaos / L8 scale / L9 upgrade / L10 DR との関係（ADR-TEST-004 / 005 / 006 起票時に本ファイルを再構造化）
- Phase 移行（リリース時点 → 採用初期 → 採用後の運用拡大時）での責務分界変化
- OSSF Scorecard / OpenSSF Best Practices Badge / SLSA との対応マッピング
- テスト属性タグ（@slow / @flaky / @security）と層の関係（ADR-TEST-007）

## 対応 ADR / IMP / DS

- ADR-TEST-001（Test Pyramid + testcontainers でテスト戦略を正典化）
- ADR-TEST-002（E2E を kind + tools/local-stack + reusable workflow で自動化）
- ADR-TEST-003（CNCF Conformance を Sonobuoy + kind multi-node + Calico で月次実行）
- ADR-NET-001（kind multi-node = Calico）
- ADR-POL-002（local-stack を構成 SoT に統一）
- ADR-INFRA-001（kubeadm + Cluster API）— 本番 cluster との fidelity 差認識
- ADR-CNCF-001（vanilla K8s + CNCF Conformance）— L5 の起源
- IMP-CI-RWF-010（reusable workflow 4 本構成、本ファイルでは 5 / 6 本目を追加）
- IMP-CI-CONF-001〜005（ADR-TEST-003 で確定、`90_対応IMP-CI索引/01_対応IMP-CI索引.md` への展開は別 commit）
- IMP-DIR-COMM-112（tests 配置）
- DS-DEVX-TEST-006（Playwright E2E）/ DS-DEVX-TEST-007（k6 API 負荷）
