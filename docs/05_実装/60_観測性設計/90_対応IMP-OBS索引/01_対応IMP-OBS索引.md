# 01. 対応 IMP-OBS 索引

本ファイルは `60_観測性設計/` 配下で採番した全 67 件の `IMP-OBS-*` ID を一覧化し、ID 採番ルール / 接頭辞別の所在 / 上流 ADR・DS-SW-COMP・NFR への逆引きを提供する。実装段階で「どの ID がどの設計判断に対応するか」を機械可読に追跡するための正典として固定する。

## ID 採番ルール

`IMP-OBS-<sub-prefix>-<番号>` 形式で採番する。`<sub-prefix>` は章ごとに固有で、番号は接頭辞をまたいで**単一番号空間を共有**する（POL は 001-009、OTEL は 010-019、LGTM は 020-029、PYR は 030-039、SLO は 040-049、EB は 050-059、INC は 060-079、RB は 080-089）。これは将来「番号だけ見て該当文書を見つける」ための一意性確保。

INC が 060-071 の 12 件まで使用しているため、Runbook 用 RB は 080 から開始した。1 接頭辞 10 番号枠を基本とし、INC のように超過する場合は次接頭辞をスキップする運用。

## 接頭辞別の所在と範囲

| sub-prefix | 章 | 番号レンジ | 件数 | 設計対象 |
|---|---|---|---|---|
| POL | 00_方針/01_観測性原則.md | 001-007 | 7 | 観測性原則（OTel 集約 / AGPL 分離 / SRE Book 準拠） |
| OTEL | 10_OTel_Collector配置/01_OTel_Collector配置.md | 010-019 | 10 | OTel Agent + Gateway 2 段配置 |
| LGTM | 20_LGTM_Stack/01_LGTM_Stack配置.md | 020-029 | 10 | Loki / Grafana / Tempo / Mimir 配置と AGPL 隔離 |
| PYR | 30_Pyroscope/01_Pyroscope統合.md | 030-039 | 10 | 連続プロファイリング SDK push + eBPF pull |
| SLO | 40_SLO_SLI定義/01_tier1_公開11API_SLO_SLI.md | 040-047 | 8 | tier1 公開 11 API の SLI / SLO 定義 |
| EB | 50_ErrorBudget運用/01_ErrorBudget運用.md | 050-057 | 8 | Error Budget 4 状態と自動アクション |
| INC | 60_Incident_Taxonomy/01_Incident_Taxonomy統合分類.md | 060-071 | 12 | 可用性 + セキュリティ統合 Incident 分類 |
| RB | 70_Runbook連携/01_Runbook連携.md | 080-089 | 10 | SLI ↔ Alert ↔ Runbook 1:1 結合 |

接頭辞 8 種で機能境界が物理的に分離されている。「どの章で起こっている問題か」を ID 接頭辞だけで判定でき、ID を grep するだけで影響範囲を特定できる構造を保つ。

## 全 75 件の ID 一覧

### POL: 観測性原則（00_方針）

| ID | 設計内容 |
|---|---|
| IMP-OBS-POL-001 | 全テレメトリは OTel Collector 経由（直接 backend 接続を禁ずる） |
| IMP-OBS-POL-002 | LGTM Stack の AGPL 分離（別プロセス / 別 namespace） |
| IMP-OBS-POL-003 | Google SRE Book 準拠（SLI / SLO / Error Budget の 3 点運用） |
| IMP-OBS-POL-004 | Incident Taxonomy 可用性 × セキュリティ統合（ADR-OBS-003 として起票予定） |
| IMP-OBS-POL-005 | Error Budget 100% 消費時の feature deploy 凍結 |
| IMP-OBS-POL-006 | Runbook と SLI / Alert の 1:1 紐付け |
| IMP-OBS-POL-007 | DORA 4 keys は 95 章へ分離（観測性は稼働 SLI に絞る） |

### OTEL: OTel Collector 配置（10_OTel_Collector配置）

| ID | 設計内容 |
|---|---|
| IMP-OBS-OTEL-010 | Agent を DaemonSet で全 Node 配置、収集と中継のみ |
| IMP-OBS-OTEL-011 | Gateway を Deployment + HPA、3〜10 replicas、PII / sampling / routing を集中 |
| IMP-OBS-OTEL-012 | Gateway pipeline（transform / tail_sampling / resource / batch / routing）固定 |
| IMP-OBS-OTEL-013 | PII マスキングの 2 段（t1-pii Pod = 業務 PII / Gateway = 技術 PII）責務分担 |
| IMP-OBS-OTEL-014 | tail_sampling 戦略（tier1 公開 100% / tier2/3 10% / error 100%） |
| IMP-OBS-OTEL-015 | tier1 公開 API の OTLP 100% sampling と保持の最小制約 |
| IMP-OBS-OTEL-016 | resource 必須属性（service.name / k8s.namespace.name / k8s.pod.name） |
| IMP-OBS-OTEL-017 | Pipeline 変更のカナリア適用（1 replica → 全台）と PR レビュー段階 |
| IMP-OBS-OTEL-018 | アプリ側接続点を環境変数 `OTEL_EXPORTER_OTLP_ENDPOINT` で固定 |
| IMP-OBS-OTEL-019 | Gateway バックエンド分岐の routing rule（metrics→Mimir / logs→Loki / traces→Tempo） |

### LGTM: Grafana LGTM Stack（20_LGTM_Stack）

| ID | 設計内容 |
|---|---|
| IMP-OBS-LGTM-020 | LGTM 4 コンポーネントを `observability-lgtm` namespace に集約配置 |
| IMP-OBS-LGTM-021 | Loki / Mimir / Tempo を StatefulSet、Grafana を Deployment + Postgres state |
| IMP-OBS-LGTM-022 | NetworkPolicy で ingress を `observability/otel-gateway` と `grafana` のみ許可 |
| IMP-OBS-LGTM-023 | S3 互換オブジェクトストレージ採用（リリース時点 MinIO / 採用初期 外部 S3） |
| IMP-OBS-LGTM-024 | 保持期間（Loki 30/365 日 / Mimir 30 日/13 ヶ月 / Tempo 14 日）の固定 |
| IMP-OBS-LGTM-025 | Grafana 匿名閲覧禁止 + Keycloak OIDC SSO 必須化 |
| IMP-OBS-LGTM-026 | datasource を Grafana → 各 backend へ直接接続、read / write 分離 |
| IMP-OBS-LGTM-027 | Postgres 日次 backup + S3 バージョニング + replication 冗長化 |
| IMP-OBS-LGTM-028 | Collector Gateway の disk queue 1 GB バッファ（backend 障害バッファリング） |
| IMP-OBS-LGTM-029 | 復旧優先順位 Mimir → Loki → Tempo → Grafana の固定 |

### PYR: Pyroscope 統合（30_Pyroscope）

| ID | 設計内容 |
|---|---|
| IMP-OBS-PYR-030 | 4 ランタイム（Rust / Go / Node / .NET）SDK push を主、tags 必須注入 |
| IMP-OBS-PYR-031 | tier1 Rust の `otel-util` crate に Pyroscope 初期化を集約 |
| IMP-OBS-PYR-032 | Linux Node に Grafana Alloy + eBPF pull を補完、CPU 5% 以下無視 |
| IMP-OBS-PYR-033 | Pyroscope は OTel Collector 外で運用、OTLP profiles GA 後に統合検討 |
| IMP-OBS-PYR-034 | Tempo span attribute `pyroscope.profile.id` で双方向 link |
| IMP-OBS-PYR-035 | Pyroscope server を `observability-lgtm` namespace に AGPL 同居配置 |
| IMP-OBS-PYR-036 | 保持 14 日 hot / 30 日 cold、長期は nightly aggregate で別保存 |
| IMP-OBS-PYR-037 | Grafana datasource 3 ビュー（Flame Graph / Diff / Profile from Trace） |
| IMP-OBS-PYR-038 | nightly diff レポートで regression 自動検出（採用初期） |
| IMP-OBS-PYR-039 | 障害時 SDK 5 分 buffer / Alloy 100 MB buffer / 重大時 CSV export |

### SLO: tier1 公開 11 API SLO/SLI（40_SLO_SLI定義）

| ID | 設計内容 |
|---|---|
| IMP-OBS-SLO-040 | tier1 公開 11 API のリスト確定と各 API の SLI 種別（avail / latency / correctness） |
| IMP-OBS-SLO-041 | Availability SLO 99.9% を全 11 API 共通の base line として固定 |
| IMP-OBS-SLO-042 | Latency SLO p99 < 500ms を tier1 同期 API に適用 |
| IMP-OBS-SLO-043 | Correctness SLI（アプリ手動 export）の運用と SLO 100% 設定 |
| IMP-OBS-SLO-044 | Mimir recording rule で 5 分粒度評価、SLO ダッシュボード自動生成 |
| IMP-OBS-SLO-045 | SLI 計測の HTTP / gRPC 統一定義（5xx / UNAVAILABLE / DEADLINE_EXCEEDED） |
| IMP-OBS-SLO-046 | SLO 違反時の DS-SW-COMP 単位影響範囲分析 |
| IMP-OBS-SLO-047 | SLO 値見直しの ADR 起票必須化（半年に 1 回 review） |

### EB: Error Budget 運用（50_ErrorBudget運用）

| ID | 設計内容 |
|---|---|
| IMP-OBS-EB-050 | 28 日 rolling window で budget 計算、15 分平均で状態判定 |
| IMP-OBS-EB-051 | 4 状態（HEALTHY / WARNING / ALERT / FROZEN）と境界（50/25/0%） |
| IMP-OBS-EB-052 | FROZEN 時 Argo CD `syncPolicy.automated.selfHeal: false` 切替 |
| IMP-OBS-EB-053 | budget 自動回復のみ許容、手動リセット禁止 |
| IMP-OBS-EB-054 | セキュリティ hotfix（CVSS 9.0+）の `hotfix/sec-` prefix bypass |
| IMP-OBS-EB-055 | ダッシュボード 4 要素 + simplified dashboard 二段提供 |
| IMP-OBS-EB-056 | Mimir 障害時は安全側 `block` 判定で deploy 抑止（pessimistic） |
| IMP-OBS-EB-057 | FROZEN 到達は post-mortem 自動起票、半年 2 回で SLO 見直し ADR |

### INC: Incident Taxonomy（60_Incident_Taxonomy）

| ID | 設計内容 |
|---|---|
| IMP-OBS-INC-060 | 可用性系 Severity SEV1〜SEV5 と CVSS 連動セキュリティ Severity の統合 |
| IMP-OBS-INC-061 | CVSS 9.0+ → SEV1（48h SLO） / 7.0+ → SEV2（7 日 SLO）の機械対応表 |
| IMP-OBS-INC-062 | Incident Tag タクソノミー（root_cause / impact_domain / detection_path） |
| IMP-OBS-INC-063 | post-mortem template と Incident Tag の必須付与 |
| IMP-OBS-INC-064 | Slack incident channel 自動作成と命名規則（#inc-YYYYMMDD-<id>） |
| IMP-OBS-INC-065 | PagerDuty escalation policy（5 分 → SRE / 15 分 → SRE Lead） |
| IMP-OBS-INC-066 | SEV1 / SEV2 の Incident Commander 任命と役割分担 |
| IMP-OBS-INC-067 | Incident timeline の機械記録（Slack thread → post-mortem 自動取込） |
| IMP-OBS-INC-068 | post-mortem の blameless 原則と書式固定 |
| IMP-OBS-INC-069 | 月次 Incident review と top 3 Incident への ADR 起票 |
| IMP-OBS-INC-070 | Incident metrics（MTTR / MTBF / 検知遅延）の Grafana 可視化 |
| IMP-OBS-INC-071 | セキュリティ Incident の 85_Identity / 80_サプライチェーン との連携経路 |

### RB: Runbook 連携（70_Runbook連携）

| ID | 設計内容 |
|---|---|
| IMP-OBS-RB-080 | ID 体系 `<tier>.<service>.<sli_kind>.<symptom>` を alert / Runbook で 1:1 採用 |
| IMP-OBS-RB-081 | リリース時点 Runbook 15 本（5 領域 × 3 本）の物理配置 |
| IMP-OBS-RB-082 | Runbook 5 セクション固定（症状 / 影響範囲 / 一次対応 / 根本対応 / 検証） |
| IMP-OBS-RB-083 | CI lint 3 種（alert↔Runbook 1:1 / 5 セクション / メタデータ）の `ci-overall` 必須 |
| IMP-OBS-RB-084 | Alertmanager rule の `annotations.runbook_url` 必須付与 |
| IMP-OBS-RB-085 | Runbook 起動 3 経路（PagerDuty / Grafana / post-mortem）の公式化 |
| IMP-OBS-RB-086 | post-mortem 24 時間以内の Runbook 改訂 PR 必須化と `last_updated` 検証 |
| IMP-OBS-RB-087 | 段階拡大（リリース時点 15 → 採用初期 30 → 採用拡大期 50） |
| IMP-OBS-RB-088 | 採用拡大期 50 本超過時の Runbook 索引 2 軸検索化 |
| IMP-OBS-RB-089 | GitHub 障害時のバックアップ経路（MinIO 同期 + ローカル `~/.k1s0-runbooks/`） |

## ADR 逆引き

| ADR | 関連 IMP-OBS ID |
|---|---|
| ADR-OBS-001（Grafana LGTM 採用） | POL-002 / LGTM-020〜029 / PYR-030〜039 / EB-050〜057 / RB-080〜089 |
| ADR-OBS-002（OTel Collector 採用） | POL-001 / OTEL-010〜019 / LGTM-026 / RB-084 |
| ADR-OBS-003（Incident Taxonomy 統合、本章で起票） | POL-004 / INC-060〜071 / RB-080 |
| ADR-0003（AGPL 分離） | POL-002 / LGTM-020 / LGTM-022 / PYR-035 |
| ADR-REL-001（Progressive Delivery 必須） | EB-051〜052 / SLO-040〜047（PD 判定の入力源） |
| ADR-CICD-001（GitHub Actions） | EB-051 / RB-083 |
| ADR-DX-001（DX メトリクス章分離） | POL-007（DORA は 95 章） |

## DS-SW-COMP 逆引き

| DS-SW-COMP | 関連 IMP-OBS ID |
|---|---|
| DS-SW-COMP-085（OTel Collector Gateway） | OTEL-010〜019 / LGTM-022 / LGTM-028 / OTEL-013（PII Pod 連携点） |
| DS-SW-COMP-141（Observability + Security 統合監査） | INC-060〜071 / RB-080〜089 / EB-054（セキュリティ hotfix bypass） |
| DS-SW-COMP-129（tier1 Rust 自作領域） | OTEL-018（otel-util 接続点） / PYR-031（Pyroscope 初期化集約） / PYR-034（Tempo span link） |
| DS-SW-COMP-124（tier1 Go Dapr ファサード） | POL-001（Dapr 経由のトレース propagation の主体） |
| DS-SW-COMP-135（配信系インフラ） | OTEL-019（routing） / LGTM-021〜023（StatefulSet/NetworkPolicy/S3） / EB-051〜052（Argo Rollouts/Argo CD 連動） / RB-083〜084（CI lint / Alertmanager 統合） |

OBS 章 75 件は単一の DS-SW-COMP には収まらず、085（Gateway）/ 141（Incident 統合監査）/ 129（Rust 自作）/ 124（Go Dapr）/ 135（配信系）の 5 面に分散する。これは「観測性は全 DS-SW-COMP を横断する横軸の機能」であることを反映した結合構造で、本章のどの ID も最低 1 つの DS-SW-COMP に必ず帰着する設計を保つ。

## NFR 逆引き

| NFR | 関連 IMP-OBS ID |
|---|---|
| NFR-A-CONT-001（SLA 99%） | SLO-041 / EB-051 / EB-052 |
| NFR-I-SLO-001（内部 SLO 99.9%） | SLO-040〜047 / EB-050〜057 |
| NFR-I-SLI-001（Availability SLI） | SLO-040 / SLO-045 / OTEL-019 |
| NFR-B-PERF-001（p99 < 500ms） | SLO-042 / PYR-030〜039 / OTEL-014 |
| NFR-C-NOP-001（小規模運用） | LGTM-021 / RB-082 / INC-064〜065 |
| NFR-C-NOP-002（可視性） | POL-001 / OTEL-016 / LGTM-026 / INC-070 / RB-085 |
| NFR-C-IR-001（Severity 別応答） | INC-060〜067 / RB-080〜089 |
| NFR-C-IR-002（Circuit Breaker / 自動 rollback） | EB-051〜052 |
| NFR-C-MGMT-001（設定 Git 管理） | LGTM-020〜023 / RB-081〜083 |
| NFR-C-MGMT-002（変更監査） | EB-054 / INC-068〜069 |
| NFR-E-MON-001（特権監査） | LGTM-025（Grafana SSO） / INC-071 |
| NFR-E-OPR-001（運用性） | RB-080〜089 / INC-064〜067 |

NFR-I-SLO-001 / NFR-I-SLI-001 / NFR-B-PERF-001 が本章の主目的。NFR-C-IR-001（Severity 別応答）と NFR-E-OPR-001（運用性）は INC + RB の連動で支える構造。

## トレーサビリティ更新運用

新規 IMP-OBS-ID 採番時の手順:

1. 本ファイルの「全 75 件の ID 一覧」を更新（接頭辞表とレンジ管理表も同時更新）
2. `docs/05_実装/99_索引/00_IMP-ID一覧/01_IMP-ID台帳_全12接頭辞.md` の OBS 行を更新
3. 該当 ADR を `docs/05_実装/99_索引/10_ADR対応表/01_ADR-IMP対応マトリクス.md` で更新
4. 該当 NFR を `docs/05_実装/99_索引/30_NFR対応表/01_NFR-IMP対応マトリクス.md` で更新
5. DS-SW-COMP-124 / 129 / 135 を `docs/05_実装/99_索引/20_DS-SW-COMP対応表/01_DS-SW-COMP-IMP対応マトリクス.md` で更新

5 ファイルを同一 PR で更新することで、索引のドリフトを防ぐ。漏れた場合は PR レビュー段階で `docs-review-checklist` Skill が指摘する設計。
