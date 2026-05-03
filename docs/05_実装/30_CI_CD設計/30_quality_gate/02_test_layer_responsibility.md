# 02. テスト層責務分界（L4 standard E2E と L5 conformance）

本ファイルは ADR-TEST-003（CNCF Conformance / Sonobuoy）で決定された L5 conformance 層と、ADR-TEST-008（e2e owner / user 二分構造）で決定された L4 standard E2E 層の責務分界を、実装段階の運用契約として固定する。`01_quality_gate.md` で扱う 4 ゲート（fmt / lint / unit-test / coverage）が L0–L2（Test Pyramid 下層、ADR-TEST-001）の品質を保証する一方、本ファイルは **K8s cluster を使う 2 つの上位層** の cluster 構成・実行頻度・本番 fidelity 目標を区別する。

> **L4 の実装段階**: ADR-TEST-008 で L4 を owner（multipass + kubeadm + 本番再現フルスタック、48GB host 専用）と user（kind + minimum stack、16GB host OK）に二分する設計を確定済。本ファイルでは ADR-TEST-008 の決定を実装段階の運用契約に展開する。実装（`tests/e2e/{owner,user}/` ディレクトリ + `tools/e2e/{owner,user}/up.sh` の専用スクリプト）は ADR の移行・対応事項に従って配置する。`tools/local-stack/up.sh --role` の cone profile 引数空間と e2e cluster orchestration は物理分離する。

## なぜ層を分けるのか

L4 standard E2E は ADR-TEST-008 で **owner suite**（multipass × 5 + kubeadm 3CP HA + Cilium + Longhorn + MetalLB + フルスタック、48GB host 専用、CI 不可、不定期実走、`make e2e-owner-*`）と **user suite**（kind + minimum stack、16GB host OK、PR + nightly CI 可、`make e2e-user-{smoke,full}`）に物理分離している。owner は OSS 完成度検証（業務シナリオ tier1→2→3 + 4 言語 SDK + 観測性 + security + HA / DR / upgrade）を本番再現スタック上で網羅、user は利用者の自アプリ動作確認を最小成立形で機械検証する。実装経路は ADR-TEST-008 で確定済、本ファイルではその責務分界を運用契約として記述する。

一方 ADR-TEST-003 で確定した CNCF Conformance は、Sonobuoy が **vanilla Kubernetes 機能のみ** を 500+ テストで検証する層であり、Argo CD / Istio / Dapr 等のスタック起動は **不要**（むしろテスト namespace を汚染するため避ける）。実行頻度も nightly ではなく月次で、ADR-CNCF-001 の「移行・対応事項」を充足する位置づけである。

両層を「kind cluster を使う E2E」として一括扱うと、cluster 起動コスト・実行時間予算・failure 時のトリアージ動線が混線する。本ファイルで責務分界を明文化することで、`tools/local-stack/up.sh --role` の引数体系・reusable workflow の分離・artifact 保管経路が ADR と整合する。

## L4 と L5 の対比

責務分界を以下表で固定する。L4 は ADR-TEST-008（owner / user 二分構造）で確定済、L5 は ADR-TEST-003 で確定済。本ファイルは両 ADR の決定を運用契約として固定する。L4 は owner / user の 2 系統に分かれるため 3 列で並列に記述する。

| 軸 | L4-owner（OSS 完成度検証） | L4-user（自アプリ動作確認） | L5 conformance |
|----|----------------------------|------------------------------|----------------|
| 起源 ADR | ADR-TEST-008 + ADR-TEST-009 / 010 / 011 | ADR-TEST-008 + ADR-TEST-010 | ADR-TEST-003 |
| cluster 起動 | `tools/e2e/owner/up.sh` | `tools/e2e/user/up.sh` | `tools/local-stack/up.sh --role conformance` |
| node 構成 | multipass × 5（3CP HA + 2W）+ kubeadm | kind（CP1 + W1） | kind（control-plane 1 + worker 3） |
| host RAM 要件 | 48GB host 専用 | 16GB host OK | Actions runner 14GB |
| CNI / CSI / LB | Cilium / Longhorn / MetalLB | Calico / local-path / extraPortMappings | Calico / local-path / extraPortMappings |
| 起動コンポーネント | フルスタック（Argo CD / Istio Ambient / Dapr / CNPG / Strimzi / MinIO / Valkey / OpenBao / Backstage / Grafana LGTM / Keycloak） | minimum（Dapr + tier1 facade + Keycloak + 1 backend） | vanilla K8s のみ（Sonobuoy が実行する Conformance テスト用） |
| 実行ツール | Go test + chromedp（tier3-web）+ k6（perf） | Go test + 利用者は test-fixtures (4 言語、ADR-TEST-010) 経由で TS Vitest+Playwright も可 | Sonobuoy v0.57+（`--mode certified-conformance`） |
| reusable workflow | （CI 不可） | `_reusable-e2e-user.yml` | `_reusable-conformance.yml` |
| trigger | localhost のみ（不定期、起案者判断） | `pr.yml`（smoke）+ `nightly.yml`（full） | `conformance.yml`（cron 月初 03:00 JST） |
| 実行頻度 | 不定期（release tag 切る前 / k8s upgrade 前 / SDK major 改訂後） | PR 毎（smoke 5 分以内）+ nightly（full 30〜45 分） | 月次 |
| 所要時間 | 約 1 時間 45 分（VM 起動 30 + 全件 60 + cleanup 15） | smoke 5 分 / full 30〜45 分 | 約 60〜120 分 |
| failure artifact | full-result.tar.zst / cluster-info / dmesg（git LFS 12 ヶ月） | screenshot / HAR / k6-summary（14 日保存） | sonobuoy results.tar.gz / summary.md（90 日保存 + 12 ヶ月版管理） |
| fidelity 目標 | 業務シナリオ + 4 言語 SDK + 観測性 + security + HA / DR / upgrade の網羅 | 利用者の自アプリ tier2/tier3 が k1s0 SDK 越しに動くか | vanilla K8s API surface の準拠（CNCF 認証経路） |
| 代替保証 | ADR-TEST-011（release tag ゲート、cut.sh で PASS sha256 必須） | CI 機械検証（pr.yml + nightly.yml） | CI 機械検証（conformance.yml） |

## kind / Calico を共有する理由と限界

両層は cluster 実装（kind）と CNI（Calico）を共有する。これは ADR-NET-001（kind multi-node = Calico）の決定が両層に等しく適用されるためで、cluster 実装を分けるとローカル開発機の起動経路が散逸する。一方で本番 cluster は kubeadm + Cilium（ADR-INFRA-001 / ADR-NET-001）であり、kind / Calico では以下の本番 fidelity が取れない:

- **CSI**: kind は local-path-provisioner、本番は Longhorn（ADR-STOR-001）。PV snapshot / replication の検証は不可
- **LB**: kind は extraPortMappings、本番は MetalLB（ADR-STOR-002）の L2 / BGP モード。LB 挙動の検証は不可
- **CNI eBPF**: kind は Calico iptables、本番は Cilium eBPF。NetworkPolicy 強制 / mTLS / Hubble 観測性の検証は不可
- **multi-AZ**: kind は単一 host 内、本番は multi-AZ。topology spread / zone failure の検証は不可

これらは L4 / L5 の射程外として `docs/40_運用ライフサイクル/conformance-results.md`（採用初期で初回作成）の skip 項目に明記する。本番 fidelity の完全検証は採用組織の production cluster 上で取得する設計となり、本リポジトリの CI 経路では取得しない（採用組織側で取得した結果を README に背景情報として cite する経路は採用初期で別途整備）。

## ローカル再現と CI の同一経路

L4-owner / L4-user は専用スクリプト（`tools/e2e/owner/up.sh` / `tools/e2e/user/up.sh`）で起動し、L5 は `tools/local-stack/up.sh --role conformance` で起動する。各層の起動経路は別エントリで物理分離されているため、開発者が devcontainer 内で実行する経路と CI workflow が呼ぶ経路が機械的に一致する。helm values / manifests のレベルでは ADR-POL-002（local-stack を構成 SoT に統一）の SoT を尊重し、`tools/e2e/{owner,user}/up.sh` は内部で `tools/local-stack/install/<component>/` を install helper として再利用する。これにより orchestration（起動順序 / cluster ライフサイクル）は層ごとに独立、構成（何を install するか）は SoT に集約という責務分離を成立させる。

`Makefile` には `verify-conformance`（L5）を追加済、L4 用は ADR-TEST-008 で `make e2e-owner-{full,platform,observability,security,ha-dr,upgrade,sdk-roundtrip,tier3-web,perf}` / `make e2e-user-{smoke,full}` の 10 target を追加する設計を確定済。実装は ADR-TEST-008 の移行・対応事項に従って Makefile を改訂する。

## 拡張余地

本ファイルは L4 / L5 の責務分界に射程を絞っており、以下は後続 ADR-TEST-* で順次拡張する:

- L4 自動化経路の再策定（前 ADR-TEST-002 撤回後の新 ADR と本ファイル改訂）
- L7 chaos / L8 scale / L9 upgrade / L10 DR との関係（ADR-TEST-004 / 005 起票時に本ファイルを再構造化）
- Phase 移行（リリース時点 → 採用初期 → 採用後の運用拡大時）での責務分界変化
- OSSF Scorecard / OpenSSF Best Practices Badge / SLSA との対応マッピング
- テスト属性タグ（@slow / @flaky / @security）と層の関係（ADR-TEST-007）

## 対応 ADR / IMP / DS

- ADR-TEST-001（Test Pyramid + testcontainers でテスト戦略を正典化）
- ADR-TEST-003（CNCF Conformance を Sonobuoy + kind multi-node + Calico で月次実行）
- ADR-NET-001（kind multi-node = Calico）
- ADR-POL-002（local-stack を構成 SoT に統一）
- ADR-INFRA-001（kubeadm + Cluster API）— 本番 cluster との fidelity 差認識
- ADR-CNCF-001（vanilla K8s + CNCF Conformance）— L5 の起源
- IMP-CI-RWF-010（reusable workflow 構成）
- IMP-CI-CONF-001〜005（ADR-TEST-003 で確定、`90_対応IMP-CI索引/01_対応IMP-CI索引.md` への展開は別 commit）
- IMP-DIR-COMM-112（tests 配置）
- L4 用の対応 ADR / DS-DEVX-TEST-* は刷新後の新 ADR で再策定
