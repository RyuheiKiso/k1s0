# 06. FMEA 分析

本書は k1s0 プラットフォームの主要 15 コンポーネントについて、故障モード・影響・原因分析（FMEA: Failure Mode and Effects Analysis）を実施し、各コンポーネントが故障した際の影響度・発生頻度・検出容易性を RPN（Risk Priority Number）で数値化する。OR-INC-007（FMEA 実施）の具体化であり、インシデント対応フロー（OR-INC-002）の Runbook 整備の基礎資料となる。

## 本書の位置付け

プラットフォームが稼働を始めると、コンポーネントのどこが落ちても「それに対する初動が決まっていない」事態が許されない。Runbook 整備（NFR-C-NOP-003）の前提として「何が起きうるか」を網羅的に列挙する必要がある。FMEA は信頼性工学の標準手法で、製造業・航空宇宙・医療機器で使われてきた体系的な故障分析プロセスである。本書はこれをソフトウェアプラットフォームに適用し、tier1 API・データ層・ネットワーク層・コントロールプレーンの代表的故障モードを整理する。

RPN は 1〜1000 のスケールで、以下 3 つの評価軸の積で算出する。

- **Severity（影響度）1〜10**: 故障が発生した時の業務影響の大きさ
- **Occurrence（発生頻度）1〜10**: その故障モードが発生する頻度
- **Detection（検出容易性）1〜10**: 故障発生を早期検知できる容易さ（10 は検出困難、1 は即座検知）

RPN が高いほど優先的な対策が必要。100 以上は要即時対策、50〜99 は要計画的対策、49 以下は監視継続で許容可能と分類する。

## 評価対象コンポーネント

k1s0 の主要 15 コンポーネントを対象とする。各コンポーネントに代表的な 2〜3 故障モードを挙げる。

### 1. tier1 ファサード層（Go SDK + Rust 自作）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| gRPC パニック | コードバグ・nil 参照 | 当該 tier1 Pod 停止、HPA で復旧 | 7 | 3 | 2 | 42 | 起動前 smoke test、sentry 連携 |
| メモリリーク | ライブラリバグ・意図しない参照保持 | OOM Kill、Pod 再起動 | 5 | 2 | 3 | 30 | 定期 pprof、Prometheus メモリ監視 |
| デッドロック | goroutine/Rust async バグ | 応答停止、タイムアウト | 8 | 2 | 5 | 80 | goroutine dump 自動取得、lease timeout |

### 2. Istio Ambient（ztunnel / waypoint）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| ztunnel クラッシュ | Ambient 成熟度・IPtables 干渉 | Node 単位で通信不通、復旧まで N 秒 | 9 | 3 | 2 | 54 | Node 冗長、ztunnel liveness probe |
| 証明書期限切れ | cert-manager 障害 | mTLS 失敗、全通信不能 | 10 | 1 | 2 | 20 | 期限 30 日前アラート、自動更新 |
| ポリシー誤設定 | AuthorizationPolicy 誤り | 特定経路の遮断 | 7 | 3 | 4 | 84 | Kyverno 事前検証、staging 先行適用 |

### 3. CloudNativePG（PostgreSQL）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| プライマリ障害 | Pod crash、Node 障害 | フェイルオーバー完了まで 30 秒停止 | 8 | 2 | 2 | 32 | streaming replication、自動昇格 |
| レプリ遅延 | ネットワーク・ディスク I/O 低下 | RPO 超過、データ不整合リスク | 9 | 3 | 3 | 81 | `pg_stat_replication` 監視、LAG 閾値 |
| ディスクフル | 容量設計ミス・想定外増加 | 書込停止、読込は可能 | 8 | 3 | 2 | 48 | 使用率 70/85/95% アラート、自動 PVC 拡張 |

### 4. Strimzi Kafka

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| ブローカー障害 | Pod crash、JVM OOM | 他ブローカーで継続（replication 3）、復旧中 P99 悪化 | 6 | 3 | 2 | 36 | ISR 監視、MirrorMaker2 で DR |
| コンシューマラグ拡大 | Subscriber 側の処理遅延 | イベント処理遅延、下流業務滞留 | 7 | 4 | 3 | 84 | ラグ閾値アラート、HPA 連動 |
| パーティション偏り | Key 分布不均衡 | 特定ブローカー過負荷 | 5 | 3 | 5 | 75 | partition rebalance、key 設計レビュー |

### 5. Valkey（KV Store / Cache）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| インスタンス障害 | Pod crash | フェイルオーバー 10 秒、セッション切断 | 6 | 3 | 2 | 36 | Sentinel 冗長、client retry |
| メモリ圧迫 | キー増加・TTL 設定ミス | evict 発生、キャッシュヒット率低下 | 5 | 4 | 3 | 60 | maxmemory policy、TTL 監査 |
| ネットワーク分断 | VLAN 設定変更 | 一時的な切断、リトライで自動復旧 | 6 | 2 | 3 | 36 | multi-AZ 構成（VM 単位） |

### 6. MinIO（オブジェクトストレージ）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| ディスク障害 | HDD/SSD 故障 | erasure coding で継続、復旧作業要 | 6 | 4 | 2 | 48 | 月次ドライブチェック、予備機確保 |
| バケットポリシー誤り | IAM 設定ミス | アクセス不可、または過剰権限 | 7 | 3 | 4 | 84 | Kyverno バリデーション、定期監査 |
| 容量枯渇 | 容量計画不足 | 書込失敗、読込は可能 | 8 | 2 | 2 | 32 | 使用率 80% アラート、ライフサイクル設定 |

### 7. Temporal（Workflow）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| Determinism 違反 | ワークフローコード非決定 | Replay 失敗、ワークフロー停止 | 9 | 3 | 5 | 135 | workflow replay test 必須、CI/CD で検証 |
| Task Queue 滞留 | Worker 不足 | ワークフロー進行遅延 | 6 | 4 | 2 | 48 | queue depth 監視、HPA |
| 永続化層障害 | Postgres 障害 | ワークフロー状態更新不可 | 9 | 2 | 2 | 36 | Postgres HA 前提、retry backoff |

### 8. ZEN Engine（Decision）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| ルール評価バグ | JDM 評価器の実装バグ | 誤判定、業務誤動作 | 9 | 2 | 7 | 126 | ゴールデンケースでの回帰テスト、段階リリース |
| ルール無限ループ | ルール定義の循環参照 | 評価タイムアウト | 7 | 2 | 3 | 42 | 評価深さ上限、タイムアウト設定 |
| ルールバージョン不整合 | 旧バージョン参照 | 一貫性喪失 | 6 | 3 | 5 | 90 | version pinning、evaluation trace |

### 9. Keycloak（認証）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| Keycloak 障害 | Pod crash、DB 障害 | SSO 経路不通、既発行 JWT は有効 | 9 | 2 | 2 | 36 | HA 構成、トークン有効期限延長で緩和 |
| LDAP 連携障害 | 社内 LDAP サーバ停止 | 新規ログイン不可 | 7 | 2 | 3 | 42 | LDAP キャッシュ、代替 IdP 切替手順 |
| トークン署名鍵ローテ失敗 | 鍵管理ミス | 全トークン無効化 | 10 | 1 | 4 | 40 | ローテ手順 Runbook、rollback 準備 |

### 10. OpenBao（Secrets）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| シール状態（起動直後） | OpenBao 再起動時 | 全 Secrets 取得不可 | 10 | 2 | 1 | 20 | auto-unseal、複数 HSM |
| 権限ポリシー不備 | ポリシー設定ミス | 過剰権限または拒否 | 8 | 3 | 5 | 120 | policy as code、Kyverno 検証 |
| Audit ログ喪失 | ディスクフル | 監査不能、コンプライアンス違反 | 9 | 1 | 4 | 36 | 複数宛先ログ出力、容量監視 |

### 11. Kubernetes（Control Plane）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| etcd クォーラム喪失 | ネットワーク分断・複数 Node 障害 | 全 API 不能、新規 Pod 起動不可 | 10 | 1 | 2 | 20 | 3/5 Node 冗長、スナップショット |
| API Server 過負荷 | 過剰な list/watch | 応答遅延、controller 停滞 | 7 | 3 | 3 | 63 | rate limit、priority level |
| kubelet 接続切断 | Node 障害・ネットワーク | Node NotReady、Pod 再配置 | 6 | 3 | 2 | 36 | multi-Node 冗長、自動再配置 |

### 12. Longhorn（ストレージ）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| ボリュームデタッチ失敗 | Node 障害時 | Pod がアタッチできず起動失敗 | 8 | 2 | 3 | 48 | RWX 利用、force detach 手順 |
| レプリカ不整合 | レプリ間で書込順序ずれ | 読取エラー、要レプリカ修復 | 7 | 2 | 4 | 56 | regular snapshot、自動修復 |
| 容量圧迫 | 無計画スナップショット | ボリューム拡張不可 | 7 | 3 | 3 | 63 | スナップショットライフサイクル管理 |

### 13. MetalLB（LoadBalancer）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| VIP フェイルオーバー失敗 | speaker Pod 異常 | 外部到達不可 | 9 | 2 | 3 | 54 | speaker 冗長、BGP peering 健全性監視 |
| ARP 広告不整合 | L2 モード設定ミス | 断続的到達不可 | 7 | 2 | 5 | 70 | ネットワーク設計レビュー、staging 検証 |

### 14. Argo CD（GitOps）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| Sync 失敗 | マニフェスト不整合 | 変更反映不可、ドリフト継続 | 5 | 4 | 2 | 40 | pre-sync hook validation、PR 時 lint |
| Git リポジトリ接続断 | SSH key 期限・GitLab 障害 | Sync 停止、緊急変更不可 | 7 | 2 | 3 | 42 | mirror リポジトリ、複数 credential |

### 15. Kyverno（ポリシー）

| 故障モード | 原因 | 影響 | Sev | Occ | Det | RPN | 対策 |
|---|---|---|---:|---:|---:|---:|---|
| Webhook タイムアウト | Kyverno Pod 遅延・障害 | admission 失敗、新規 Pod 起動不可 | 8 | 2 | 2 | 32 | failurePolicy=Ignore、Kyverno HA |
| ポリシー誤設定 | 過度に厳しい規則 | 正当な操作を拒否 | 6 | 3 | 4 | 72 | staging 先行、audit モードから段階導入 |

## 優先対策リスト（RPN 100 以上）

RPN 100 を超えた高リスク項目を以下に抽出する。Phase 1b までに対策計画を確定し、Phase 1c までに運用に組み込む。

- **Temporal Determinism 違反（RPN 135）**: ワークフローコードの CI 検証（replay test）必須化。PR で Determinism 違反検知ツールを走らせる
- **ZEN Engine ルール評価バグ（RPN 126）**: ゴールデンケース回帰テスト、段階リリース（canary）、評価 trace の常時記録
- **OpenBao 権限ポリシー不備（RPN 120）**: policy as code、Kyverno での事前検証、四半期権限棚卸し
- **ZEN Engine ルールバージョン不整合（RPN 90）**: version pinning 強制、rollback 手順書
- **Istio Ambient ポリシー誤設定（RPN 84）**: Kyverno 事前検証、staging での先行適用
- **Kafka コンシューマラグ拡大（RPN 84）**: ラグ閾値アラート、HPA 連動
- **MinIO バケットポリシー誤り（RPN 84）**: Kyverno バリデーション、定期監査
- **PostgreSQL レプリ遅延（RPN 81）**: `pg_stat_replication` 継続監視、閾値アラート
- **Temporal デッドロック（RPN 80、tier1 goroutine/async）**: goroutine dump 自動取得、lease timeout

## FMEA の運用と更新

FMEA は一度作って終わりではなく、以下のトリガーで更新する。

- **新コンポーネント追加時**: ADR 策定と同時に FMEA を更新
- **インシデント発生後**: ポストモーテム（OR-INC-004）の結果を反映、既存故障モードの Sev/Occ/Det を調整、新しい故障モードを追加
- **四半期レビュー**: Product Council で全 RPN を見直し、RPN 100 以上の項目の対策進捗を確認
- **外部脅威動向**: CVE 公開・業界インシデント（SolarWinds、Log4Shell 等）を参考に Occ を見直し

FMEA 結果は Runbook（NFR-C-NOP-003）の「故障モードと初動」セクションに反映され、オンコールチームの訓練（OR-INC-006）の教材になる。

## 要件 ID

- **OR-FMEA-001**: 主要 15 コンポーネントに対する FMEA を実施、RPN を算出（MUST、Phase 1b）
- **OR-FMEA-002**: RPN 100 以上の故障モードに対して対策計画を確定（MUST、Phase 1b）
- **OR-FMEA-003**: FMEA 結果を Runbook に反映（MUST、Phase 1c）
- **OR-FMEA-004**: 四半期ごとに FMEA を更新、インシデント結果を反映（MUST、継続）
- **OR-FMEA-005**: 新コンポーネント追加時の FMEA 策定を PR テンプレートに含める（SHOULD、Phase 1c）

## メンテナンス

本書の Sev/Occ/Det は主観評価を含む。数値の調整には Product Council のコンセンサスを要する。業界共通の FMEA 評価基準（AIAG-VDA FMEA Handbook 2019）を参考にしつつ、k1s0 固有の業務文脈で調整する。
