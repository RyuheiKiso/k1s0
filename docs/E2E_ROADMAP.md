# E2E 完璧完成へのロードマップ

本書は k1s0 の e2e（end-to-end testing）を **CNCF Graduated 級 OSS と並ぶ testing maturity** に到達させるための live document である。`SHIP_STATUS.md` / `AUDIT.md` / `INDEX.md` と並列で、ルート直下に配置する。Phase 完了時に該当節を「達成済」に更新し、未達項目の現状を継続的に明示する責務を持つ。

採用検討組織が release artifact を起点に k1s0 の testing maturity を評価する際、ADR-TEST-001〜007 の決定だけでは「いつ・何が・どこまで実装されるか」が読めない。本書は ADR の決定を時系列・タスク粒度に落とし込み、Phase 0（リリース時点）から Phase 4（CNCF Graduated 級）までの完成経路を示す。各 Phase の移行条件は客観条件で書かれ、「未来への先送り」（CLAUDE.md ポリシー）を構造的に防ぐ。

## 完成定義

「e2e 完璧かつ完全に完成」を以下のように定義する。曖昧表現を排し、外部評価軸との対応で機械検証可能な状態を目指す。

> **ADR-TEST-001〜007 の全決定が release artifact に反映され、CNCF Project Maturity Levels（Sandbox / Incubating / Graduated）/ OSSF Scorecard / OpenSSF Best Practices Badge の testing maturity 評価軸で減点されないレベルに到達し、採用検討組織が `make verify-*` 1 系統で全 e2e を手元再現できる状態。**

この定義は以下 4 軸で計測される:

1. **ADR 決定の reflection 率** — ADR-TEST-001〜007 の各「決定」「移行・対応事項」が実装されている割合（手動レビューで計測、Phase 完了時 100% を目標）
2. **CNCF / OpenSSF / OSSF 外部評価軸** — Sonobuoy PASS / Best Practices Badge ステータス / Scorecard スコア（機械評価）
3. **release artifact の testing maturity 証跡** — Sonobuoy report / SLSA provenance / Chaos drill 結果 / DR drill 結果 / 観測性 E2E 結果が release tag 直下に同梱されている件数（機械集計）
4. **採用検討者の手元再現容易性** — `make verify-e2e` / `make verify-conformance` / `make qualify` が 1 コマンドで `git clone` 直後の devcontainer から起動成功する（手動検証、四半期）

## Phase 構造

ADR-TEST-001 / 004 / 005 で「リリース時点」「採用初期」「採用後の運用拡大時」「採用側のマルチクラスタ移行時」と段階導入が決まっている。これに対応して 5 段の Phase を設定する。Phase 移行は以下の客観条件で起動し、起案者の主観判断ではない:

| Phase | 移行条件（客観条件） | 期間目安 | 主体リソース |
|---|---|---|---|
| **Phase 0** | リリース時点（OSS 公開時点） | 完了済 | 起案者 1 名 |
| **Phase 1** | OSS 公開後、採用初期に contributor 1 名以上が PR を提出 | 1.5〜3 ヶ月 | 起案者 + 採用初期 1 名 |
| **Phase 2** | contributor 2 名以上の継続参画 / sponsor 月 50 USD 以上獲得（ADR-TEST-001 Phase 表と整合） | 2〜4 ヶ月 | contributor 2-3 名 |
| **Phase 3** | sponsor 月 200 USD 以上 / 商用採用検討組織出現 / CNCF Sandbox 申請開始 | 3〜6 ヶ月 | 3-5 名 |
| **Phase 4** | CNCF Sandbox 採択後、Incubating 申請 / Graduated 級到達 | 長期（1〜3 年） | 専任 0.5+ FTE |

Phase 移行条件が未充足の段階で次 Phase の作業を開始しない。これは「先送り防止」と「リソース過剰投入防止」の両軸で、起案者の単独運用が破綻するのを構造的に防ぐ。

## Phase 0 達成記録（リリース時点）

リリース時点で以下が完備されている。本書執筆時点では Phase 0 完了状態であり、未達項目はない。

### ADR 起票（10 commit）

| commit | 主旨 |
|---|---|
| `ccce5828e` | ADR-TEST-001 起票（Test Pyramid + testcontainers）+ docs-orphan 2 件解消 |
| `854f167cf` | ADR-TEST-002 起票（E2E 自動化 / kind + tools/local-stack + reusable workflow） |
| `60939ffe3` | ADR-TEST-003 起票（CNCF Conformance / Sonobuoy 月次）+ `02_test_layer_responsibility.md` 新設 |
| `39cff033b` | ADR-TEST-004 起票（LitmusChaos 採用）+ 概要設計 Chaos Mesh 訂正 |
| `4a7de14a5` | ADR-TEST-005 起票（Upgrade / DR drill、Velero 不採用） |
| `9ed921af5` | ADR-TEST-006 起票（観測性 E2E 5 検証） |
| `96808d037` | ADR-TEST-007 起票（テスト属性タグ + フェーズ分離） |
| `720528057` | 既存 12 ADR への relate-back + ADR-DATA-003 Velero 想定訂正 |
| `a8208f2f2` | ADR-TEST-001 self-relate-back（Chaos / CI 予算行訂正） |
| `54e462f01` | IMP-CI 索引に CONF / TAG 系列 10 ID 追加（58 → 68） |

### 達成事項

- **ADR-TEST 系列 7 本完備**（全 5 段構成、検討肢 3 件以上、決定理由、影響セクションを充足）
- **既存実装と整合**: 既存 GitHub Actions / verify-local.sh / 10 役 devcontainer / pre-commit framework を覆さず
- **既存 ADR 12 本との双方向トレーサビリティ確立**: ADR-CNCF-001 / POL-002 / DIR-002 / DATA-001/003 / INFRA-001 / NET-001 / OBS-001/002/003 / OPS-001 / TIER1-001
- **SoT drift 解消**: 概要設計 Chaos Mesh → LitmusChaos / ADR-DATA-003 Velero 想定 → ADR-TEST-005 不採用
- **docs-orphan 解消**: ADR-TEST-001 / ADR-DEVEX-003 / IMP-CI-CONF-* / IMP-CI-TAG-* の 4 系統

### Phase 0 で未実装（Phase 1 以降に持ち越し）

ADR の「移行・対応事項」で「リリース時点では skeleton のみ / 実装は採用初期から」と明記された項目は、Phase 0 完了状態でも未実装で正しい。これは ADR の決定そのものが「リリース時点では実装ゼロ」を採用しているため。

## Phase 1 詳細タスク（採用初期）

Phase 1 は OSS 公開後、採用初期に contributor 1 名以上が PR を提出した時点で開始する。10 系統のタスクから構成され、それぞれ独立した PR で完結可能（並列実行可、ただし依存関係あり）。

### 1. tools/local-stack 拡張

`tools/local-stack/up.sh` に `--role e2e` と `--role conformance` を追加する。前者は本番再現フルスタック（Argo CD / Istio Ambient / Dapr / CNPG / Strimzi / MinIO / Valkey / OpenBao / Backstage / Grafana LGTM / Keycloak）、後者は vanilla K8s のみ + Calico CNI（Sonobuoy 用）。`--verify` で cluster ready 判定を出力する。

依存: なし（Phase 1 の出発点）。
工数: 3〜5 人日。
完成判定: `tools/local-stack/up.sh --role e2e --verify` が exit 0 で完了し、`kubectl get pods --all-namespaces` で全 Pod Ready。

### 2. reusable workflow と trigger workflow の新設

`.github/workflows/_reusable-e2e.yml` / `_reusable-conformance.yml`（reusable）+ `nightly.yml` / `conformance.yml` / `weekly.yml` / `flaky-report.yml`（trigger、新設）の 6 本を整備する。既存 `_reusable-lint.yml` / `_reusable-test.yml` / `_reusable-build.yml` / `_reusable-push.yml` の 4 本構成（IMP-CI-RWF-010）に **5 本目以降を追加** する形で、既存パターンを継承する。

依存: 1（tools/local-stack 拡張）に依存。
工数: 5〜8 人日。
完成判定: `nightly.yml` が `workflow_dispatch` で起動成功し、`tests/e2e/scenarios/` の少なくとも 1 シナリオが PASS で完了する。

### 3. tests/e2e/scenarios の Skip 解除と実装

既存 `tests/e2e/scenarios/tenant_onboarding_test.go` の `t.Skip("PHASE: release-initial")` を解除して実装する。続いて `payroll_full_flow_test.go` / `audit_pii_flow_test.go` を `tests/02_tests配置.md` の雛形に従って実装する。Playwright（UI）/ k6（API 負荷）を Go test がラップする構造で、3 シナリオすべて kind cluster 上の本番再現スタックを相手に動作する。

依存: 1（tools/local-stack 拡張）に依存。
工数: 7〜10 人日（3 シナリオ × 2〜3 人日）。
完成判定: 3 シナリオすべてが nightly workflow で連続 4 週 PASS。

### 4. 観測性 E2E 検証 1（trace 貫通）の実装

ADR-TEST-006 の 5 検証のうち最初の検証 1（OTLP trace 貫通）を `tests/e2e/observability/trace-propagation/` に実装する。Tempo HTTP API（`/api/traces/<trace-id>`）を叩いて span tree が tier1→2→3 の順で連続していることを Go test で assert する。残 4 検証（cardinality / log↔trace / SLO alert / dashboard goldenfile）は Phase 2 で順次完備する。

依存: 1（tools/local-stack 拡張）に依存。
工数: 3〜5 人日。
完成判定: trace 貫通検証が nightly workflow で連続 4 週 PASS。

### 5. 観測性 baseline 整備

ADR-TEST-006 の検証 2（cardinality regression）と検証 5（dashboard goldenfile）の baseline JSON を版管理に組み込む。`tests/e2e/observability/cardinality/baselines/<metric>.json` で各 metric の cardinality 上限値、`tests/e2e/observability/dashboard-goldenfile/baselines/<dashboard>.json` で `infra/observability/grafana/dashboards/*.json` の固定済 baseline を保存する。Phase 2 で実 assertion を実装するための前提準備。

依存: なし（独立）。
工数: 2〜3 人日。
完成判定: baseline JSON が `tests/e2e/observability/*/baselines/` に commit され、PR レビュー対象として CODEOWNERS が設定されている。

### 6. flaky 検出 + tag lint の整備

`tools/qualify/flaky-detector.py` を新設し、GitHub Actions API 経由で直近 20 PR の CI 結果を取得して fail 率 ≥ 5% を `tests/.flaky-quarantine.yaml` に自動追加する。`tools/lint/test-tag-lint.sh` を新設し、5 秒超の test に `@slow` タグが付与されていない場合に CI で fail させる。両者とも ADR-TEST-007 の IMP-CI-TAG-004 / 005 を実装段階に展開する。

依存: 2（reusable workflow）に依存。
工数: 3〜5 人日。
完成判定: flaky-detector.py が PR で fail 率を出力し、`tests/.flaky-quarantine.yaml` の更新 PR が起案者レビュー対象として運用開始。

### 7. Makefile target 追加

`Makefile` に `verify-e2e` / `verify-conformance` target を追加する。それぞれ `tools/local-stack/up.sh --role <role> && cd tests/e2e && go test ./scenarios/... -v -timeout=30m` 相当の 1 コマンドで完結する経路を提供する。採用検討者が devcontainer 起動後に 1 コマンドでローカル再現できる経路の確立。

依存: 1（tools/local-stack 拡張）に依存。
工数: 1 人日。
完成判定: `make verify-e2e` が devcontainer 内で exit 0 で完了する。

### 8. Runbook skeleton 整備

ADR-TEST-005（DR drill 4 経路）/ ADR-TEST-004（Chaos 5 シナリオ、Phase 2 用 skeleton）/ ADR-TEST-007（flaky quarantine）に対応する Runbook を 8 セクション形式（ADR-OPS-001 準拠）で skeleton 整備する。具体的には `ops/runbooks/RB-UPGRADE-001-kubeadm-minor-upgrade.md` / `RB-DR-001〜004-*.md` / `RB-CHAOS-001〜005-*.md`（Phase 2 用、skeleton のみ） / `RB-TEST-001-flaky-quarantine.md` の合計 11 本。

依存: なし（独立）。
工数: 5〜8 人日（11 本 × 0.5〜1 人日）。
完成判定: 全 11 Runbook が skeleton として `ops/runbooks/` に配置され、YAML frontmatter（runbook_id 等 10 項目）が充足。

### 9. 月次サマリ doc の初回作成

`docs/40_運用ライフサイクル/conformance-results.md` / `e2e-results.md` / `dr-drill-results.md` を採用初期で初回作成する。Phase 1 の 1 ヶ月目から月次更新を運用開始し、採用検討者が時系列で testing maturity を確認できる経路を確立する。

依存: 2（reusable workflow）+ 3（tests/e2e 実装）に依存。
工数: 1〜2 人日（初回テンプレ作成 + 月次運用フロー確立）。
完成判定: 3 doc が `docs/40_運用ライフサイクル/` に配置され、Phase 1 の 1 ヶ月目時点で初回サマリが記載されている。

### 10. L6 portability 手動 1 回実走

ADR-TEST-001 Phase 0 例外として「L6 portability は Phase 0 で手動 1 回、Phase 3 で自動化」と決定済。**ADR-CNCF-001 / ADR-INFRA-001 と整合させ、portability は「kind 以外の vanilla K8s 実装」で動くことを検証する**。具体的には `tools/qualify/portability/run.sh` を local で 1 回実行（multipass + kubeadm + Calico の 3-node cluster 起動 → cluster Ready 確認 → cluster-info を artifact 保存）し、結果を `docs/40_運用ライフサイクル/portability-results.md`（新設）に記載する。k3s 派生（k3d 含む）は ADR-CNCF-001 で次点と判定済のため不採用。マネージド K8s（EKS / GKE / AKS）での実走は採用組織側の責務（採用組織の手元で同 script を流用可能）。

依存: 1〜9 のうち最低 1（tools/local-stack 拡張）と 7（Makefile target）が完了していればよい。
工数: 1〜2 人日（cluster 立ち上げ + 1 回実走 + report 作成）。
完成判定: multipass + kubeadm + Calico の 3-node cluster で `tools/qualify/portability/run.sh` が exit 0 で完了し、cluster-info / conformance-link が `tests/.portability/<YYYY-MM-DD>/` に記録され、`portability-results.md` に集約される。

### Phase 1 完成判定

Phase 1 は以下すべてを満たした時点で完了とし、Phase 2 移行条件（contributor 2 名以上 / sponsor 50 USD/月）が充足するまで Phase 2 作業を開始しない:

- 10 系統タスクすべて完了
- nightly E2E が連続 4 週 PASS
- CNCF Conformance 月次運用が 1 回以上完了し、初回 Sonobuoy report が release artifact に同梱
- 観測性 E2E 検証 1（trace 貫通）が連続 4 週 PASS
- L6 portability 手動 1 回完了
- 11 Runbook の skeleton 整備
- 月次サマリ doc の初回作成

## Phase 2 概要（採用後の運用拡大時）

Phase 2 では採用組織の運用拡大に伴い、Phase 1 で skeleton として確保した Chaos / DR drill / 観測性 E2E 5 検証完備 / DAST / mutation testing / coverage 90% を本格稼働させる。各 ADR で「採用後の運用拡大時」と決定された項目すべての実装着手フェーズ。

主要マイルストーン:

- LitmusChaos デプロイ（`infra/chaos/`、`operation` namespace）+ 5 シナリオ CRD 配置 + 週次 CronChaosEngine 運用開始
- Upgrade drill 月次運用開始（kubeadm 公式 plan/apply/node 経路、staging cluster で N → N+1）
- DR drill 4 経路（A: etcd snapshot / B: GitOps 完全再構築 / C: barman-cloud / D: Realm Export）の四半期ローテーション運用開始
- 観測性 E2E 検証 2-5 完備（cardinality / log↔trace / SLO alert / dashboard goldenfile）
- OWASP ZAP / DAST 統合（`@security` タグ、weekly workflow 経由）
- mutation testing 導入（IMP-CI-RWF-018 の運用拡大段階）
- coverage 90% 強制化（IMP-CI-QG-065 運用拡大段階）
- Backstage TechInsights ファクト送信開始
- Litmus Portal デプロイ（採用組織の運用チーム向け Web UI）

工数見積: 60〜100 人日（contributor 2-3 名で 2〜4 ヶ月）。

完成判定: 全項目が稼働し、月次レポート（conformance-results / e2e-results / dr-drill-results / chaos-drill-results / observability-results）が継続更新されている。

## Phase 3 概要（CNCF Sandbox 申請 / 採用側のマルチクラスタ移行時）

Phase 3 では CNCF Sandbox 申請パッケージを完備し、外部評価軸（OSSF Scorecard / OpenSSF Best Practices Badge / SLSA）の達成水準を高める。採用側のマルチクラスタ移行時にも対応するため、cross-cluster failover や 24h soak も整備する。

主要マイルストーン:

- OpenSSF Best Practices Badge **Silver** 取得（自己評価 70 項目充足）
- OSSF Scorecard **8.0/10 以上** 達成（17 項目、CI-Tests 除く）
- SLSA **L3** build provenance 完備（in-toto attestation + cosign keyless 署名 + Rekor 公開）
- cross-cluster failover drill（multi-region 模擬、ADR-TEST-005 経路 D の拡張）
- 24h soak test 月次運用化
- L6 portability 自動化拡張（採用組織 cluster + EKS / GKE / AKS の検証経路を採用組織と協調、ADR-CNCF-001 と整合）
- testgrid 相当の dashboard 公開（採用検討者向け）
- CNCF Sandbox 申請パッケージ完成（testing maturity 軸の証跡が `compliance/` に統合同梱）
- CNCF TAG Contributor Strategy への提示（採用時の助言獲得）

工数見積: 100〜200 人日（3-5 名で 3〜6 ヶ月）。

完成判定: CNCF Sandbox 採択 + Best Practices Badge Silver 取得 + Scorecard 8.0+ + SLSA L3 PASS。

## Phase 4 概要（CNCF Graduated 級）

Phase 4 では CNCF Graduated 級 OSS（Kubernetes / Istio / Cilium / ArgoCD）と並ぶ testing maturity に到達する。本書の完成定義の最終形態。

主要マイルストーン:

- OpenSSF Best Practices Badge **Gold** 取得
- SLSA L4 検討（reproducible build の完全実現）
- testgrid 相当 dashboard の高度化（CNCF プロジェクトの公開 dashboard と同等）
- CNCF Graduated 級慣行（chaos / scale / soak / upgrade / DR / portability の全層が release ごとに継続検証）
- 採用組織が CNCF Conformance 認証を取得する際の参考実装として cite される

工数見積: 200〜400 人日（専任 0.5+ FTE で長期）。

完成判定: CNCF Incubating → Graduated 申請パッケージ完備 + 全 testing 軸が外部評価で減点なし。

## 完成判定基準の機械検証手順

各 Phase の完成判定は手作業ではなく、可能な限り機械検証で行う。これは「主観で完成宣言する」を排し、採用検討者が同じ基準で確認できる経路を確立するため。

### Phase 1 完成の機械検証

```bash
# tools/qualify/phase1-completion-check.sh（Phase 1 で新設）
#
# 1. 10 系統タスクの完了状態を git log + ファイル存在で確認
# 2. nightly E2E の直近 4 週の workflow run を GitHub Actions API から取得し全 PASS 確認
# 3. tests/.conformance/ ディレクトリに少なくとも 1 件の monthly report 存在
# 4. tests/e2e/observability/trace-propagation/ の Go test が exit 0 完了
# 5. docs/40_運用ライフサイクル/portability-results.md に L6 1 回完了記録
# 6. ops/runbooks/RB-UPGRADE-001 / RB-DR-001〜004 / RB-CHAOS-001〜005 / RB-TEST-001 の 11 ファイル存在
# 7. docs/40_運用ライフサイクル/{conformance,e2e,dr-drill}-results.md の 3 ファイル存在
```

このスクリプトが exit 0 で完了したら Phase 1 完成と判定する。スクリプト自体も Phase 1 のタスクとして整備する。

### Phase 2 完成の機械検証

LitmusChaos が `operation` namespace で running / 5 シナリオ CRD が `infra/chaos/` 配下に配置 / Upgrade drill / DR drill / 観測性 E2E 5 検証 / DAST がそれぞれ直近 1 ヶ月以内に PASS している、を `tools/qualify/phase2-completion-check.sh` で機械検証する。

### Phase 3 完成の機械検証

OpenSSF Best Practices Badge Silver の API ステータス取得 / OSSF Scorecard JSON の Aggregate Score 取得 / `compliance/slsa-attestation/` ディレクトリの全 build artifact 件数確認、を `tools/qualify/phase3-completion-check.sh` で機械検証する。

### Phase 4 完成の機械検証

Phase 3 の検証 + OpenSSF Best Practices Badge Gold ステータス + CNCF Graduated 級慣行との比較表（手動レビュー要素を含むため完全自動化は困難、四半期手動レビュー）。

## 阻害要因とリスク

ロードマップは以下の阻害要因により実効性が損なわれる可能性がある。本書の Phase 移行条件は客観条件で書かれており、阻害要因が顕在化した場合は Phase 移行が機械的に保留される構造。

- **採用組織の出現タイミング** — Phase 1 → Phase 2 移行は contributor 確保 / sponsor がトリガで、起案者単独でコントロール不可。OSS 公開後 12 ヶ月で contributor 0 名の場合、Phase 1 完了状態を維持しつつ Phase 2 を凍結する判断を取る
- **GitHub Actions free tier 月間制限** — Phase 2 で nightly + weekly + conformance + flaky-report が並走すると月間制限を超える可能性。Phase 2 中盤で self-hosted runner / public repo 維持 / 有償移行のいずれかを ADR で決定する判断ポイント
- **LitmusChaos v3 schema 移行** — major version upgrade で CRD schema が変わるリスク。Renovate 自動 PR + 手動レビュー経路（採用初期で整備）で吸収するが、上流停滞時は ADR-TEST-004 の選択肢 B（Chaos Mesh）への移行 ADR を起票する余地を残す
- **SLSA L3 reproducible build 要件** — Phase 3 で Rust / Go の build 設定変更（`SOURCE_DATE_EPOCH` 固定 / build path 正規化 / dependency lock）が必要。tier1 / tier2 / tier3 / SDK の 4 系統で並行対応するため工数が膨張する可能性
- **CNCF Sandbox 申請の testing maturity 評価ハードル** — Phase 3 の出口判定で証跡（Sonobuoy + SLSA + Badge + Scorecard）が release artifact に統合同梱されている必要。証跡 1 つでも欠けると申請が遅延
- **Phase 移行条件の客観性が崩れるリスク** — sponsor 月予算 50 USD / 200 USD は OSS の sponsor 構造によって達成困難な場合がある。Phase 2 / 3 の移行条件を別軸（contributor 数 / 採用組織数）で代替する ADR 改訂を起案者の判断で実施する余地を残す

## 既存 ADR との対応表

本書の各 Phase 達成項目は ADR-TEST-001〜007 + 関連 ADR 12 本の決定を時系列に再構成したもの。各タスクは特定の ADR の「移行・対応事項」を履行する位置づけで、ADR と本書は双方向トレーサビリティを持つ。

| ADR | 本書での対応 |
|---|---|
| ADR-TEST-001（Test Pyramid + testcontainers） | Phase 0 完了、4 段フェーズ分離は ADR-TEST-007 self-relate-back で確定 |
| ADR-TEST-002（E2E 自動化） | Phase 1 タスク 1 / 2 / 3 / 7 で実装 |
| ADR-TEST-003（CNCF Conformance） | Phase 1 タスク 1 / 2 で skeleton、Phase 1 中盤で月次運用開始 |
| ADR-TEST-004（LitmusChaos） | Phase 1 タスク 8 で skeleton、Phase 2 で本格稼働 |
| ADR-TEST-005（Upgrade / DR drill） | Phase 1 タスク 8 で skeleton、Phase 2 で運用開始 |
| ADR-TEST-006（観測性 E2E） | Phase 1 タスク 4 / 5 で検証 1 + baseline、Phase 2 で 5 検証完備 |
| ADR-TEST-007（テスト属性タグ） | Phase 1 タスク 6（flaky-detector + tag lint）+ Phase 2 で weekly 運用 |
| ADR-CNCF-001（CNCF Conformance） | ADR-TEST-003 経由で Phase 1 / Phase 3 充足 |
| ADR-NET-001（CNI 選定） | Phase 1 で kind multi-node + Calico 構成、Phase 3 で Cilium portability |
| ADR-INFRA-001（kubeadm + Cluster API） | Phase 1 で staging cluster 構築準備、Phase 2 で Upgrade drill 月次 |
| ADR-DATA-001（CloudNativePG） | Phase 2 で barman-cloud restore drill（経路 C） |
| ADR-DATA-003（MinIO） | Phase 2 で DR backup target 検証（Velero 不採用、ADR-TEST-005 と整合） |
| ADR-OBS-001 / 002 / 003（Grafana LGTM / OTel / インシデント分類） | Phase 1 タスク 4 で trace 貫通検証、Phase 2 で 5 検証完備 |
| ADR-OPS-001（Runbook 標準化） | Phase 1 タスク 8 で 11 Runbook skeleton |
| ADR-POL-002（local-stack SoT） | Phase 1 タスク 1 で `--role e2e` / `--role conformance` 拡張 |
| ADR-DIR-002（infra 分離） | Phase 2 で `infra/chaos/` LitmusChaos CRD 配置 |
| ADR-TIER1-001（Go + Rust ハイブリッド） | Phase 1 タスク 6 で 4 言語属性タグ実装 |

## 更新ルール

本書は live document として、以下のタイミングで更新する。更新責務は起案者 + 採用初期以降は contributor の合意で分担する:

- **Phase 完了時**: 該当 Phase 節の「完成判定」が機械検証で PASS した時点で「達成済」マーカを付与し、commit に含める
- **Phase 内タスク完了時**: タスク粒度の達成状況は SHIP_STATUS.md で日々更新、本書は Phase 単位の俯瞰のみ
- **ADR-TEST 系列の改訂時**: ADR が Superseded / Deprecated に移行した場合、本書の対応表と該当 Phase 節を改訂
- **Phase 移行条件の客観性が崩れた時**: ADR 改訂で移行条件を変更した場合、本書も同 commit で同期更新
- **採用検討者からのフィードバック時**: testing maturity 評価で減点項目が判明した場合、本書の Phase 達成項目に追記

更新時は ADR と同様に **「## 関連」「## 参考資料」セクションへの relate-back** を維持し、双方向トレーサビリティを破壊しない。

## 関連

- ADR-TEST-001〜007（テスト戦略系列、`docs/02_構想設計/adr/`）
- ADR-TEST 系列が relate-back する既存 12 ADR（ADR-CNCF-001 / POL-002 / DIR-002 / DATA-001/003 / INFRA-001 / NET-001 / OBS-001/002/003 / OPS-001 / TIER1-001）
- IMP-CI 索引（`docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md`）
- 概要設計テスト戦略方式（`docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md`）
- 実装段階テスト層責務分界（`docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md`）
- SHIP_STATUS（`docs/SHIP_STATUS.md`）— 日々のタスク粒度進捗
- AUDIT（`docs/AUDIT.md`）— docs-orphan / code-orphan の継続監査
- INDEX（`docs/INDEX.md`）— docs 全体の探索動線

## 参考資料

- CNCF Project Maturity Levels: cncf.io/projects/
- OSSF Scorecard: scorecard.dev
- OpenSSF Best Practices Badge: bestpractices.coreinfrastructure.org
- SLSA: slsa.dev
- Sonobuoy: sonobuoy.io
- LitmusChaos: litmuschaos.io
- Velero（不採用、参照のみ）: velero.io
- Grafana LGTM: grafana.com/oss/
