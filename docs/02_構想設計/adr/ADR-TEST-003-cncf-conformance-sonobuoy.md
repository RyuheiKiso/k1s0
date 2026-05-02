# ADR-TEST-003: CNCF Conformance を Sonobuoy + kind multi-node + Calico で月次実行する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / セキュリティ担当 / コンプライアンス担当（採用初期）

## コンテキスト

ADR-CNCF-001 で「vanilla Kubernetes（CNCF Conformance 互換）を維持」と決定済で、その「移行・対応事項」に **「CNCF Conformance テスト（sonobuoy）を kind multi-node で定期実行する CI を整備（IMP-CI-CONF-*）」** が指示されている。しかし IMP-CI-CONF-* は docs/05_実装 配下で採番されておらず（`docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` に未記載）、ADR-CNCF-001 の決定が机上に留まっている状態である。

これは ADR-TEST-001（Test Pyramid + testcontainers）と並行で起票される必要がある。理由は ① L4 standard E2E（kind / Calico / 単一 cluster）と L5 conformance（本番 fidelity 重視）の責務分界を `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` で整備する必要がある、② ADR-CNCF-001 が指す「定期実行 CI」の頻度・cluster 構成・report 保管・採用検討者向け公開経路が未確定、の 2 点。なお L4 E2E の自動化経路 ADR は前 ADR-TEST-002 を撤回し、ADR-TEST-008（owner / user 二分構造）で再策定済である。

実用上の選択肢:

- **Sonobuoy on kind multi-node**（CNCF 公式の Conformance 取得ツール、kind cluster 上で動作可能）
- **Sonobuoy on production cluster**（リリース時点で production cluster は存在しない、採用組織の手元）
- **Sonobuoy 不採用、Conformance テスト省略**（ADR-CNCF-001 の決定を空洞化）
- **自前 conformance test 実装**（CNCF e2e test スイートを自分で書き直す）

実行頻度の選択肢:

- **月次**（schedule trigger 月初、所要 1〜2 時間）
- **週次**（負荷高、artifact が 4 倍蓄積）
- **release tag ごと**（リリース頻度に依存、頻度予測が立たない）
- **PR ごと**（時間予算超過、ADR-TEST-001 の PR 5 分制約と矛盾）

cluster の CNI 選択（kind multi-node に限定）:

- **Calico**（ADR-NET-001 で「kind multi-node = Calico」と既存決定）
- **Cilium**（ADR-NET-001 で「production = Cilium」だが kind 上では制約あり）
- **kindnet**（ADR-NET-001 で「kind single-node = kindnet」、multi-node の Conformance には不適）

選定では以下を満たす必要がある:

- **ADR-CNCF-001 の移行事項を充足する**（IMP-CI-CONF-* の起票経路を確立する）
- **ADR-NET-001 と整合する**（kind multi-node = Calico）
- **L4 / L5 責務分界を尊重する**（L5 conformance は別 cluster + 本番 fidelity 重視）
- **GitHub Actions runner（4 core / 14 GB / 6 時間）で動作可能**
- **採用検討者が結果を確認できる経路**（月次 report の公開）
- **個人 OSS の運用工数を圧迫しない頻度**

## 決定

**CNCF Conformance テストは Sonobuoy v0.57+ を kind multi-node cluster + Calico CNI 上で `--mode certified-conformance` で月次実行する。** `tools/local-stack/up.sh --role conformance`（新設）で cluster を起動し、`.github/workflows/_reusable-conformance.yml`（新設）を `.github/workflows/conformance.yml`（新設、月次 schedule + workflow_dispatch）から呼ぶ構造とする。

### 1. cluster 構成

`tools/local-stack/up.sh --role conformance` を新設し、以下構成を起動する:

- kind cluster: control-plane 1 + worker 3（Sonobuoy のデフォルト推奨構成）
- CNI: Calico（ADR-NET-001 の「kind multi-node = Calico」と整合）
- StorageClass: local-path-provisioner（kind デフォルト）
- DNS: CoreDNS（kind 同梱）
- 追加コンポーネント: なし（Conformance テストは vanilla K8s 機能のみ検証、フルスタック（Argo CD / Istio / Dapr 等）の起動は **不要**）

L4 E2E 用の cluster 構成（フルスタック）と `--role conformance`（本 ADR、vanilla のみ）は責務が異なることを `tools/local-stack/README.md` で明文化する。L4 用の起動経路は ADR-TEST-008 で `tools/e2e/owner/up.sh` / `tools/e2e/user/up.sh` の専用スクリプトとして再確定済（`--role` 引数空間とは物理分離）。

### 2. Sonobuoy 実行

`sonobuoy run --mode certified-conformance --wait` で全 Conformance テスト（Kubernetes e2e の `Conformance` ラベル付き約 500 テスト）を実行する。所要時間は約 60〜120 分。`--wait` で完了まで待ち、`sonobuoy retrieve` で結果 tar.gz を取得、`sonobuoy results <tarball>` で human readable summary を生成する。

実行は `tools/qualify/conformance/run.sh`（新設）でラップし、以下を順次実行:

1. kind cluster ready 確認（`kubectl wait --for=condition=Ready node --all --timeout=300s`）
2. `sonobuoy run --mode certified-conformance --wait`
3. `sonobuoy retrieve > results.tar.gz`
4. `sonobuoy results results.tar.gz > summary.md`
5. failed test が 0 でなければ exit 1
6. `sonobuoy delete --wait` で cluster をクリーンアップ

### 3. CI 統合

`.github/workflows/_reusable-conformance.yml`（新設）の構造:

```yaml
name: _reusable-conformance
on:
  workflow_call:
    inputs:
      timeout_minutes:
        required: false
        type: number
        default: 180
permissions:
  contents: read
jobs:
  conformance:
    runs-on: ubuntu-latest
    timeout-minutes: ${{ inputs.timeout_minutes }}
    steps:
      - uses: actions/checkout@v4
      - name: setup kind + calico
        run: ./tools/local-stack/up.sh --role conformance
      - name: install sonobuoy
        run: |
          curl -L https://github.com/vmware-tanzu/sonobuoy/releases/download/v0.57.3/sonobuoy_0.57.3_linux_amd64.tar.gz | tar xz
          sudo mv sonobuoy /usr/local/bin/
      - name: run conformance
        run: ./tools/qualify/conformance/run.sh
      - name: collect artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: sonobuoy-results-${{ github.run_id }}
          path: |
            tests/.conformance/results.tar.gz
            tests/.conformance/summary.md
          retention-days: 90
```

呼び出し元 `.github/workflows/conformance.yml`（新設）:

```yaml
name: conformance
on:
  schedule:
    - cron: "0 18 1 * *"  # 毎月 1 日 03:00 JST = 18:00 UTC
  workflow_dispatch:
permissions:
  contents: read
jobs:
  conformance:
    uses: ./.github/workflows/_reusable-conformance.yml
    with:
      timeout_minutes: 180
```

`pr.yml` および将来再導入される nightly workflow からは Conformance を呼ばない。月次専用の独立 workflow とする。

### 4. report 保管

Conformance 結果は `tests/.conformance/<YYYY-MM>/sonobuoy-results.tar.gz` + `summary.md` として **12 ヶ月分を git LFS で版管理**する。これにより採用検討者が時系列で「k1s0 が Conformance を継続的に取得しているか」を確認可能になる。

`docs/40_運用ライフサイクル/conformance-results.md`（新設）に月次サマリ（PASS / FAIL / 失敗テスト一覧 / 対応 issue）を記載し、最新 release artifact（ADR-TEST-001 release artifact 中心モデル）に同梱する。

### 5. IMP-CI-CONF-* の起票経路確立

ADR-CNCF-001 の「移行・対応事項」で cite されている IMP-CI-CONF-* について、本 ADR で具体内容を ADR レベルで確定する:

- IMP-CI-CONF-001: Sonobuoy v0.57+ を `--mode certified-conformance` で実行
- IMP-CI-CONF-002: kind multi-node（control-plane 1 + worker 3）+ Calico CNI 構成
- IMP-CI-CONF-003: 月次実行（cron 毎月 1 日 03:00 JST）+ workflow_dispatch
- IMP-CI-CONF-004: results.tar.gz + summary.md を 12 ヶ月分 git LFS で版管理
- IMP-CI-CONF-005: failure 時に `docs/40_運用ライフサイクル/conformance-results.md` で公開、起案者に notify

実装段階の詳細（IMP-CI-CONF-001〜005 の正典記述）は `docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` への追記を別 commit / 別 PR で実施する（本 ADR commit ではスコープ外、ADR レベルの確定のみ）。

### 6. QUALIFY-POLICY.md 同時整備

**`docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` の整備**を本 ADR 起票と同時に実施する。当面の射程は「L4 standard E2E（kind / Calico / フルスタック / nightly）と L5 conformance（kind multi-node / Calico / vanilla / 月次）の責務分界」のみとし、他の topic（Phase 移行 / OSSF Scorecard マッピング / SLSA / 成熟度ロードマップ等）は後続 ADR-TEST-* で順次拡張する。なお L4 E2E 側の正典 ADR は ADR-TEST-008（owner / user 二分構造）が担う。

## 検討した選択肢

### 選択肢 A: Sonobuoy on kind multi-node + Calico、月次実行（採用）

- 概要: CNCF 公式 Sonobuoy CLI を kind cluster（control-plane 1 + worker 3、Calico CNI）上で `--mode certified-conformance` で月次実行
- メリット:
  - **ADR-CNCF-001 の「移行・対応事項」と完全整合**（kind multi-node で定期実行する CI を整備）
  - ADR-NET-001（kind multi-node = Calico）と整合
  - GitHub Actions runner（4 core / 14 GB / 6 時間）で動作可能
  - kind の起動が約 30 秒、Calico install + Sonobuoy 実行で合計 60〜120 分、6 時間制約内に余裕
  - 月次頻度で個人 OSS の運用工数（artifact 蓄積 / triage）を圧迫しない
  - Sonobuoy v0.57+ は CNCF Sandbox プロジェクトとして継続メンテナンス
- デメリット:
  - kind cluster は本番 cluster と完全には fidelity 一致しない（kindnet ではなく Calico を使うことで CNI fidelity は確保するが、CSI / LB / multi-AZ は再現できない）
  - 月次頻度のため Conformance ステータス変化の検出が最大 30 日遅延
  - Sonobuoy の Conformance テスト 500+ ケースのうち、kind の制約で skip される項目が一部存在（採用検討者向けの注釈が要る）

### 選択肢 B: Sonobuoy on production cluster

- 概要: production の kubeadm cluster で Sonobuoy を実行
- メリット:
  - 本番 fidelity が 100%（実 cluster 上で取得した Conformance 結果）
  - 採用検討者が「production と同じ cluster で取った Conformance」と理解しやすい
- デメリット:
  - **リリース時点で production cluster が存在しない**（採用組織が立ち上げる cluster であり、起案者側にはない）
  - production で Conformance テストを走らせると 500 テストが namespace を汚染し、業務影響リスクがある
  - 「k1s0 が Conformance を取っている」を release artifact で示せない（cluster が採用組織側にあるため、結果を起案者が公開できない）

### 選択肢 C: Sonobuoy 不採用、Conformance テスト省略

- 概要: Conformance テストを CI で実施せず、ADR-CNCF-001 の宣言のみで終わる
- メリット:
  - 実装工数ゼロ
  - GitHub Actions runner のリソース消費なし
  - 月次 artifact 蓄積なし
- デメリット:
  - **ADR-CNCF-001 の「移行・対応事項」を未充足のまま放置**（決定が空洞化）
  - 採用検討者が「k1s0 は CNCF Conformance を宣言だけしている、実証なし」と判定し、testing maturity 評価が低下
  - Phase 移行で CNCF Sandbox 申請する際、Conformance 証跡を持たない状態が判明し、申請が通らない
  - upstream Kubernetes バージョン更新時に互換性破綻を検出する経路がなくなり、採用組織のフィードバックまで気づかない

### 選択肢 D: 自前 conformance test 実装

- 概要: Sonobuoy を使わず、Kubernetes e2e test スイートから Conformance ラベル付きテストを抽出して自前で実行する shell script を書く
- メリット:
  - Sonobuoy 依存なし、全制御を自分が持つ
  - kind 制約で skip される項目を明示的に列挙できる
- デメリット:
  - **Conformance テストスイート（500+ ケース）を維持する工数が爆発**: Kubernetes upstream の e2e test スイートは年 4 回更新され、追従が必須。Sonobuoy がこれを吸収しているため、自前実装は車輪の再発明
  - 「CNCF 公式の Conformance」と認められない可能性（CNCF が認証するのは Sonobuoy 由来の結果）
  - 採用検討者から見て「自前テストは信頼できるか」の説明工数が継続発生

## 決定理由

選択肢 A（Sonobuoy on kind multi-node + Calico、月次実行）を採用する根拠は以下。

- **ADR-CNCF-001 / ADR-NET-001 との完全整合**: ADR-CNCF-001 が「Sonobuoy + kind multi-node で定期実行する CI を整備」と指示、ADR-NET-001 が「kind multi-node = Calico」と決定済。選択肢 A はこの 2 つの既存 ADR を字義通り履行する唯一の選択肢
- **個人 OSS の運用工数との整合**: 月次頻度（年 12 回）は artifact 蓄積（12 ヶ月分 = 12 件 × 約 100 MB = 1.2 GB）が git LFS で扱える規模で、月次 triage 工数も起案者一人で吸収可能。週次は 4 倍負荷で運用破綻、release tag ごとは頻度予測不能、PR ごとは ADR-TEST-001 の CI 時間予算と矛盾
- **採用検討者向け証跡の継続性**: 12 ヶ月分の月次 report が時系列で版管理されることで、採用検討者が「k1s0 が Conformance を継続取得しているか」を時系列で確認できる。release artifact 中心モデル（ADR-TEST-001）における Conformance 軸の証跡を物理的に補強
- **GitHub Actions runner との整合**: kind cluster + Calico install + Sonobuoy 実行で合計 60〜120 分、Actions runner の 6 時間制約に余裕で収まる。選択肢 D（自前実装）は実装工数で破綻、選択肢 B（production cluster）は cluster 自体が無い
- **退路の確保**: 選択肢 A は kind cluster が将来 multi-node 構成を変更する場合（例: control-plane 3 へ HA 化）にも `tools/local-stack/up.sh --role conformance` の引数追加で吸収可能。Sonobuoy のバージョン追従も Conformance テストスイート側が CNCF 標準なので、上流の更新が破綻リスクを最小化
- **IMP-CI-CONF-* docs-orphan の解消経路**: 選択肢 A の決定により、IMP-CI-CONF-001〜005 を ADR レベルで起票し、`docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` への追記を別 commit で実施する経路が確立。ADR-CNCF-001 の cite が docs-orphan 状態を脱する

## 影響

### ポジティブな影響

- ADR-CNCF-001 の「移行・対応事項」が本 ADR で具体化され、CNCF Conformance の継続取得経路が確立する
- 12 ヶ月分の月次 Conformance report が git LFS で版管理され、採用検討者が時系列で testing maturity を評価できる
- upstream Kubernetes バージョン更新時の互換性破綻が月次 Conformance で早期検出される
- IMP-CI-CONF-001〜005 が ADR レベルで確定し、`docs/05_実装/30_CI_CD設計/` への展開経路が開く
- `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` が L4 / L5 責務分界を最低限の射程で整備される
- L4 standard E2E（kind / Calico / フルスタック / nightly）と L5 conformance（kind multi-node / Calico / vanilla / 月次）の cluster 構成・実行頻度・責務分界が ADR で正典化される

### ネガティブな影響 / リスク

- 月次 Conformance workflow の GitHub Actions runner リソース消費（4 core / 14 GB / 60〜120 分 / 月 1 回）が継続的に発生。Public repo では無制限だが private repo 移行時は予算管理が要る
- kind cluster は CSI / LB / multi-AZ の本番 fidelity が不足し、Conformance テスト 500+ のうち skip される項目が一部存在。`docs/40_運用ライフサイクル/conformance-results.md` で skip 項目を採用検討者向けに明示する必要
- 月次頻度のため Conformance ステータス変化の検出が最大 30 日遅延。Kubernetes minor version 更新（年 3 回）の影響を月次 1 回しか確認できないリスクは ADR-CNCF-001 の「四半期パッチ追従」運用と整合させる
- Sonobuoy v0.57+ の CNCF Sandbox プロジェクト継続性に依存。プロジェクトが停滞した場合、選択肢 D（自前実装）への移行を Phase 移行で検討する余地を残す（リリース時点では発生していない）
- git LFS の月次 100 MB 蓄積で年 1.2 GB / 5 年 6 GB の累積。GitHub free tier の LFS 制限（1 GB）を超えるため、release asset への昇格 + 古い report の cold storage 移行を採用初期で整備する必要

### 移行・対応事項

- `tools/local-stack/up.sh` に `--role conformance` を追加し、kind cluster（control-plane 1 + worker 3）+ Calico CNI のみを起動する経路を整備（フルスタックは起動しない、L4 E2E 用は ADR-TEST-008 で `tools/e2e/{owner,user}/up.sh` の専用スクリプトとして正典化、本 role と物理分離）
- `tools/qualify/conformance/run.sh` を新設し、Sonobuoy 実行 → retrieve → results 整形 → cleanup を冪等 shell script として実装
- `.github/workflows/_reusable-conformance.yml` を新設し、`workflow_call` で `timeout_minutes` を inputs として受け取る構造で実装
- `.github/workflows/conformance.yml` を新設し、`schedule: 0 18 1 * *`（月初 03:00 JST）+ `workflow_dispatch` で `_reusable-conformance.yml` を呼ぶ
- `tests/.conformance/<YYYY-MM>/` ディレクトリ構造を整備、git LFS で `*.tar.gz` を track（`.gitattributes` 更新）
- `docs/40_運用ライフサイクル/conformance-results.md` を新設し、月次 PASS / FAIL / 失敗テスト一覧 / kind 制約による skip 項目を散文 + 表で記載
- `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` を新設し、L4 / L5 責務分界（cluster 構成 / CNI / フルスタック有無 / 実行頻度 / fidelity 目標）を散文で明文化
- IMP-CI-CONF-001〜005 を `docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` に追記する作業を別 commit / 別 PR で実施（本 ADR の射程外、ADR レベルの確定のみ完了）
- ADR-CNCF-001 の「移行・対応事項」行 105 で cite されている `IMP-CI-CONF-*` が本 ADR で確定したことを ADR-CNCF-001 の relate-back で追記
- ADR-NET-001 の「帰結」セクションに「kind multi-node + Calico は L5 conformance（ADR-TEST-003）でも採用」を追記する relate-back 作業
- 採用初期で git LFS の累積容量管理を Runbook 化（`ops/runbooks/RB-CI-001-conformance-artifact-rotation.md`、ADR-OPS-001 8 セクション形式）

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— L5 conformance が Test Pyramid の orthogonal 軸として位置づけられる根拠
- ADR-TEST-008（e2e owner / user 二分構造）— L4 と L5 の責務分界、L4 = `tools/e2e/{owner,user}/up.sh` 専用スクリプト / L5 = `tools/local-stack/up.sh --role conformance` の物理分離を本 ADR と整合
- ADR-CNCF-001（vanilla K8s + CNCF Conformance 維持）— 本 ADR が「移行・対応事項」を充足
- ADR-NET-001（CNI 選定）— kind multi-node = Calico の整合
- ADR-INFRA-001（kubeadm + Cluster API）— production cluster 構成、kind との fidelity 差認識
- ADR-POL-002（local-stack を構成 SoT に統一）— `--role conformance` の SoT 拡張
- ADR-OPS-001（Runbook 標準化）— RB-CI-001 の形式根拠
- IMP-CI-CONF-001〜005（本 ADR で確定、`docs/05_実装/30_CI_CD設計/90_対応IMP-CI索引/01_対応IMP-CI索引.md` への展開は別 commit）
- Sonobuoy: sonobuoy.io
- CNCF Certified Kubernetes Conformance Program: cncf.io/training/certification/software-conformance
- Kubernetes e2e test framework: kubernetes.io/docs/reference/setup-tools/kubeadm/kubeadm-conformance/
- 関連 ADR（採用検討中）: ADR-TEST-004（Chaos ツール選定）/ ADR-TEST-005（Upgrade / DR drill）/ ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）/ ADR-TEST-008（e2e owner/user 二分）/ ADR-TEST-009（観測性 E2E）/ ADR-TEST-010（test-fixtures）/ ADR-TEST-011（release tag ゲート）
