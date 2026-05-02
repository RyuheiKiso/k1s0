# ADR-TEST-004: Chaos Engineering を LitmusChaos で実装し、概要設計の Chaos Mesh 記述を訂正する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 運用チーム（採用後）

## コンテキスト

ADR-TEST-001（Test Pyramid + testcontainers でテスト戦略を正典化）で Chaos / DAST のツール選定を **「別 ADR-TEST-* で確定」** と保留した。理由は構想設計と概要設計で記述が乖離しており、ADR-TEST-001 の射程内で確定すると一方を「正」と決めて他方を「誤」と暗黙判定することになるため。具体的には:

- **構想設計** `docs/02_構想設計/03_技術選定/03_周辺OSS/02_周辺OSS.md:383-393` で **Litmus を採用、Chaos Mesh は次点**と明記。採用理由は CNCF Incubating + Apache 2.0 + Web UI（Litmus Portal）の成熟度 + k8s ネイティブ
- **構想設計** `docs/02_構想設計/04_CICDと配信/03_テスト戦略.md:185-186` で「採用後の運用拡大時 / 採用側のマルチクラスタ移行時 に Litmus による Chaos Engineering を週次 CronChaosEngine で実行」と段階導入を確定
- **構想設計** `docs/02_構想設計/adr/ADR-DIR-002-infra-separation.md:117` で `infra/chaos/` ディレクトリを予約
- **概要設計** `docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md:3 / 60-66 / 106` で **Chaos Mesh** と記述（古い記述、構想設計の Litmus 決定が反映されていない）

つまり構想設計レベルでは Litmus 採用が確定済だが、概要設計が追従していない docs-drift 状態である。本 ADR でこの drift を解消し、両設計の SoT を **Litmus** に統一する。

加えて、ADR-OPS-001（Runbook 標準化）が「四半期 Chaos Drill」を Runbook 品質指標として cite しており、Chaos Engineering ツールの確定なしには Chaos Drill の実施手順 Runbook が整備できない。本 ADR は ADR-OPS-001 の前提となるツール確定を担う。

選定では以下を満たす必要がある:

- **構想設計の決定（Litmus 採用）と整合**
- **CNCF プロジェクトであること**（採用組織の標準スキル流用性、10 年保守の継続性）
- **k8s ネイティブ**（CRD ベースで `infra/chaos/` に置ける、ADR-DIR-002 と整合）
- **Web UI / ワークフロー管理の成熟度**（採用組織の運用チームの学習曲線緩和）
- **Apache 2.0 等の許容ライセンス**（ADR-0003 AGPL 分離、商用採用の選択肢を消さない）
- **採用後の運用拡大時 段階導入**（ADR-TEST-001 / 概要設計と整合、リリース時点では未実装）
- **5 シナリオ最低セット**（Pod Kill / Network Latency / Network Partition / CPU Stress / Disk IO Stress、概要設計の chaos セクションと一致）

## 決定

**Chaos Engineering ツールは LitmusChaos（v3+）を採用する。** 採用後の運用拡大時に `operation` namespace にデプロイし、週次 CronChaosEngine で 5 シナリオ最低セットを順次実行する。

### 1. LitmusChaos 採用の確定事項

- **バージョン**: LitmusChaos v3 系（CRD: ChaosEngine / ChaosExperiment / ChaosResult / ChaosSchedule）
- **ライセンス**: Apache 2.0（ADR-0003 AGPL 分離との整合）
- **CNCF ステータス**: Incubating（CNCF プロジェクト、採用組織のスキル流用性）
- **deploy 配置**: `operation` namespace（chaos 実験の隔離、業務 namespace 非汚染）
- **管理 UI**: Litmus Portal（オプション、リソース余裕がある場合のみ）。リソース逼迫時は CLI + CRD のみで運用
- **配置 SoT**: `infra/chaos/`（ADR-DIR-002 で予約済ディレクトリ）

### 2. シナリオ最低セット

採用後の運用拡大時で実装する 5 シナリオ（概要設計 `05_テスト戦略方式.md:64` と一致）:

| シナリオ | 内容 | 対象 | 検証指標 |
|---------|------|------|---------|
| Pod Kill | tier1 Pod をランダム削除 | tier1 Go ファサード / Rust サービス | Service 継続（Availability ≥ 99%） |
| Network Latency | 依存サービス間に 500ms の遅延注入 | tier1 ↔ Valkey / Kafka / PostgreSQL | タイムアウト動作の正常性 |
| Network Partition | Valkey / Kafka を一時切断 | tier1 ↔ data layer | フォールバック動作 |
| CPU Stress | worker ノード CPU 80% 占有 | 全 Pod | scheduler の他ノード退避 |
| Disk IO Stress | 永続ボリューム書き込み遅延注入 | CNPG / OpenBao | IO 遅延時の SLO 維持 |

各シナリオは `infra/chaos/<scenario-name>/` 配下に ChaosEngine / ChaosExperiment CRD として記述する。実行頻度は週次（CronChaosEngine 経由）で、staging 環境の `operation` namespace で実行。

### 3. 段階導入計画

| 段階 | 実装内容 |
|------|---------|
| リリース時点 | 本 ADR + 概要設計訂正のみ。Litmus 自体はデプロイしない |
| 採用初期 | `infra/chaos/` ディレクトリと `manifests/litmus/` の Helm values を整備（実デプロイは保留） |
| 採用後の運用拡大時 | Litmus を `operation` namespace にデプロイ、5 シナリオを CronChaosEngine で週次実行開始 |
| 採用側のマルチクラスタ移行時 | カスケード障害テスト（ネットワーク障害注入・本番 namespace 対象）に拡張 |

リリース時点では実装ゼロ、ADR と概要設計訂正で「決定の記録」を残す。

### 4. 評価指標

実行結果は MTTR / Availability / Error Rate の 3 指標で評価し、SLO 破綻が発生したシナリオは設計見直しの起点とする。Grafana LGTM スタック（ADR-OBS-002）との統合により、Litmus 実行中の SLI 推移を可視化する（ADR-TEST-006 で詳細決定予定）。

### 5. 概要設計訂正

本 ADR の起票と同時に、概要設計 `docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md` の Chaos Mesh 記述 3 箇所（line 3 / line 60-66 セクションタイトル + 本文 / line 106 設計 ID 表）を **LitmusChaos** に訂正する。これは docs SoT を構想設計に合わせる作業で、本 ADR commit と同 commit で実施する（drift を放置しない）。

## 検討した選択肢

### 選択肢 A: LitmusChaos（採用）

- 概要: ChaosNative / Harness 主導の Chaos Engineering プラットフォーム、CNCF Incubating、k8s ネイティブ CRD + Web UI
- メリット:
  - **構想設計の決定（02_周辺OSS / 04_CICDと配信）と完全整合**
  - CNCF Incubating + Apache 2.0、10 年保守の継続性
  - Web UI（Litmus Portal）が採用組織の運用チーム学習曲線を緩和
  - CRD ベースで `infra/chaos/` に置ける（ADR-DIR-002 整合）
  - ChaosHub（公開シナリオライブラリ）が充実、5 シナリオ最低セットを既存定義から流用可能
  - CronChaosEngine で週次 schedule が標準機能として提供される
- デメリット:
  - Litmus Portal を起動するとリソース消費が増える（CPU / メモリ）。Portal なしの CLI + CRD 運用に切替可能だが運用コスト計測が要る
  - LitmusChaos v2 → v3 のバージョン移行で CRD schema が変わった経緯があり、将来の major upgrade に追従コストが要る

### 選択肢 B: Chaos Mesh

- 概要: PingCAP 主導の Chaos Engineering ツール、CNCF Incubating、k8s ネイティブ CRD + Dashboard
- メリット:
  - 機能は LitmusChaos と同等（Pod / Network / IO / Time / Stress / DNS の 6 系統 chaos）
  - PingCAP の活発な開発、TiDB プロジェクトでの本番運用実績
  - Dashboard UI が提供される
- デメリット:
  - **構想設計（02_周辺OSS）で「次点」と明示**、Web UI / ワークフロー管理が Litmus に劣ると判定済
  - 概要設計 `05_テスト戦略方式.md` の Chaos Mesh 記述は構想設計を反映していない古い記述で、本記述に従うと SoT が drift する
  - ADR-DIR-002 の `infra/chaos/` ディレクトリ予約は Litmus 想定で、Chaos Mesh に変更すると ADR-DIR-002 への影響が波及

### 選択肢 C: Toxiproxy

- 概要: Shopify 製の軽量ネットワーク chaos ツール、Go 製、k8s 不要
- メリット:
  - 軽量、起動が高速
  - JSON 設定でシンプル
  - testcontainers から呼び出して結合テスト層で chaos 注入可能
- デメリット:
  - **ネットワーク chaos に特化**: Pod kill / IO stress / CPU stress / Time chaos が再現できず、5 シナリオ最低セットの 4/5 を満たせない
  - k8s ネイティブでない、CRD 管理外、ADR-DIR-002 の `infra/chaos/` 構造に乗らない
  - 構想設計の選定軸（Web UI / ワークフロー管理）と不整合

### 選択肢 D: 自前 chaos 実装

- 概要: shell script + kubectl 操作で chaos 注入を自前実装
- メリット:
  - 外部依存ゼロ
  - 全制御を自分が持つ
- デメリット:
  - **運用工数が爆発**: 5 シナリオ × バージョン追従 × Grafana 統合 × Portal UI を自前で書くと年数十人日の工数
  - Litmus / Chaos Mesh が CNCF プロジェクトとして提供する成熟機能（CronChaosEngine schedule / ChaosHub / 結果集約 / SLO assertion）を再発明することになる
  - 採用検討者から見て「自前 chaos は信頼できるか」の説明工数が継続発生

## 決定理由

選択肢 A（LitmusChaos）を採用する根拠は以下。

- **構想設計の既存決定との完全整合**: `docs/02_構想設計/03_技術選定/03_周辺OSS/02_周辺OSS.md:383-393` で「Litmus 採用、Chaos Mesh は次点」と明記済。本 ADR は構想設計の決定を構想設計レベル ADR として正典化し、概要設計の drift を訂正する役割。選択肢 B（Chaos Mesh）は構想設計を覆す決定になり、ADR-0003 / ADR-DIR-002 / 02_周辺OSS の整合性連鎖を破る
- **ADR-DIR-002 の infra/chaos/ ディレクトリ予約との整合**: 構想設計レベルで `infra/chaos/` 配置が予約済、Litmus はこのディレクトリ構造を CRD で素直に表現できる。選択肢 C（Toxiproxy）は k8s ネイティブでなく、`infra/chaos/` 構造に乗らない
- **採用組織の運用チーム学習曲線の緩和**: Litmus Portal は実験の実行状況・結果を可視化する Web UI を提供し、採用組織の運用エンジニアが CRD を直接読み書きせずに chaos drill を回せる。これは ADR-OPS-001 の「夜間休日に協力者が単独対応するためのバス係数 2」と整合（chaos drill 実施を運用エンジニアが Portal 経由で完結できる）
- **CNCF プロジェクトとしての継続性**: Litmus は CNCF Incubating + Apache 2.0 で、10 年保守の前提で OSS の継続性が高い。選択肢 D（自前実装）は維持工数で破綻、選択肢 C（Toxiproxy）は CNCF 採択外で長期保守の担保が弱い
- **段階導入と採用後の運用拡大時射程の整合**: 構想設計 `04_CICDと配信/03_テスト戦略.md:185-186` で「採用後の運用拡大時 / 採用側のマルチクラスタ移行時 に Litmus による Chaos Engineering」と段階導入が確定済。リリース時点では実装ゼロ、ADR + 概要設計訂正のみ。これは ADR-TEST-001 の Chaos 保留と完全整合
- **概要設計 drift の解消経路確立**: 本 ADR の決定により、概要設計 `05_テスト戦略方式.md` の Chaos Mesh 記述 3 箇所を LitmusChaos に訂正する作業を本 commit で履行できる。selection drift（構想設計 vs 概要設計）を放置せず、SoT を Litmus に統一する

## 影響

### ポジティブな影響

- 構想設計と概要設計の Chaos ツール記述が LitmusChaos に統一され、SoT drift が解消する
- ADR-TEST-001 で保留した Chaos / DAST のうち Chaos が確定し、後続 ADR-TEST-* の前提が明確化
- ADR-DIR-002 の `infra/chaos/` ディレクトリ予約が Litmus 想定として固定される
- ADR-OPS-001 の Chaos Drill 四半期実施が Litmus 経由で実装可能になり、Runbook 整備の前提が確立
- 採用組織の運用エンジニアが Litmus Portal の Web UI で chaos drill を完結でき、バス係数 2 と整合
- 5 シナリオ最低セットが ChaosHub の公開シナリオから流用でき、採用後の運用拡大時の実装工数が削減
- 概要設計 `05_テスト戦略方式.md` の DS-DEVX-TEST-008（Chaos Mesh、採用後の運用拡大時）を DS-DEVX-TEST-008（LitmusChaos、採用後の運用拡大時）に訂正することで、概要設計 ID と本 ADR の対応が確立

### ネガティブな影響 / リスク

- LitmusChaos Portal を採用組織がデプロイすると `operation` namespace に追加リソース消費が発生（CPU / メモリ）。リソース逼迫時は Portal なしの CRD のみ運用に切替可能だが、運用チーム向けの learning curve が上がる
- LitmusChaos v2 → v3 で CRD schema が変わった経緯があり、将来の major version upgrade で `infra/chaos/` 配下の CRD を全件更新する必要がある。採用初期で Renovate 自動 PR + 手動レビュー経路を整備
- 採用後の運用拡大時で初めて実デプロイされるため、リリース時点〜採用初期の間 Litmus 自体の動作検証は手元 kind cluster での dry-run のみ。本番 cluster での挙動差は採用後に判明するリスク
- 概要設計 `05_テスト戦略方式.md` の訂正で Chaos Mesh → LitmusChaos に書き換えるが、概要設計の他章（例: 性能設計 / 可用性設計）が Chaos Mesh 前提で書かれている可能性があり、本 commit のスコープを超えた drift が残るリスク。`/audit` 実行で発見した場合は別 PR で順次訂正

### 移行・対応事項

- 本 ADR commit で概要設計 `docs/04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md` の Chaos Mesh 記述 3 箇所を LitmusChaos に訂正（line 3 / line 60-66 / line 106）
- `docs/03_要件定義/00_要件定義方針/08_ADR索引.md` の TEST 系列に本 ADR を登録
- `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` の「拡張余地」に「ADR-TEST-004 起票時に L7 chaos の責務分界を本ファイルに追記」を予告として記載（本 commit では拡張せず、ADR-TEST-005 / 006 起票時にまとめて再構造化）
- 採用初期で `infra/chaos/` ディレクトリと 5 シナリオ最低セットの ChaosEngine / ChaosExperiment CRD 雛形を配置（実デプロイは採用後の運用拡大時）
- 採用後の運用拡大時で `manifests/litmus/` Helm values を `infra/environments/<env>/` overlay で起動、`operation` namespace にデプロイ
- 採用後の運用拡大時で 5 シナリオの実行 Runbook（`ops/runbooks/RB-CHAOS-001〜005`）を 8 セクション形式（ADR-OPS-001 準拠）で整備、ADR-OPS-001 の Chaos Drill 四半期実施に組み込み
- ADR-DIR-002 の「帰結」セクションに「`infra/chaos/` は LitmusChaos 想定（ADR-TEST-004）」を追記する relate-back 作業
- ADR-TEST-001 の「決定」表内 Chaos 行（「ツール選定は別 ADR-TEST-004 で決定」）を本 ADR 確定後に「LitmusChaos（ADR-TEST-004）」に訂正する relate-back 作業（ADR-TEST-007 起票時または `/audit` 実行で発見した時点で実施）

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— Chaos ツール選定を本 ADR に保留した起源
- ADR-TEST-003（CNCF Conformance / Sonobuoy）— 同じ「採用後の運用拡大時段階導入」の前例
- ADR-DIR-002（infra 分離）— `infra/chaos/` ディレクトリ予約
- ADR-OPS-001（Runbook 標準化）— Chaos Drill 四半期実施の前提ツール確定
- ADR-OBS-002（Grafana LGTM）— Litmus 実行中の SLI 可視化（ADR-TEST-006 で詳細決定）
- ADR-0003（AGPL 分離）— Litmus の Apache 2.0 整合
- 構想設計 `02_構想設計/03_技術選定/03_周辺OSS/02_周辺OSS.md:383-393` — Litmus 採用の根拠
- 構想設計 `02_構想設計/04_CICDと配信/03_テスト戦略.md:185-186` — 段階導入計画
- 概要設計 `04_概要設計/70_開発者体験方式設計/05_テスト戦略方式.md`（本 commit で訂正）
- DS-DEVX-TEST-008（Chaos、採用後の運用拡大時）
- ChaosNative LitmusChaos: litmuschaos.io
- ChaosHub: hub.litmuschaos.io
- 関連 ADR（採用検討中）: ADR-TEST-005（Upgrade / DR drill）/ ADR-TEST-006（観測性 E2E）/ ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）
