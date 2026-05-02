# ADR-TEST-003: テストピラミッドを L0–L10 の 11 層に階層化し、層別 gate と責任分界を正典化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / QA リード

## コンテキスト

ADR-TEST-001 で qualify 全層をローカル `make qualify-release` に必須化することを決定した。しかし「全層」が具体的に何を指すかは未定であり、層の粒度・責任分界・gate 配置を ADR で正典化しないと、書きやすい層（unit / integration）に偏ったテスト群が積み上がり、本番 fidelity を担保する層（conformance / chaos / upgrade / DR）が「いつかやる」と先送りされるリスクがある。これは CLAUDE.md ポリシー「未来への先送りは許さない」と全面衝突する。

既存 `tests/e2e/` は tier1→3 を貫通する単一 Go module として雛形 stub（`t.Skip("PHASE: release-initial")`）のみが置かれており、層分解されていない。kind cluster 上で E2E を回す前提だけが README に記述されているが、conformance（vanilla K8s 上での Sonobuoy）や DR（etcd PITR / Velero）のような層は未定義のままである。リリース時点でこの状態のまま release tag を切ると、採用検討者が `tests/` ディレクトリを覗いた瞬間に「testing maturity が薄い」と判定する。

加えて、採用検討者は OSSF Scorecard の `Fuzzing` / `CI-Tests` / `SAST` / `License`、CNCF Sandbox 採用基準の testing maturity 軸、OpenSSF Best Practices Badge の Silver / Gold 要件で「層別に網羅されたテストがあるか」を評価する。これらの外部評価軸は層を 11 段階で分けて要求しているわけではないが、CNCF Graduated 級 OSS（Kubernetes / Istio / Cilium / ArgoCD / Knative）が実際に持っているテスト体制を逆算すると、11 層程度に分解されているのが共通項である。

層の選定では以下を満たす必要がある:

- **責任分界が一意**（テストを書く時に「これは L4 か L5 か」で迷わない）
- **gate 配置が一意**（pre-commit / pre-push / `make qualify` / `make nightly` / `make qualify-release` / `make qualify-soak` のどこで走るかが層から自動決定する）
- **所要時間の予測可能性**（各層の実行時間が見積もり可能で、開発者が「pre-push に何分待つか」を予期できる）
- **層別環境の独立性**（kind / multipass kubeadm / KWOK / 実 cloud のどれが要るかが層から自動決定する）
- **採用検討者が層別に testing maturity を評価できる**（CNCF Graduated 級 OSS の慣行と概念的に一致する）
- **書き手の認知負荷が爆発しない**（11 層は CNCF Graduated 級 OSS の慣行と整合する最大値で、これ以上分割すると個人 OSS の運用工数を破綻させる）

層分解の選定は **two-way door に近いが移行コストが高い** 決定である。後から層を増減すると `Makefile` target / `tests/e2e/<layer>/` ディレクトリ構造 / qualify report の JSON schema / 採用検討者向け doc の総書き換えが発生する。リリース時点で確定させ、Phase 移行で層数が増減しないことを構造的に保証する必要がある。

## 決定

**テストピラミッドを以下の 11 層 (L0–L10) に階層化し、`tests/e2e/<layer>/` のディレクトリ単位で物理的に分離する。** 各層は独立 Go module（または Rust crate）として閉じ、build tag による条件コンパイルではなくディレクトリ隔離で層間の干渉を防ぐ。

| 層 | 内容 | 環境 | gate（trigger） | 所要 | release blocking |
|----|------|------|-----------------|------|-----------------|
| **L0** | contract test（buf breaking、proto schema 互換、SemVer 強制） | local プロセス | `pre-commit` | < 10 秒 | 必須 |
| **L1** | unit test（Rust: cargo nextest、Go: go test、TypeScript: vitest） | local プロセス | `pre-commit` | < 60 秒 | 必須 |
| **L2** | integration test（同一プロセス内 / testcontainers で外部依存を Docker 化） | local + Docker | `pre-commit` | < 5 分 | 必須 |
| **L3** | smoke E2E（kind 単一クラスタ、tier1→3 の happy path 1 本のみ） | kind | `pre-push` | < 5 分 | 必須 |
| **L4** | standard E2E（kind、全シナリオを並列実行） | kind | `make qualify` | < 30 分 | 必須 |
| **L5** | conformance E2E（multipass で立てた 3-node kubeadm cluster、Sonobuoy + 本番 fidelity） | multipass kubeadm | `make qualify-release` | 1〜2 時間 | 必須 |
| **L6** | portability E2E（EKS / GKE / AKS） | 実 cloud | Phase 0: 手動 1 回 / Phase 3: 自動 | 数時間 | release qualify report に同梱 |
| **L7** | chaos E2E（kind + Chaos Mesh、pod-kill / network-partition / disk-fill / clock-skew） | kind + Chaos Mesh | `make nightly` + `make qualify-release` | 30 分〜2 時間 | 必須 |
| **L8** | scale / soak E2E（KWOK 1000 node、k6 24h continuous load、月次） | KWOK + k6 | `make qualify-soak`（月次） | 24 時間+ | release qualify report に同梱 |
| **L9** | upgrade E2E（N-2 → N → N+1 のローリング） | multipass kubeadm | `make qualify-release` | 1〜2 時間 | 必須 |
| **L10** | DR E2E（etcd PITR / Velero restore / region failover シミュレーション） | multipass kubeadm + minio + Velero | `make qualify-release` | 1〜2 時間 | 必須 |

**L6 portability の例外的扱い**: ADR-TEST-001 Phase 表で「Phase 0 では手動 1 回のみ実走、Phase 3 で自動化」と決定済。リリース時点では起案者が EKS / GKE / AKS のいずれかで `make qualify-portability-once` を 1 回実行し、その JSON + Markdown report を release artifact に同梱する。Phase 1〜2 の間は手動再走の頻度をリリースごとに 1 回維持する（Phase 3 の自動化以前でも証跡は途切れない）。

ディレクトリ構造は以下とし、各層が独立 module として build / test できるようにする:

```text
tests/
├── README.md                      # 11 層の俯瞰、gate 対応、層別 README へのリンク
├── e2e/
│   ├── L0_contract/              # proto / buf / openapi の schema 互換
│   ├── L1_unit/                  # 各 src/* 配下の unit test とは別の cross-module unit
│   ├── L2_integration/           # testcontainers + 同一プロセス integration
│   ├── L3_smoke/                 # 既存 tests/e2e/scenarios/ を移管
│   ├── L4_standard/              # 全シナリオ並列、kind 専用
│   ├── L5_conformance/           # Sonobuoy + 本番 fidelity 検証
│   ├── L6_portability/           # cloud 実機検証、Phase 0 は手動 1 回
│   ├── L7_chaos/                 # Chaos Mesh experiment YAML + Go 検証コード
│   ├── L8_scale/                 # KWOK + k6 シナリオ
│   ├── L9_upgrade/               # N-2 → N → N+1 シナリオ
│   └── L10_dr/                   # Velero / etcd PITR / region failover
├── perf/                         # 性能 regression（L1/L2 と独立、p99 budget assertion）
└── fuzz/                         # cargo fuzz / go-fuzz（L1 と独立、coverage 集計）
```

`Makefile` には gate ごとに以下の target を新設する:

- `make qualify-pre-commit` — L0 + L1 + L2 を並列実行（pre-commit hook から呼ばれる）
- `make qualify-pre-push` — L3 を実行（pre-push hook から呼ばれる）
- `make qualify` — L0–L4 をシーケンシャル実行（開発者が PR 前に手動で叩く）
- `make qualify-nightly` — L7 を毎晩実行（cron + `notify-send` で結果を通知）
- `make qualify-release` — L0–L5 + L7 + L9 + L10 を release tag 直前に必須実行（`tools/release/cut.sh` から呼ばれる、ADR-TEST-004 で詳細）
- `make qualify-soak` — L8 を月次実行（個別 cron）
- `make qualify-portability-once` — L6 を Phase 0 では手動 1 回実行

各層は `tests/e2e/<layer>/RESOURCES.md` を持ち、必要 RAM / CPU / 所要時間 / 前提環境を宣言する。これにより採用検討者が「自分のマシンで L3 だけ回したい」「L8 は無理だから skip したい」を選択できる。

## 検討した選択肢

### 選択肢 A: 4 層モデル（古典的 unit / integration / e2e / manual）

- 概要: テストピラミッドの古典形（Mike Cohn 1990s）に準拠し、unit / integration / e2e / manual の 4 層で分割する
- メリット:
  - 業界で最も知名度が高く、新規参画者の理解が早い
  - Makefile target も `make test-unit` / `make test-integration` / `make test-e2e` / `make test-manual` の 4 つで済む
  - 認知負荷が低い
- デメリット:
  - **conformance / chaos / DR / upgrade / scale / portability が「e2e」に潰れる**。これらは互いに環境・所要時間・gate 配置が大きく異なるため、同一 target で扱うと「e2e が遅い、何が走っているのか分からない」という典型的アンチパターンに陥る
  - 採用検討者が「testing maturity が層別に評価できない」（OSSF Scorecard / CNCF Sandbox 採点との不整合）
  - CNCF Graduated 級 OSS の慣行と乖離し、k1s0 が CNCF Sandbox 申請する場合に testing 体系の説明で不利

### 選択肢 B: 7 層モデル（Google Test Pyramid 拡張 + production layer）

- 概要: Google の Software Engineering at Google で示される拡張ピラミッド（unit / integration / e2e / smoke / canary / production probe / synthetic）を採用
- メリット:
  - 知名度が一定あり、Google 由来として説得力がある
  - smoke / canary / production probe を分離しているため、L3 / L7 相当の概念は表現できる
- デメリット:
  - **portability / DR / upgrade / scale が unspecified**。Google 内部では別系統で扱われているがオープンに正典化されていない
  - canary / production probe は production 環境前提で、リリース時点 / 採用初期にはまだ production が無い
  - L8 scale soak / L10 DR drill のような「故意に壊す」系の層を表現できない

### 選択肢 C: 11 層モデル（採用、CNCF Graduated 慣行）

- 概要: Kubernetes / Istio / Cilium / ArgoCD / Knative が実際に持つテスト体制を逆算し、L0–L10 の 11 層に分解する
- メリット:
  - **責任分界が一意**: 各層の環境・gate・所要時間・本番 fidelity が独立、テストを書く時の置き場所が機械的に決まる
  - **gate 配置が層から自動決定**: 開発者が「これは pre-push か qualify-release か」を層番号から逆引きできる
  - **採用検討者の評価軸と整合**: CNCF Sandbox 採用基準・OSSF Scorecard・OpenSSF Best Practices Badge が要求する testing maturity の各項目が層に対応する
  - **CNCF Graduated 級 OSS との比較可能性**: 採用検討者が「Cilium と比べて k1s0 は L7 がどう違うか」を概念ベースで議論できる
- デメリット:
  - 11 層は粒度が細かく、リリース時点で全層実装する工数負担が大きい
  - 層が多いと開発者の認知負荷が上がる（ただしディレクトリ構造で物理分離するため、混乱はディレクトリ navigation で吸収可能）
  - 層境界が曖昧なテスト（例: integration と smoke の境界）を書く時に判断が要る

### 選択肢 D: 階層化なし（書きたい人が書きたい時に書く）

- 概要: テストピラミッドを定義せず、`tests/` 配下に開発者が好きにテストを置く
- メリット:
  - 規約整備工数ゼロ、初期の開発速度が最大
  - 開発者の自由度が高い
- デメリット:
  - **書きやすい層に偏る**: unit / integration ばかり積み上がり、conformance / chaos / DR が永遠に「いつかやる」になる
  - **採用検討者が testing maturity を評価できない**: `tests/` を覗いて何が網羅されているか不明
  - **gate 配置が成立しない**: pre-push で何を走らせるべきか決まらず、全テストを実行して時間がかかるか、何も走らせず品質崩壊するかの二択になる
  - CLAUDE.md ポリシー「未来への先送りは許さない」と全面衝突

## 決定理由

選択肢 C（11 層モデル）を採用する根拠は以下。

- **責任分界の一意性が他選択肢と桁違い**: 11 層は CNCF Graduated 級 OSS の慣行から逆算した粒度で、各層の環境・所要時間・本番 fidelity がほぼ重複しない。新規テストを書く時に「これは L4 か L5 か」で迷う場面は、層境界の defining property（L5 は本番 fidelity を取りに行く層、L4 は kind で完結する層）から機械的に決まる。選択肢 A / B では多くのテストが「e2e」に潰れて判断が属人化する
- **gate 配置の自動決定**: 各層に gate を 1:1 対応させているため、開発者が `make qualify-pre-push` を叩いた時に何が走るかが層番号から逆引きできる。Makefile target の責務分担が明確で、bash script 内で層をフィルタする条件分岐を書かなくて済む。選択肢 D ではこの構造が成立しない
- **採用検討者の評価軸との整合**: OSSF Scorecard `CI-Tests` / `Fuzzing`、CNCF Sandbox 採用基準の testing maturity、OpenSSF Best Practices Badge Silver の "Test suite covers a wide variety of test types" などが、L0–L10 のどれに該当するかを `docs/governance/QUALIFY-POLICY.md` で 1:1 マッピング可能。選択肢 A / B では「e2e」に潰れた層が外部評価軸の複数項目に分散対応する形になり、説明工数が膨張する
- **CNCF Graduated 級 OSS との比較可能性**: Cilium が L7 chaos に Litmus、L8 scale に kubemark を使っているのに対し、k1s0 は Chaos Mesh / KWOK を使う、という比較が層単位で議論可能。選択肢 A / B では Cilium と k1s0 の「e2e」が同じ概念を指していないため、比較が成立しない
- **L6 portability の例外的扱いの正当化**: ADR-TEST-001 の Phase 表で portability のみが「Phase 0 で手動 1 回」と決まっているが、これを L6 として独立層に切り出すことで、Phase 0〜2 では `make qualify-portability-once` という固有 target を持ち、Phase 3 で自動化に統合される、という Phase 移行が層単位で局所化できる。選択肢 A / B では portability が「e2e」または「production probe」に紛れ込み、Phase 移行の局所性が失われる
- **個人 OSS の運用工数とのバランス**: 11 層という粒度は、CNCF Graduated 級 OSS の慣行と整合しつつ、それ以上分割しても個人 OSS の運用工数では維持できない最大値である。例えば 13 層に分けて performance / fuzzing を独立層に昇格する案もあったが、これらは層ではなく orthogonal な軸（`tests/perf/` `tests/fuzz/` として並列配置）にすることで、層数を 11 に維持した

## 影響

### ポジティブな影響

- 各層の責任分界が ADR で正典化され、新規テストを書く時の置き場所判断が機械化される（「これは L4 か L5 か」を defining property から逆引きできる）
- gate 配置が層から自動決定し、`pre-commit` / `pre-push` / `make qualify` / `make qualify-release` / `make qualify-soak` の責務分担が `Makefile` で 1:1 対応する
- 採用検討者が `tests/README.md` の 11 層俯瞰表を見るだけで testing maturity を評価でき、OSSF Scorecard / CNCF Sandbox 採用基準への対応説明が `docs/governance/QUALIFY-POLICY.md` で機械化される
- Phase 移行（ADR-TEST-001）の局所性が層単位で保証される。例えば L6 portability のみが Phase 3 で自動化対象になる、という Phase 表が層番号で記述できる
- 各層が `tests/e2e/<layer>/RESOURCES.md` で必要 RAM / CPU を宣言するため、採用検討者が「自分のマシンで L3 まで回す」「L7 / L8 は skip する」を選択できる試走柔軟性が確保される
- ディレクトリ単位の物理分離により、層間の依存・干渉が原理的に発生しない（build tag による条件コンパイルでは、誤って跨いだ依存が混入する）

### ネガティブな影響 / リスク

- 11 層の全層をリリース時点で実装する工数負担が大きい。特に L7 chaos / L8 scale / L10 DR は単独で 5〜10 人日規模の構築工数が発生する。リリース時点での実装範囲は ADR-TEST-001 の射程（L3/L4/L5/L7/L9/L10 を release blocking、L6/L8 は補助）と整合させ、L0–L2 は既存 `tests/` の延長で吸収する
- 11 層という多さが新規参画者の認知負荷を上げる。`tests/README.md` の 11 層俯瞰表 + 各層の README + `docs/governance/QUALIFY-POLICY.md` の三段で説明動線を整備し、新規参画者が層を覚えなくても「やりたいこと → 該当層」が逆引きできるようにする
- 層境界が曖昧なテスト（例: integration と smoke、smoke と standard）を書く時の判断が要る。`tests/e2e/<layer>/README.md` に「この層に属するテストの defining property」を散文で書き、判断を ADR ではなく層別 README に委譲する
- 既存 `tests/e2e/scenarios/tenant_onboarding_test.go` を `tests/e2e/L3_smoke/` に移管する作業が発生する。この移管は ADR-TEST-004 の二層 E2E 構造化と合わせて行う
- KWOK 1000 node（L8）/ multipass 3-node kubeadm（L5/L9/L10）/ Chaos Mesh（L7）/ Velero + minio（L10）の構築運用が個人 OSS の起案者一人に乗る。これは ADR-TEST-002 のハードウェア最低要件で物理的に支えるが、起案者不在時に協力者が立ち上げる手順を Runbook 化（`ops/runbooks/RB-OPS-002-qualify-cluster-bootstrap.md`）する必要

### 移行・対応事項

- `tests/` ディレクトリ構造を 11 層 + perf + fuzz の 13 ディレクトリに再編する（既存 `tests/e2e/scenarios/` を `tests/e2e/L3_smoke/` に移管）
- 各層に `tests/e2e/<layer>/README.md` と `tests/e2e/<layer>/RESOURCES.md` を新設し、defining property / 必要リソースを散文で記述する
- 各層を独立 Go module（または Rust crate）として `go.mod` / `Cargo.toml` を分離する。L0–L2 は既存リポジトリ構造に合わせて部分的に統合可能
- `Makefile` に層別 gate target を新設（`qualify-pre-commit` / `qualify-pre-push` / `qualify` / `qualify-nightly` / `qualify-release` / `qualify-soak` / `qualify-portability-once`）
- `tests/README.md` を新設し、11 層俯瞰表 + gate 対応 + 各層 README へのリンクを散文で公開
- `docs/governance/QUALIFY-POLICY.md` に「k1s0 のテスト層と OSSF Scorecard / CNCF Sandbox 採用基準 / OpenSSF Best Practices Badge の項目との 1:1 マッピング」を整備
- `ops/runbooks/RB-OPS-002-qualify-cluster-bootstrap.md` を新設し、L5 / L9 / L10 の multipass kubeadm cluster を協力者が立ち上げる手順を Runbook 化（ADR-OPS-001 の 8 セクション形式に準拠）
- `.githooks/pre-commit` に `make qualify-pre-commit` を、`.githooks/pre-push` に `make qualify-pre-push` を呼ぶラッパを設置（ADR-TEST-004 で `core.hooksPath = .githooks` 強制と合わせて実装）
- 既存 `tests/e2e/scenarios/tenant_onboarding_test.go` の `t.Skip("PHASE: release-initial")` を、層分解後に `tests/e2e/L3_smoke/tenant_onboarding_test.go` で実装に置き換える（採用初期）

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— 本 ADR の前提となる qualify 全層の射程
- ADR-TEST-002（devcontainer + HW 要件）— L7 / L8 / L9 / L10 を回す前提のハードウェア要件
- ADR-OPS-001（Runbook 標準化）— qualify cluster bootstrap Runbook（RB-OPS-002）の形式根拠
- ADR-CNCF-001（CNCF Conformance）— L5 conformance E2E が満たすべき外部基準
- NFR-A-CONT-001（HA / RTO 4 時間）— L9 upgrade / L10 DR の検証対象
- NFR-B-PERF-001〜007（性能要件）— L8 scale / soak の assertion 対象
- NFR-E-AC-001〜005（アクセス制御）— L7 chaos の network-partition / pod-kill 検証
- OSSF Scorecard `CI-Tests` / `Fuzzing` 項目: scorecard.dev/checks/
- OpenSSF Best Practices Badge Silver `test_continuous_integration` / `test_unit_test_coverage`
- CNCF Project Maturity Levels（Sandbox / Incubating / Graduated）の testing maturity 軸
- Kubernetes test/e2e の層分割: github.com/kubernetes/community/blob/master/contributors/devel/sig-testing/e2e-tests.md
- Cilium test infrastructure: docs.cilium.io/en/stable/contributing/testing/
- 関連 ADR（採用検討中）: ADR-TEST-004（kind + multipass 二層 E2E）/ ADR-TEST-005（環境マトリクス）/ ADR-TEST-006（chaos / scale / soak）/ ADR-TEST-007（upgrade / DR）/ ADR-TEST-008（コンプライアンス）/ ADR-TEST-009（観測性 E2E）
