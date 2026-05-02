# ADR-TEST-006: 観測性 E2E を OTLP trace 貫通 / Prometheus cardinality / log↔trace 結合 / SLO alert 発火 / dashboard goldenfile の 5 検証で構造化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 観測性担当（採用初期）

## コンテキスト

ADR-OBS-001（OpenTelemetry）/ ADR-OBS-002（Grafana LGTM）/ ADR-OBS-003（インシデント分類体系）で観測性基盤の **構築方針** は確定済だが、**観測性基盤自体が機能しているかを検証する層** は未定義のままである。具体的には以下が「実装されているが検証経路が無い」状態:

- tier1→2→3 を貫通する trace_id が分断なく伝播しているか
- Prometheus メトリクスのカーディナリティが想定上限を超えていないか
- 構造化ログに `trace_id` フィールドが含まれ、Loki から Tempo へ navigate できる結合率が SLO（≥ 95%）を満たすか
- 故意に SLO 違反を起こした際に Alertmanager が Burn rate ベースで発火するか
- Grafana dashboard JSON が想定 panel / query / threshold を保持しているか

これら 5 検証は ADR-TEST-001 の Test Pyramid（UT / 結合 / E2E）の **どの層にも単純には属さない orthogonal 軸** で、L4 standard E2E（ADR-TEST-002）のシナリオが副次的に観察できるが、観測性固有の特性は assertion されない。本番リリース後に「アラートが鳴らなかった」「ダッシュボードが空になった」「trace が分断されていた」と判明するのは典型的な観測性負債で、これを防ぐには観測性 E2E を独立検証として確立する必要がある。

選定では以下を満たす必要がある:

- **ADR-OBS-001 / 002 / 003 の既存決定を覆さない**（OTel / LGTM スタック / インシデント分類を前提とする）
- **ADR-TEST-002 の E2E 自動化経路を再利用する**（`tools/local-stack/up.sh --role e2e` で起動する LGTM スタック上で動作）
- **5 検証の独立性**（trace tree / cardinality / log content / alert manager state / JSON diff の assertion 対象が異なる）
- **release blocking ではなく nightly 検出**（CI 時間予算と整合、ADR-TEST-001 の夜間バッチ枠）
- **採用後の運用拡大時の段階導入**（リリース時点では設計のみ）

## 決定

**観測性 E2E は以下 5 検証で構造化し、`tests/e2e/observability/` 配下に Go test として配置、ADR-TEST-002 の `_reusable-e2e.yml` から `nightly.yml` 経由で実行する。**

| # | 検証 | 配置 | assertion |
|---|------|------|-----------|
| 1 | OTLP trace 貫通 | `tests/e2e/observability/trace-propagation/` | tier1→2→3 で同一 trace_id、span tree が tier 跨ぎで連続 |
| 2 | Prometheus cardinality regression | `tests/e2e/observability/cardinality/` | metric label 組み合わせ数が baseline × 1.2 未満 |
| 3 | log ↔ trace 結合率 | `tests/e2e/observability/log-trace-correlation/` | Loki ログのうち `trace_id` フィールド有が ≥ 95% |
| 4 | SLO burn rate alert 発火 | `tests/e2e/observability/slo-alert/` | 故意 SLO 違反で fast burn alert が 5 分以内に発火 + `runbook_url` ラベル設定済 |
| 5 | Grafana dashboard goldenfile | `tests/e2e/observability/dashboard-goldenfile/` | dashboard JSON の panel / query / threshold が baseline と一致 |

検証 4 は `tools/qualify/observability/inject-slo-violation.sh`（採用初期で新設）で k6 を使い 10% error response を意図注入する。alert 発火後、Alertmanager API（mock または PagerDuty 経由）で alert 状態を確認し、`runbook_url` ラベルが ADR-OPS-001 と整合しているかを検証する。

検証 5 は `infra/observability/grafana/dashboards/*.json` を `tests/e2e/observability/dashboard-goldenfile/baselines/<dashboard>.json` の固定 baseline と jq による構造的 diff（unstable な `id` / `gnetId` は無視）で比較する。意図的更新は baseline 更新 PR で許容する。

実行頻度は **nightly**（ADR-TEST-002 の `nightly.yml` から呼ぶ、release blocking ではないが連続 N 回 fail で release tag を保留）。所要時間は約 30〜45 分（log↔trace 結合の 10 分蓄積待ちが支配的）。

採用後の運用拡大時で実装着手し、リリース時点では本 ADR + `tests/e2e/observability/` ディレクトリ skeleton（README + .gitkeep のみ）に留める。実装は採用初期で 1 検証（trace 貫通）から開始し、採用後の運用拡大時で 5 検証完備を目標とする。

## 検討した選択肢

### 選択肢 A: 5 検証統合（採用）

- 概要: 5 検証を `tests/e2e/observability/` の独立サブディレクトリで実装、`_reusable-e2e.yml` 経由で nightly 実行
- メリット:
  - **ADR-OBS-001 / 002 / 003 の既存決定が「実装かつ機能継続検証」になる**
  - 5 検証が独立 assert で「どの観測性軸が壊れたか」が機械的に特定される
  - ADR-TEST-002 の `tools/local-stack/up.sh --role e2e` の LGTM スタックを再利用、追加 cluster 起動コストなし
  - dashboard goldenfile で意図しない panel 削除 / query 誤変更が PR レビュー段階で検出される
  - SLO burn rate alert 発火検証が ADR-OPS-001 の `runbook_url` ラベル必須要件と整合的に検証される
- デメリット:
  - 5 検証実装で 5〜10 人日の初期工数（採用後の運用拡大時に分散）
  - log↔trace の 10 分蓄積待ちが nightly workflow 時間を圧迫
  - dashboard baseline 更新 PR が頻繁に発生する可能性（dashboard 編集が活発な期間）

### 選択肢 B: trace 貫通のみ

- 概要: 5 検証のうち trace 貫通だけを採用初期で実装、他 4 検証は採用後に先送り
- メリット:
  - 実装工数最小（2〜3 人日）
  - nightly 時間追加が少ない
- デメリット:
  - **cardinality 爆発 / SLO alert 不発火 / dashboard 破壊が検出されない**
  - 採用検討者の testing maturity 評価で「観測性検証が薄い」と判定される

### 選択肢 C: 観測性 E2E なし

- 概要: ADR-OBS-001/002/003 の構築のみで、機能継続検証は実施しない
- メリット:
  - 実装工数ゼロ
- デメリット:
  - **観測性負債が本番で初めて顕在化**: アラート不発火 / dashboard 空 / trace 分断が採用組織のフィードバックまで気づけない
  - ADR-OBS-001/002/003 の決定が「実装されているが未検証」のまま放置

### 選択肢 D: integration test に統合

- 概要: 観測性検証を L2 integration test 内で実施
- メリット:
  - 既存層の延長で実装可能
  - ディレクトリ構造を増やさない
- デメリット:
  - **L2 の defining property（同一プロセス / testcontainers）を超える**: kind cluster + LGTM スタック全体を要求するため L2 の境界を破る
  - SLO burn rate alert 発火のような 5 分窓が L2 の所要時間予算（< 5 分、ADR-TEST-001）に収まらない
  - 観測性 artifact が L2 の出力に紛れ込み、release artifact での切り出しが困難

## 決定理由

選択肢 A（5 検証統合）を採用する根拠は以下。

- **ADR-OBS-001 / 002 / 003 の決定の継続検証完備**: 既存 ADR で OTel + LGTM + インシデント分類を採用済だが、「実装されているが未検証」状態の放置は ADR の決定責任を果たしていない。5 検証統合により観測性の 5 主要側面が release ごとに継続検証される
- **ADR-TEST-002 のインフラ再利用**: `tools/local-stack/up.sh --role e2e` で起動する LGTM スタックを再利用するため、追加 cluster 起動コストや helm chart install が不要。selection B / D / C にはこの再利用効率がない
- **5 検証の独立性が責務分界を保つ**: trace tree / cardinality / log content / alert manager state / JSON diff の 5 つは assert 対象が異なるため、独立に失敗できる。「どの側面が壊れたか」が release artifact の `tests/.observability/` ディレクトリで機械特定でき、トリアージが速い
- **dashboard goldenfile の独自価値**: Grafana dashboard は手作業更新で誤削除 / 誤変更が業界で頻発する。goldenfile test は本検証類型でしか実装できず、選択肢 B では完全に欠落する独立価値
- **SLO burn rate alert 発火検証の ADR-OPS-001 整合**: 検証 4 は Alertmanager の `runbook_url` ラベル必須（ADR-OPS-001）を機械検証する経路で、観測性とインシデント対応 Runbook の統合品質を担保する
- **段階導入の整合**: リリース時点 = 設計のみ、採用初期 = trace 貫通実装、採用後の運用拡大時 = 5 検証完備、という段階導入が ADR-TEST-001 の Chaos / DAST 保留 / ADR-TEST-004 LitmusChaos 採用後の運用拡大時導入と並列で運用工数を平準化する

## 影響

### ポジティブな影響

- ADR-OBS-001 / 002 / 003 の決定が「実装かつ機能継続検証」になり、release ごとに観測性の 5 主要側面が assertion される
- 本番リリース前に trace 分断 / cardinality 爆発 / log↔trace 結合崩壊 / SLO alert 不発火 / dashboard 破壊が検出される
- ADR-OPS-001 の Runbook `runbook_url` ラベル必須要件が観測性 E2E の検証 4 で機械検証される
- Grafana dashboard JSON が PR レビュー対象（goldenfile baseline）となり、意図的更新と意図しない破壊が PR 単位で区別される
- ADR-TEST-002 の `_reusable-e2e.yml` インフラが再利用され、追加 cluster 起動コストなしで観測性 E2E が成立する
- 採用検討組織が「k1s0 の観測性は CI で継続検証されている」と評価でき、testing maturity が補強される

### ネガティブな影響 / リスク

- 5 検証実装で 5〜10 人日の初期工数が発生（採用後の運用拡大時に分散）
- log↔trace 結合検証の 10 分蓄積待ちが nightly workflow 時間を圧迫、ADR-TEST-002 の所要時間 30〜45 分から +10 分の追加
- dashboard goldenfile の baseline 更新 PR が dashboard 編集の活発期に頻発する可能性。`tools/qualify/observability/dashboard-baseline-update.sh` の半自動化を採用初期で整備
- cardinality regression の baseline 維持が継続コスト。新規 metric 追加ごとに baseline 登録 PR が要る
- SLO alert 発火検証の意図注入が L7 chaos（ADR-TEST-004）と時間窓衝突するとフレーキー化。ADR-TEST-006 の `make qualify-observability` を順次実行に固定し並列化を避ける
- PagerDuty mock endpoint で検証する経路は採用後の運用拡大時に実 PagerDuty 統合へ切替。ADR-SEC-002（OpenBao）に PagerDuty API key を格納する移行手順を採用後の運用拡大時に整備

### 移行・対応事項

- リリース時点で `tests/e2e/observability/{trace-propagation,cardinality,log-trace-correlation,slo-alert,dashboard-goldenfile}/` の skeleton（README + .gitkeep）を新設
- 採用初期で検証 1（trace 貫通）から実装開始、Tempo HTTP API（`/api/traces/<trace-id>`）で span tree を取得し Go test で assert
- 採用初期で `tests/e2e/observability/cardinality/baselines/<metric>.json` を版管理、各 metric の cardinality 上限値を宣言
- 採用初期で `tests/e2e/observability/dashboard-goldenfile/baselines/<dashboard>.json` を版管理、`infra/observability/grafana/dashboards/*.json` の固定済 baseline を保存
- 採用後の運用拡大時で `tools/qualify/observability/inject-slo-violation.sh` を新設、k6 で意図注入を実行
- 採用後の運用拡大時で `tools/qualify/observability/dashboard-baseline-update.sh` を新設、baseline 更新を半自動化
- 採用後の運用拡大時で `Makefile` に `qualify-observability` target を追加（5 検証を順次実行）
- ADR-TEST-002 の `_reusable-e2e.yml` で観測性 E2E を呼ぶ追加 step を採用後の運用拡大時に整備
- ADR-OBS-001 / 002 / 003 の各「帰結」セクションに「観測性 E2E（ADR-TEST-006）で 5 検証として継続検証される」を追記する relate-back 作業
- ADR-OPS-001 の Runbook `runbook_url` ラベル必須要件が ADR-TEST-006 検証 4 で機械検証されることを ADR-OPS-001 の relate-back で追記
- `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` の「拡張余地」を採用初期に再構造化し、観測性 E2E の責務分界を本ファイルに追記

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— 観測性 E2E が orthogonal 軸として位置づけられる根拠
- ADR-TEST-002（E2E 自動化）— `_reusable-e2e.yml` インフラ再利用の経路
- ADR-OBS-001（OpenTelemetry）— 検証 1 / 3 の前提
- ADR-OBS-002（Grafana LGTM）— 検証 2 / 3 / 5 の前提
- ADR-OBS-003（インシデント分類体系）— 検証 4 の SLO 分類前提
- ADR-OPS-001（Runbook 標準化）— 検証 4 の `runbook_url` ラベル必須要件
- ADR-SEC-002（OpenBao）— 採用後の運用拡大時で PagerDuty API key 格納
- IMP-DIR-COMM-112（tests 配置）— `tests/e2e/observability/` の物理配置
- NFR-B-PERF-001〜007（性能要件）— SLO burn rate alert の SLI 定義
- NFR-C-NOP-001〜003（運用要件）— Runbook 連動の前提
- Google SRE Workbook（SLI / SLO / Burn Rate）: sre.google/workbook/
- Tempo HTTP API: grafana.com/docs/tempo/latest/api_docs/
- Prometheus cardinality_estimate API: prometheus.io/docs/prometheus/latest/querying/api/
- 関連 ADR（採用検討中）: ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）
