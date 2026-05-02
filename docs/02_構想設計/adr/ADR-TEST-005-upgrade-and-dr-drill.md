# ADR-TEST-005: Upgrade drill（kubeadm N-2→N→N+1）と DR drill（既存 barman-cloud / etcdctl / GitOps 経路の実機検証）を採用後の運用拡大時に実施する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: 起案者 / 採用検討組織 / SRE / 運用チーム（採用後）

## コンテキスト

ADR-TEST-001 で確定した Test Pyramid の最上位 E2E 層（5%）と、ADR-TEST-003 / 004 で確定した Conformance / Chaos の各実施経路に続き、**Upgrade（K8s minor version 移行）と DR（Disaster Recovery）の drill 実施方針** を本 ADR で確定する（L4 E2E 自動化経路 ADR は撤回済、テスト基盤刷新後に新 ADR で再策定）。Upgrade と DR は「机上の手順書がある」と「drill で実走検証されている」の間に決定的な差がある領域で、本番リリース後に「Velero backup が実は復元できなかった」「kubeadm upgrade で deprecated API 互換崩壊」のような最悪事態を防ぐにはローカル / staging で drill を実施する仕組みが必要である。

既存設計の調査で以下が判明した:

- **`docs/02_構想設計/01_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md`** で **コンポーネント別バックアップ戦略**が確定済:
  - PostgreSQL: CloudNativePG の `barman-cloud`、MinIO 保管、RPO 数秒 / RTO 15 分
  - etcd: `etcdctl snapshot save` 日次 + MinIO 保管、RPO 24h / RTO 30 分
  - Harbor: DB は CloudNativePG / イメージは MinIO、RTO 30 分
  - Keycloak: DB + Realm Export（JSON を Git 管理）、RTO 15-30 分
  - OpenBao: 別途
- **`docs/02_構想設計/01_アーキテクチャ/02_可用性と信頼性/06_壊滅的障害シナリオ/02_etcd全ノード障害.md`** で **etcd 全壊時の 2 経路復旧**が確定済:
  - 経路 A（snapshot あり）: `etcdctl snapshot restore`、RTO 30 分
  - 経路 B（snapshot なし）: `tofu apply` + `kubeadm init` + Argo CD 同期、RTO 4 時間
- **ADR-INFRA-001** で kubeadm + Cluster API 採用、kubeadm 公式 upgrade 手順が前提
- **ADR-DATA-003** で MinIO が backup target として採用済
- **既存設計に Velero は登場しない**。バックアップは「コンポーネント別の専用ツール」で構成されている

つまり k1s0 の DR 戦略は **「Velero 等の汎用 K8s resource backup ツールに依存しない、コンポーネント別の最適解を組み合わせる構成」** が既に確定している。本 ADR の射程は **新規ツール導入ではなく、既存戦略の drill 実施方針** とする。

drill 実施方針として以下が未確定:

- Upgrade drill の頻度・対象 cluster・成功判定基準・release tag との関係
- DR drill の頻度・経路網羅（A / B / コンポーネント別 restore）・成功判定基準
- drill 失敗時の対応（再 drill / Runbook 修正 / ADR 改訂のトリガ）
- 「四半期 Chaos Drill」（ADR-OPS-001）との並列運用

選定では以下を満たす必要がある:

- **既存設計（barman-cloud / etcdctl / Realm Export / GitOps）と完全整合**（新規ツールで覆さない）
- **ADR-INFRA-001 の kubeadm 公式 upgrade と整合**
- **ADR-OPS-001 の四半期 Chaos Drill と並列で運用可能**
- **採用後の運用拡大時の段階導入**（リリース時点では実装ゼロ）
- **RTO 实測値の継続検証**（机上 RTO（30 分 / 4 時間 / 15 分）が drill で実証されること）

## 決定

**Upgrade drill と DR drill を採用後の運用拡大時に実施し、両者とも既存設計（barman-cloud / etcdctl / Realm Export / GitOps）の drill 実施方針として位置づける。Velero 等の汎用 K8s resource backup ツールは新規導入しない。**

### 1. Upgrade drill

- **頻度**: 月次（K8s upstream の minor version リリースサイクルと同期、年 3 回の minor + 月次パッチに対応）
- **対象 cluster**: staging（採用後の運用拡大時に常設、kubeadm + 3 control-plane HA + Cluster API 構成、ADR-INFRA-001 と一致）
- **手順**: kubeadm 公式 upgrade plan / apply / node 経路（control-plane 1 → 2 → 3 → worker drain → upgrade → uncordon）
- **対象バージョン**: N-2 → N-1 → N → N+1 の 3 段階移行を staging で必ず実走（production への適用は staging 完了後）
- **成功判定**: ① upgrade 中の API 可用性 ≥ 99%、② 既存 Deployment の Pod が継続稼働、③ upgrade 完了後に L4 standard E2E（テスト基盤刷新後の新 ADR で再策定）+ L5 conformance（ADR-TEST-003）が PASS、④ 所要時間が想定値（control-plane 各 5 分 / worker 各 3 分 / 全体 30 分以内）に収まる
- **release tag との関係**: production cluster の K8s upgrade 直前に staging で本 drill を必須実施。drill が PASS しないと production upgrade を起動しない
- **失敗時**: 失敗した step の Runbook（`ops/runbooks/RB-UPGRADE-*`）を更新、ADR-INFRA-001 の改訂が必要なら別 ADR 起票

### 2. DR drill

- **頻度**: 四半期（ADR-OPS-001 の Chaos Drill と同期、シナリオ別に四半期ローテーション）
- **対象 cluster**: staging（破壊的検証のため production では実施しない）
- **シナリオ**: 4 経路を四半期 1 経路ずつローテーション
  - **経路 A**: etcd snapshot 復旧（既存設計、RTO 30 分）— 第 1 四半期
  - **経路 B**: GitOps 完全再構築（既存設計、RTO 4 時間）— 第 2 四半期
  - **経路 C**: PostgreSQL barman-cloud restore（既存設計、RTO 15 分）— 第 3 四半期
  - **経路 D**: Keycloak Realm Export restore（既存設計、RTO 15-30 分）— 第 4 四半期
- **成功判定**: ① 復旧後の resource diff（`kubectl get` 比較）が想定範囲内、② PV データ整合性（PostgreSQL は WAL リプレイ後の整合 / etcd は snapshot 取得時刻以降のロスト範囲明示）、③ 所要時間が机上 RTO 値（30 分 / 4 時間 / 15 分）を超えない、④ 復旧後の L4 standard E2E が PASS
- **失敗時**: RTO 超過した経路の Runbook（`ops/runbooks/RB-DR-001〜004`）を更新、機材性能不足が原因なら ADR-INFRA-001 の HA 要件を見直す別 ADR を起票

### 3. Velero 不採用の確定

- 既存設計（`01_障害復旧とバックアップ.md`）が **コンポーネント別バックアップ**（barman-cloud / etcdctl / Realm Export）を選択済
- Velero は K8s resource manifest + PV snapshot を統合 backup する汎用解だが、既存戦略と二重化する
- Velero を採用すると ADR-DATA-001（CloudNativePG）/ ADR-DATA-003（MinIO）/ 既存設計の SoT が複数化し drift リスクが発生
- 本 ADR で Velero 不採用を明示することで、後続 PR で「Velero を入れたい」議論が出た際の判断軸を残す

### 4. 段階導入計画

| 段階 | 実装内容 |
|------|---------|
| リリース時点 | 本 ADR + Runbook 雛形（`ops/runbooks/RB-UPGRADE-*` / `RB-DR-001〜004`）の skeleton 作成 |
| 採用初期 | staging cluster の構築準備、4 経路の Runbook 詳細化（dry-run） |
| 採用後の運用拡大時 | Upgrade drill 月次開始、DR drill 四半期ローテーション開始 |
| 採用側のマルチクラスタ移行時 | DR drill にマルチクラスタ failover 経路（cluster A 全壊 → cluster B への業務移行）を追加 |

## 検討した選択肢

### 選択肢 A: 既存設計の drill 実施方針確定（採用）

- 概要: barman-cloud / etcdctl / Realm Export / GitOps の既存戦略を drill で実走検証する方針確定。新規ツール不導入
- メリット:
  - **既存設計（`01_障害復旧とバックアップ.md` / `02_etcd全ノード障害.md`）と完全整合**
  - ADR-DATA-001 / 003 / ADR-INFRA-001 / ADR-OPS-001 と整合
  - Velero / Stash / K8up 等の追加 OSS 学習・運用工数ゼロ
  - 4 経路ローテーションで採用組織の SRE が四半期に 1 経路を学べる構造
  - 机上 RTO 値（30 分 / 4 時間 / 15 分）の継続実証経路が確立
- デメリット:
  - K8s 全体の resource manifest を一括 backup する経路がないため、`kubectl get -o yaml | apply` 級の即時復旧は不可（GitOps 経路 B でカバーするが RTO 4 時間）
  - drill 実施工数が起案者 / SRE に集中、採用拡大時に分散する仕組みが要る

### 選択肢 B: Velero + minio + etcdctl 統合

- 概要: Velero を新規導入し K8s resource backup を統合、minio + etcdctl と並列運用
- メリット:
  - K8s resource manifest + PV snapshot の統合 backup が成立
  - Velero CRD（Backup / Schedule / Restore）で宣言的管理
  - CNCF Incubating の安定性
- デメリット:
  - **既存設計を覆す**: barman-cloud / etcdctl / Realm Export と Velero PV snapshot が二重化、SoT が割れる
  - ADR-DATA-001（CloudNativePG）の barman-cloud と Velero の PostgreSQL backup が競合
  - 既存設計の RTO 机上値（PostgreSQL 15 分 / etcd 30 分）は専用ツール前提で算出されており、Velero 統合では RTO が異なる
  - 採用組織が「k1s0 採用 = Velero も学ぶ」となり学習コスト増

### 選択肢 C: drill 実施なし（机上記述のみ）

- 概要: 既存設計の Runbook を maintain するに留め、定期 drill を実施しない
- メリット:
  - 実装工数ゼロ、staging cluster 維持費なし
- デメリット:
  - **机上 RTO が実測で裏付けられない**: 「RTO 30 分」が本当に達成可能か本番災害まで未検証
  - 採用検討組織から「DR drill 実施有無」を問われた際、「実施していない」となり信頼が低下
  - ADR-OPS-001 の四半期 Chaos Drill との並列運用の機会損失（drill 実施インフラを共有できる）
  - Runbook の陳腐化が drill フィードバックなしで進行、いざという時に手順が古い状態

### 選択肢 D: 専用 DR 自動化ツール導入（例: Kasten K10）

- 概要: Veeam Kasten K10 等の商用 / OSS 専用 DR ツールで K8s 全体を backup-restore 自動化
- メリット:
  - DR drill が tool 内蔵の機能で自動実行
  - PV snapshot / cross-cluster replication / immutability 等のエンタープライズ機能
- デメリット:
  - 商用ライセンス（Kasten K10 は機能制限フリー版あるが本格利用はライセンス）、ADR-0003 OSS 方針と乖離
  - 既存戦略を完全に置き換える大変更で、ADR-DATA-* / ADR-INFRA-001 の改訂が連鎖的に必要
  - 採用組織が k1s0 と一緒に Kasten を採用する強制になり、技術選定の自由度を奪う

## 決定理由

選択肢 A（既存設計の drill 実施方針確定）を採用する根拠は以下。

- **既存設計との完全整合**: `docs/02_構想設計/01_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md` で確定済の barman-cloud / etcdctl / Realm Export / GitOps 戦略と、`02_etcd全ノード障害.md` で確定済の経路 A / B を本 ADR は「drill 実施方針」レベルで補完する。選択肢 B（Velero）は既存戦略を覆す、選択肢 D（Kasten）は完全置換で連鎖的 ADR 改訂が要る
- **新規 ADR 改訂の連鎖を最小化**: 選択肢 A は既存 ADR-DATA-001 / 003 / ADR-INFRA-001 を改訂せずに済む。drill 実施方針という単一の決定で完結し、k1s0 の SoT 構造を保つ。選択肢 B は ADR-DATA-001（CloudNativePG）の barman-cloud 採用根拠を改訂せざるを得ない
- **ADR-OPS-001 の Chaos Drill との並列運用**: 選択肢 A は四半期 DR drill を ADR-OPS-001 の四半期 Chaos Drill とローテーション枠で共有でき、staging cluster の維持費 / SRE の drill 実施工数を最適化できる。選択肢 C（drill なし）はこの並列運用機会を失う
- **机上 RTO 実証の継続性**: 既存設計が公開する RTO 値（PostgreSQL 15 分 / etcd 30 分 / GitOps 完全再構築 4 時間）は採用検討組織が k1s0 の RTO commitment（NFR-A-CONT-001 RTO 4 時間）を信じる根拠となる。drill による継続実証が無いとこれらの数値が「ただの机上値」になり、testing maturity 評価が低下する
- **Velero 不採用の積極的選択**: 選択肢 A は Velero 不採用を本 ADR で明示することで、後続 PR で「Velero を入れたい」議論が出た際の判断軸（既存戦略との二重化禁止）を残す。これは構造的な意思決定保全
- **段階導入の整合**: リリース時点では実装ゼロ、採用後の運用拡大時で実走開始という段階導入が ADR-TEST-001 の Chaos / DAST 保留と整合し、ADR-TEST-004 の LitmusChaos 採用後の運用拡大時導入と並列。リリース時点での過剰投資を避けつつ、ADR で決定の記録を残す

## 影響

### ポジティブな影響

- 既存設計（`01_障害復旧とバックアップ.md` / `02_etcd全ノード障害.md`）の戦略が drill で実走検証され、机上 RTO 値が実測値で裏付けられる
- ADR-DATA-001 / 003 / ADR-INFRA-001 / ADR-OPS-001 を改訂せず、既存 SoT 構造を保ったまま drill 実施方針を補完できる
- ADR-OPS-001 の四半期 Chaos Drill と DR drill のローテーション枠で staging cluster / SRE 工数を共有でき、運用コストが最適化される
- Velero 不採用を本 ADR で明示することで、後続の技術選定議論で既存戦略との二重化を防ぐ判断軸が確立する
- DR drill 4 経路（A / B / C / D）が四半期ローテーションで採用組織の SRE に学習機会を提供し、バス係数 2（ADR-OPS-001）と整合
- Upgrade drill が release tag 直前必須となり、production upgrade 失敗の事前検出経路が確立する

### ネガティブな影響 / リスク

- staging cluster の常設維持費（採用後の運用拡大時、3 control-plane HA + Cluster API 構成）が発生。採用組織のインフラ予算で吸収する前提
- drill 実施工数が起案者 / SRE に集中、採用拡大時にローテーション当番制を `ops/runbooks/RB-DR-001〜004` で確立する必要
- K8s 全体 resource manifest の一括即時復旧経路がない（GitOps 経路 B で代替するが RTO 4 時間）。マルチクラスタ failover が必要な状況では選択肢 D（Kasten 等）の再検討が必要だが、これはマルチクラスタ移行時の別判断
- 4 経路の四半期ローテーションは経路 A（etcd snapshot）の検証頻度が年 1 回となり、頻度不足のリスク。対策として ADR-OPS-001 の Chaos Drill で「Chaos × etcd 全壊」の組み合わせシナリオを別途実施することで、経路 A は実質年 2 回以上の検証経路を確保
- Upgrade drill の月次実施で staging cluster 占有時間が月 2-4 時間発生。CI runner と staging cluster の予算管理が要る
- DR drill 経路 B（GitOps 完全再構築）は手作業の `kubectl apply` がない前提で RTO 4 時間が成立する。本番運用で手作業 apply を許容すると経路 B の整合性が崩れ drill の意味が薄れるため、ADR-POL-002（local-stack SoT）と同等の運用規律が production にも要る

### 移行・対応事項

- リリース時点で `ops/runbooks/RB-UPGRADE-001-kubeadm-minor-upgrade.md`（kubeadm 公式 upgrade 手順）と `ops/runbooks/RB-DR-001〜004`（4 経路別の DR 手順）の skeleton を新設、8 セクション形式（ADR-OPS-001 準拠）
- 採用初期で staging cluster の Cluster API 構成を `infra/environments/staging/` に整備、kubeadm + 3 control-plane HA を確立
- 採用初期で Runbook 4 経路の dry-run を実施、手順抜けや所要時間予測を更新
- 採用後の運用拡大時で Upgrade drill 月次 schedule（cron で staging cluster に自動 trigger）を整備
- 採用後の運用拡大時で DR drill 四半期ローテーション（経路 A → B → C → D の年次サイクル）を起動、結果を `docs/40_運用ライフサイクル/dr-drill-results.md`（採用初期で初回作成）に月次サマリで記録
- ADR-OPS-001 の「四半期 Chaos Drill」と本 ADR の「四半期 DR Drill」のローテーション統合を `ops/drill-schedule.md`（新設）で月次カレンダー化
- ADR-DATA-001（CloudNativePG）の「帰結」に「barman-cloud restore は ADR-TEST-005 経路 C で四半期 drill」を追記する relate-back
- ADR-INFRA-001 の「帰結」に「kubeadm upgrade は ADR-TEST-005 で月次 drill」を追記する relate-back
- ADR-DATA-003（MinIO）の「帰結」に「DR backup target は引き続き MinIO（ADR-TEST-005 で確認）、Velero 等の追加 backup 層は不採用」を追記する relate-back
- 既存 `docs/02_構想設計/01_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md` の「Velero 等」言及がもしあれば本 ADR の relate-back で確認・訂正（grep で確認、無ければ無し）
- 採用後の運用拡大時で drill 実績の Grafana dashboard（`infra/observability/grafana/dashboards/dr-drill-results.json`）を整備、ADR-OBS-002 と整合

## 参考資料

- ADR-TEST-001（Test Pyramid + testcontainers）— Chaos / DAST と並列で本 ADR が Upgrade / DR drill を扱う位置づけ
- ADR-TEST-003（CNCF Conformance）— L5 conformance が upgrade 後の整合確認に使われる
- L4 standard E2E は drill 後の整合確認に使われる予定（ADR はテスト基盤刷新後に再策定）
- ADR-TEST-004（LitmusChaos）— 同じ採用後の運用拡大時段階導入の前例
- ADR-INFRA-001（kubeadm + Cluster API）— Upgrade drill の前提
- ADR-DATA-001（CloudNativePG）— 経路 C（PostgreSQL restore）の前提
- ADR-DATA-003（MinIO）— DR backup target、Velero 不採用の根拠
- ADR-OPS-001（Runbook 標準化）— DR Runbook の形式、Chaos Drill との並列運用
- 構想設計 `02_構想設計/01_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md` — DR 戦略の正典
- 構想設計 `02_構想設計/01_アーキテクチャ/02_可用性と信頼性/06_壊滅的障害シナリオ/02_etcd全ノード障害.md` — 経路 A / B の正典
- 概要設計 `04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md` — Runbook 目録
- NFR-A-CONT-001（HA / RTO 4 時間）— 機能保全の要件
- NFR-A-DR-002（RPO / バックアップ）
- NFR-A-REC-002（復旧可能性検証）
- 関連 ADR（採用検討中）: ADR-TEST-007（テスト属性タグ + 実行フェーズ分離）。観測性 E2E はテスト基盤刷新後に新 ADR で再策定
