---
id: detail.cross_pii.audit_ingest_gap_monitor
axis: cross_pii
phase: cross_cutting
kind: cross_cut_spec
status: published
version: 1.0.0
depends_on:
  - detail.security.threat_model_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [B, C, D, E]
  proof_classes: []
related_axes:
  - security
  - ops
---

# audit ingest gap monitor + heartbeat（v1）

## 位置づけ
- [security 監査方針](../../03_概要設計/07_security設計方針/05_監査方針.md) / [脅威モデル適合仕様](../01_適合仕様/15_脅威モデル適合仕様.md) / [ops_edge_cluster](10_ops_edge_cluster.md) と双方向 lock
- audit_event hash chain が「書かれた行同士の改竄検知」しか保証できない（ingest 段の loss は chain に現れない）ことを honest に受け止め、ingest pipeline の gap / drop を別経路で検出する仕組みを定義する

## 不可避性
- hash chain は cryptographic forgery を検知できるが、actor が ingest pipeline（application emit / pgaudit / kube-apiserver audit / Vector）を停止すれば chain は止まらず空白を作るのみ。停止期間中の audit_event は存在しないため、chain は「prev=NULL の新 chain」で継続される
- 「forgery impossibility / 取りこぼしゼロ」表現は強すぎる。実際は「tamper-evident（書かれた行の改竄検知）」+ 「ingest gap detection（heartbeat insert + gap monitor）」の二経路で初めて閉じる

## 役割

### 役割 AGM-A: heartbeat insert
- 各 ingest source（application emit / pgaudit / kube-apiserver audit / Vector）が **30 秒ごと**に synthetic audit_event `ingest_heartbeat` を emit（至高路線: 60s より 2 倍高頻度で gap を早期検出）
- heartbeat event は通常の audit_event と同じ chain に混入し、ClickHouse audit table に書込まれる
- heartbeat に含むフィールド: `source_id`, `emitted_at`, `sequence_number`, `source_instance_id`

### 役割 AGM-B: gap monitor
- ClickHouse の audit table を 30 秒ごと query:
  ```sql
  SELECT source_id, MAX(emitted_at) as latest
  FROM audit_event
  WHERE event_type = 'ingest_heartbeat'
  GROUP BY source_id
  ```
- `latest` が現在時刻 - 90 sec を超えた source は gap と判定（30 sec interval + 60 sec 余裕）
- `audit_ingest_gap_seconds` metric を Prometheus に export、SLO 化（gap > 0 で page）

### 役割 AGM-C: kube-apiserver drop 検出 + audit backend 二重化
- apiserver の `apiserver_audit_event_dropped_total` が > 0 で page
- kube-apiserver audit policy は backend を **file (rotated, on-host PVC) + webhook の二重出力** で構成する（`--audit-log-path` + `--audit-webhook-config-file` 並列）
- webhook 先は Vector の disk-backed persistent buffer（`buffer.type = disk`、容量試算は 1 source × peak event-rate × 24h を想定）で drop 期間を吸収する
- drop 復旧は (a) webhook 先 Vector persistent buffer の再 ingest、(b) on-host file backend の rotated log からの再投入（CI batch job 日次実行）の二経路で行う
- audit-only field（actor / requestURI / responseObject）は etcd 側に存在しないため、etcd snapshot からは原理的に再構築できない。本機構は etcd snapshot 経路に依存しない

### 役割 AGM-D: SPOF 対策
- gap monitor 自身の障害は、ops-edge cluster（[ops_edge_cluster](10_ops_edge_cluster.md)）から監視。target cluster 側 monitor が idle になると ops-edge から page
- gap monitor の出力（gap event）は target cluster / ops-edge cluster の両方に書込（dual-write）

## honest な仕様文言（適用箇所）
- 「forgery impossibility」を「tamper-evident（書かれた行の改竄は cryptographic に検知可能、ingest 段の loss は別経路で検出）」に置換
- 「取りこぼしゼロ」を「ingest gap > 0 → CI fail + page」に置換
- 「100% 保存」を「sampling 禁止 + drop 検出 + audit backend file + webhook 二重出力 + webhook 先 Vector persistent buffer (disk-backed) + rotated file log の再投入経路」に置換。kube-apiserver audit-only field は etcd snapshot から原理的に復元不能であるため、本機構は etcd snapshot 経路に依存しない（依存しない旨を honest に明示）

## 適用 ingest source の網羅

### source_id 列挙
| source_id | 内容 |
|---|---|
| `tier1_library_emit` | tier1 Library 内 emit |
| `tier2_application_emit` | tier2 Service 内 emit |
| `postgresql_pgaudit` | PostgreSQL pgaudit extension |
| `kube_apiserver_audit` | k8s apiserver audit policy |
| `vector_pipeline` | Vector → ClickHouse pipeline |
| `openbao_audit` | OpenBao audit device |
| `keycloak_event_listener` | Keycloak event listener SPI |
| `cosign_rekor_log` | cosign / Rekor transparency log |

- 各 source は heartbeat を 30 sec ごと emit する責務を持つ
- 新 source を追加する際は heartbeat 実装を mandatory として PR review checklist 化

## 副作用

### heartbeat insert 量
- `source_id` ごとに `source_instance_id` 軸を持つ
- 各 source の instance 数は環境ごとに異なる（典型値: tier1_library_emit ≈ tier1 server pod 数 (例 50)、tier2_application_emit ≈ tier2 service pod 数 (例 200)、postgresql_pgaudit ≈ CNPG instance 数 (例 6)、kube_apiserver_audit ≈ apiserver instance 数 (例 3)、vector_pipeline ≈ Vector pod 数 (例 30)、openbao_audit ≈ OpenBao instance 数 (例 3)、keycloak_event_listener ≈ Keycloak instance 数 (例 3)、cosign_rekor_log ≈ Rekor instance 数 (例 3)）
- 上記 reference 環境（合計 instance 数 ≈ 298）で 1 instance × 30 sec × 120 = 120 events/hour/instance、24h で 298 × 120 × 24 ≈ 858,240 events/day（30s 間隔換算）
- ClickHouse の typical write throughput は 100k events/sec オーダーであり、本機構 heartbeat の流量比は 0.01% / hour オーダー（無視可能）

### gap monitor の自己 SPOF
- ops-edge cluster で担保するため、ops-edge cluster は本機構の運用 prerequisite

### drop 復旧の rotated file backend 再投入
- CI batch job で日次実行
- Vector persistent buffer 経路は webhook 復旧時に自動 drain
- 再投入コストは drop 頻度に比例（通常運用では drop はゼロ前提、buffer 容量試算で peak burst を吸収）
- audit backend file は on-host PVC を消費するため、PV sizing（peak event-rate × rotation 保持期間）を node spec に折り込む

## 整合
- 整合 1: [security 監査方針](../../03_概要設計/07_security設計方針/05_監査方針.md) の「forgery impossibility」表現は「tamper-evident + ingest gap detection」に書き換え
- 整合 2: 同方針の「100% 保存」表現は「sampling 禁止 + drop 検出 + secondary etcd audit log redundancy」に書き換え
- 整合 3: [脅威モデル適合仕様](../01_適合仕様/15_脅威モデル適合仕様.md) の audit mitigation pointer に「ingest gap detection」を追加（detective class）
- 整合 4: [ops_edge_cluster](10_ops_edge_cluster.md) の役割 OE-D（dead-man）と本機構の SPOF 対策が二重に閉路を作ることを明記
- 整合 5: [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md) の Chaos drill に「ingest pipeline kill → 60 sec 以内に gap 検出 page（30s interval × 2 周期以内）」を追加

## 関連参照
- [security 監査方針](../../03_概要設計/07_security設計方針/05_監査方針.md)
- [脅威モデル適合仕様](../01_適合仕様/15_脅威モデル適合仕様.md)
- [security 強制機構](../02_強制機構/06_security強制機構.md)
