# ADR-TEST-009: 観測性 E2E を OTLP trace 貫通 / Prometheus cardinality / log↔trace 結合 / SLO burn rate alert / dashboard goldenfile の 5 検証で構造化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 観測性担当（採用初期）

## コンテキスト

ADR-OBS-001 で OpenTelemetry（OTel）を全言語の計装標準、ADR-OBS-002 で Grafana LGTM スタック（Loki / Grafana / Tempo / Mimir）、ADR-OBS-003 でインシデント分類体系を採用済である。これらは「観測性基盤を構築する」決定だが、**観測性基盤自体が機能しているかをテストする層** は未定義のままだった。

観測性は qualify の L0–L10 のいずれにも単純には属さない orthogonal な軸である。trace 貫通や log↔trace 結合のような検証は L4 standard E2E のシナリオ実行中に副次的に観察できるが、それは「シナリオが成功したか」を assertion しているだけで、「trace_id が tier1→2→3 を正しく伝播したか」「Prometheus メトリクスのカーディナリティが爆発していないか」「故意に SLO 違反を起こしたら正しくアラートが発火するか」のような観測性固有の特性を直接 assertion してはいない。本番リリース後に「アラートが鳴らなかった」「ダッシュボードが空になった」「trace が分断されていた」と判明するのは典型的な観測性負債で、これを防ぐには観測性 E2E を独立検証として `make qualify-release` に組み込む必要がある。

観測性 E2E が拾うべき検証類型は以下 5 つに分類できる:

1. **OTLP trace 貫通**: tier1→2→3 を貫通する単一の trace_id が、tier1 の入口で生成されてから tier3 まで分断なく伝播していること。span parent-child 関係が正しく構築されていること
2. **Prometheus cardinality regression**: メトリクスのラベル組み合わせ数（cardinality）が想定上限を超えていないこと。リリースごとに新規追加された label が cardinality 爆発を起こしていないこと
3. **log ↔ trace 結合率**: 構造化ログに `trace_id` フィールドが含まれ、Loki から Tempo へ trace ID で navigate できる結合率が SLO（≥ 95%）を満たすこと
4. **SLO burn rate alert 発火**: `tools/qualify/observability/inject-slo-violation.sh` で意図的に SLO 違反を起こし、Alertmanager / PagerDuty への通知が SLO burn rate ベースで発火することを assert
5. **Grafana dashboard goldenfile test**: dashboard JSON が想定どおりの panel / query / threshold を保持していることを `tests/e2e/observability/goldenfiles/<dashboard>.json` の固定済 baseline と diff 比較

これら 5 検証はそれぞれ独立に実装可能だが、観測性軸として統合的に扱う必要がある。理由は ① いずれも Grafana LGTM スタック全体への依存（Loki + Grafana + Tempo + Mimir + Alertmanager + OTel Collector）が共通、② いずれも tier1→2→3 の貫通シナリオの上で初めて成立、③ いずれも release blocking とすべき性質（一つでも壊れると本番観測性が成立しない）— にある。

選定では以下を満たす必要がある:

- **ADR-OBS-001 / 002 / 003 の整合**: OTel + Grafana LGTM + インシデント分類体系の前提に整合
- **kind cluster で動く** 軽量さ（multipass までは不要、L7 / L8 と同じ kind 環境で済ませる）
- **release blocking 可能な所要時間**: 5 検証合計で 30 分〜1 時間以内
- **assertion の機械判定**: 各検証の合格基準が SLO クエリ / JSON diff / count 比較で機械化される
- **CNCF Sandbox 採用基準の observability maturity との整合**: SRE Workbook / Google SRE Handbook の SLI / SLO / Burn Rate alert の慣行と一致

## 決定

**観測性 E2E を以下 5 検証で構造化し、`tests/e2e/observability/` ディレクトリに配置、`make qualify-release` から呼ばれる `make qualify-observability` target で統合実行する。**

### 1. OTLP trace 貫通 (`tests/e2e/observability/trace-propagation/`)

- **シナリオ**: kind cluster 上で tier1→2→3 を貫通する代表シナリオ（tenant-onboarding と同じフロー）を実行
- **検証**: tier1 の入口で生成された `trace_id` が、Tempo に投入された全 span で同一であること、span tree が tier1→2→3 の順で parent-child 関係を保つこと
- **assertion**:
  - `tempo` HTTP API（`/api/traces/<trace-id>`）で trace 全体を取得
  - span 数 ≥ 3（tier1 / tier2 / tier3 各 1 以上）
  - 全 span の `trace_id` が同一
  - root span が tier1、leaf span が tier3
  - span の parent-child 関係が tier 跨ぎで連続している
- **所要時間**: 約 5 分

### 2. Prometheus cardinality regression (`tests/e2e/observability/cardinality/`)

- **シナリオ**: kind cluster の Mimir に対し、全 metric の label 組み合わせ数を `cardinality_estimate` API で取得
- **検証**: 各 metric の cardinality が想定上限を下回ること、リリース間で 20% 以上の急増が無いこと
- **assertion**:
  - `tests/e2e/observability/cardinality/baselines/<metric>.json` に各 metric の上限値を版管理
  - 現 cardinality > baseline × 1.2 で fail
  - 新規 metric は警告のみ（baseline に未登録の metric は次 release で baseline 化を必須化）
- **所要時間**: 約 5 分

### 3. log ↔ trace 結合率 (`tests/e2e/observability/log-trace-correlation/`)

- **シナリオ**: kind cluster で代表シナリオを実行 → Loki と Tempo に投入されたログ・trace を取得
- **検証**: Loki ログのうち `trace_id` フィールドを含むログの割合が 95% 以上、`trace_id` から Tempo への navigation が 100% 成功
- **assertion**:
  - Loki LogQL で `count_over_time({app=~".+"} | json | trace_id != "" [10m])` / `count_over_time({app=~".+"} [10m])` ≥ 0.95
  - 取得した trace_id をランダムに 10 件サンプル → Tempo `/api/traces/<trace-id>` で全件取得成功
- **所要時間**: 約 10 分（10 分間ログ蓄積待ち）

### 4. SLO burn rate alert 発火 (`tests/e2e/observability/slo-alert/`)

- **シナリオ**: kind cluster で `tools/qualify/observability/inject-slo-violation.sh` を実行し、tier1 の availability を意図的に 90% に低下させる（k6 で 10% error response を強制注入）
- **検証**: Alertmanager で SLO burn rate alert が **fast burn**（5 分窓 14.4× rate）で発火し、SilenceMatcher / PagerDuty 連携経路で通知が届くこと
- **assertion**:
  - SLO 違反開始から 5 分以内に Alertmanager `alerts` API でアクティブな alert を確認
  - alert label に `severity=page` / `category=availability` / `slo_window=fast`
  - `runbook_url` ラベルが設定済（ADR-OPS-001 / RB-OPS-* と整合）
  - PagerDuty mock endpoint への通知 POST を確認（リリース時点ではローカル mock、Phase 3 で実 PagerDuty）
- **所要時間**: 約 10 分

### 5. Grafana dashboard goldenfile test (`tests/e2e/observability/dashboard-goldenfile/`)

- **シナリオ**: `infra/observability/grafana/dashboards/*.json` の各 dashboard JSON を `tests/e2e/observability/dashboard-goldenfile/baselines/` の固定済 baseline と diff 比較
- **検証**: 各 dashboard が想定どおりの panel / query / threshold / alert を保持していること
- **assertion**:
  - JSON 構造的 diff（jq でキー単位比較、unstable な `id` / `gnetId` は無視）
  - panel 数 / query 内の PromQL / threshold 値が baseline と一致
  - 差分があれば fail、ただし baseline 更新 PR で意図的更新は許容（PR レビュー対象）
- **所要時間**: 約 1 分

### 統合実行

`make qualify-observability` で 5 検証を順次実行し、結果を `tests/qualify-report/<version>/observability/` に統合する。`make qualify-release`（ADR-TEST-001）から呼ばれ、release tag 時に必須実行される。所要時間は合計約 30〜45 分（log↔trace 結合の 10 分蓄積待ちが支配的）。

## 検討した選択肢

### 選択肢 A: 5 検証統合（採用）

- 概要: trace 貫通 / cardinality / log↔trace 結合 / SLO alert / dashboard goldenfile の 5 検証を `make qualify-observability` で一括実行、release blocking
- メリット:
  - 観測性軸を 1 ヶ所で網羅し、release artifact `observability/` ディレクトリに統合同梱
  - ADR-OBS-001 / 002 / 003 の決定が「実装上機能しているか」を継続検証
  - SRE Workbook / Google SRE Handbook の SLI / SLO / Burn Rate alert の慣行と整合
  - 5 検証が独立に失敗できるため、観測性のどの側面が壊れたかが artifact から特定可能
- デメリット:
  - 5 検証の実装工数が中規模（5〜10 人日）
  - log↔trace 結合の 10 分蓄積待ちが release qualify 時間を圧迫
  - dashboard goldenfile の baseline 更新 PR が頻繁に発生する可能性

### 選択肢 B: trace 貫通のみ

- 概要: tier1→2→3 の trace 貫通だけを検証、他 4 検証は Phase 1 以降に先送り
- メリット:
  - 実装工数が最小（2〜3 人日）
  - release qualify 時間の追加が小さい
- デメリット:
  - **cardinality 爆発が検出できない**: 本番で metric scrape が破綻するまで気づけない
  - **SLO alert 発火検証なし**: 本番で「アラートが鳴るはずが鳴らなかった」事故を防げない
  - **dashboard 破壊の検出なし**: Grafana dashboard 更新で意図せず panel が消えても検出不可
  - 観測性軸が「貫通だけ」では本番観測性の信頼性が担保されない

### 選択肢 C: 観測性 E2E なし（既存テストに含めない）

- 概要: 観測性検証を独立層化せず、L4 standard E2E のシナリオが副次的に observability を観察するに留める
- メリット:
  - 実装工数ゼロ
  - 既存層のシナリオで「事実上 trace が動いていれば」OK とする
- デメリット:
  - **observability 固有の検証が原理的に成立しない**: cardinality / SLO alert / dashboard / log↔trace のどれも L4 のシナリオでは assertion されない
  - ADR-OBS-001 / 002 / 003 の決定が「実装されているが検証されていない」状態になり、本番で破綻するまで気づけない
  - CNCF Sandbox 採用基準の observability maturity が低評価

### 選択肢 D: integration test に統合

- 概要: 観測性検証を L2 integration test 内で実施し、独立層化しない
- メリット:
  - L2 既存層の延長で実装でき、ディレクトリ構造を増やさない
  - 認知負荷が低い
- デメリット:
  - **integration test の責務が肥大化**: L2 は同一プロセス内 / testcontainers が defining property だが、観測性検証は kind cluster + Grafana LGTM スタック全体を要求するため L2 の境界を超える
  - SLO burn rate alert 発火のような時間窓を必要とする検証は integration test の所要時間予算（< 5 分）に収まらない
  - 観測性 artifact が L2 の出力に紛れ込み、release artifact での切り出しが難しい

## 決定理由

選択肢 A（5 検証統合）を採用する根拠は以下。

- **ADR-OBS-001 / 002 / 003 の決定の検証完備**: 既存 ADR で OTel + Grafana LGTM + インシデント分類体系を採用済だが、これらが「実装されているが機能しているか検証されていない」状態を放置するのは ADR の決定責任を果たしていない。5 検証統合により、観測性の 5 つの主要側面（trace / metric / log / alert / dashboard）が release tag ごとに継続検証される
- **本番観測性の信頼性担保**: 選択肢 B / C / D はいずれかの側面（特に SLO alert / cardinality / dashboard）の検証を欠き、本番で観測性破綻が発生する可能性を残す。5 検証統合は本番リリース前に観測性の各側面を機械判定するため、観測性負債が release tag 段階で必ず検出される
- **CNCF Sandbox / Graduated 級 OSS の observability maturity と整合**: Cilium / Istio / ArgoCD などの CNCF Graduated 級 OSS は観測性 E2E を独立検証として持っており、k1s0 の testing maturity 評価で「観測性軸を独立検証している」と説明可能になる。選択肢 D（integration 内）は CNCF プロジェクトの慣行と乖離
- **Grafana LGTM 統合の自然性**: ADR-TEST-006 の k6 / Chaos Mesh 統合と同じく、kind cluster + Grafana LGTM の組み合わせで完結する。multipass kubeadm までは不要で、kind 上の helm chart install で全 LGTM コンポーネントが動く。所要時間 30〜45 分は release qualify 全体の time budget に収まる
- **5 検証の独立性**: 5 検証は assertion 対象が異なる（trace tree / cardinality / log content / alert manager state / JSON diff）ため、独立に失敗できる。release artifact の `observability/` ディレクトリで「どの側面が壊れたか」が機械的に特定でき、トリアージが速い。選択肢 B では他軸の検証が欠落、選択肢 D では失敗の局所性が崩れる
- **dashboard goldenfile の本質的価値**: Grafana dashboard は手作業で更新されることが多く、panel が誤削除 / query が誤変更される事故が業界で頻発する。goldenfile test で baseline と diff 比較することで、意図しない dashboard 破壊が必ず検出される。これは選択肢 B には含まれない独立価値

## 影響

### ポジティブな影響

- ADR-OBS-001 / 002 / 003 の決定が「実装かつ機能継続検証」になり、release ごとに観測性の 5 側面が assert される
- 本番リリース前に trace 分断 / cardinality 爆発 / log↔trace 結合崩壊 / SLO alert 不発火 / dashboard 破壊が検出される
- release artifact `observability/` ディレクトリで観測性の継続検証証跡が採用検討者向けに公開される
- SLO burn rate alert の発火検証が確立し、ADR-OPS-001 の Runbook（runbook_url ラベル必須）と統合的に動作確認される
- Grafana dashboard JSON が PR レビュー対象となり、意図的更新と意図しない破壊が PR 単位で区別される
- CNCF Sandbox 採用基準の observability maturity が「独立検証あり」と評価される

### ネガティブな影響 / リスク

- 5 検証実装で 5〜10 人日の初期工数が発生する。特に SLO burn rate alert 発火検証は inject-slo-violation.sh の k6 シナリオ + Alertmanager 連携の確認で時間がかかる
- log↔trace 結合検証の 10 分蓄積待ちが release qualify 時間を圧迫する。並列実行で他検証と重ねられるが、`make qualify-release` 全体の連続マシン占有が伸びる
- dashboard goldenfile の baseline 更新 PR が頻繁に発生する可能性がある。dashboard 編集が活発な期間（例: 観測性改善期）では PR レビュー工数が線形以上に増える。`tools/qualify/observability/dashboard-baseline-update.sh` で半自動化を Phase 1 で整備する必要
- cardinality regression の baseline 維持が継続コスト。新規 metric を追加するたびに baseline.json への登録 PR が要る。レビュー時に「上限値が妥当か」を判断する規律が要る
- SLO burn rate alert 発火検証は「故意に SLO 違反を起こす」ため、kind cluster 上の他検証（L7 chaos など）と時間窓が衝突するとフレーキー化する。`make qualify-observability` を順次実行に固定し並列化を避けることで吸収する
- PagerDuty mock endpoint で検証する経路は Phase 3 で実 PagerDuty に切り替える際、追加の statefulness（API key / routing rule）の管理が必要。OpenBao（ADR-SEC-002）に格納する移行手順を Phase 3 で `docs/governance/PAGERDUTY-INTEGRATION.md` で整備

### 移行・対応事項

- `tests/e2e/observability/` を新設し、5 検証のサブディレクトリ（trace-propagation / cardinality / log-trace-correlation / slo-alert / dashboard-goldenfile）を配置
- `tests/e2e/observability/cardinality/baselines/<metric>.json` を新設し、各 metric の cardinality 上限値を版管理
- `tests/e2e/observability/dashboard-goldenfile/baselines/<dashboard>.json` を新設し、各 dashboard の固定済 baseline を版管理
- `tools/qualify/observability/inject-slo-violation.sh` を新設し、k6 で 10% error response を強制注入する shell script
- `tools/qualify/observability/grafana-lgtm-install.sh` を新設し、kind cluster に Grafana LGTM スタック helm chart を冪等 install
- `tools/qualify/observability/dashboard-baseline-update.sh` を新設し、Grafana dashboard JSON の baseline 更新を半自動化（Phase 1 で完成）
- `Makefile` に `qualify-observability` target を追加（5 検証を順次実行し `tests/qualify-report/<version>/observability/` に集約）
- `docs/governance/OBSERVABILITY-E2E.md` を新設し、5 検証の defining property と assertion ロジックを散文 + 図で記述、採用検討者向けの説明動線を確立
- `infra/observability/grafana/dashboards/` 配下の各 dashboard JSON に「goldenfile test 対象」ラベルを追加し、PR レビュー時に baseline 更新の有無を必ず確認する規律を `docs/governance/QUALIFY-POLICY.md` で明文化
- ADR-OBS-001（OTel）/ ADR-OBS-002（Grafana LGTM）/ ADR-OBS-003（インシデント分類）の各「帰結」セクションに「観測性 E2E（ADR-TEST-009）で 5 検証として継続検証される」を追記する relate-back 作業
- ADR-OPS-001 の Runbook `runbook_url` ラベル必須要件と SLO burn rate alert 発火検証の結合を、`tests/e2e/observability/slo-alert/runbook-url-assertion.go` で機械判定

## 参考資料

- ADR-TEST-001（CI 留保 + qualify portable 設計）— release artifact `observability/` 同梱の整合
- ADR-TEST-003（テストピラミッド L0–L10）— 観測性 E2E が orthogonal 軸として扱われる根拠
- ADR-TEST-004（kind + multipass 二層 E2E）— 観測性 E2E が kind 上で動く根拠
- ADR-TEST-006（chaos / scale / soak）— k6 / Chaos Mesh と Grafana LGTM の統合
- ADR-OBS-001（OpenTelemetry）— 検証 1 の前提
- ADR-OBS-002（Grafana LGTM）— 検証 2 / 3 / 5 の前提
- ADR-OBS-003（インシデント分類体系）— 検証 4 の SLO 分類
- ADR-OPS-001（Runbook 標準化）— 検証 4 の `runbook_url` ラベル必須要件
- ADR-SEC-002（OpenBao）— Phase 3 で PagerDuty API key 格納
- NFR-B-PERF-001〜007（性能要件）— SLO burn rate alert の SLI 定義
- NFR-C-NOP-001〜003（運用要件）— Runbook 連動の前提
- Google SRE Workbook（SLI / SLO / Burn Rate）: sre.google/workbook/
- OpenTelemetry: opentelemetry.io
- Grafana LGTM: grafana.com/oss/
- Tempo HTTP API: grafana.com/docs/tempo/latest/api_docs/
- Prometheus cardinality_estimate API: prometheus.io/docs/prometheus/latest/querying/api/
