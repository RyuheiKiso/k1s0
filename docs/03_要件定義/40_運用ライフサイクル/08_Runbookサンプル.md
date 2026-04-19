# Runbook サンプル

本書は k1s0 で実際に発生する可能性が高い 5 シナリオについて、**実運用で使える粒度** の Runbook を示す。Runbook は「アラートから該当ページへの導線」「事実確認手順」「復旧手順」「根本原因調査」「事後処理」の 5 段構成を共通テンプレートとし、インシデント対応者が迷いなく動ける構造にする。

## Runbook の思想

[01_インシデント対応詳細.md](01_インシデント対応詳細.md) で規定した対応フローに対して、本書は **具体シナリオの実体** を提供する。理想論（「慎重に確認してから対応する」）ではなく、**具体的なコマンド、Grafana クエリ、判断閾値** を明記することで属人化を防ぐ。各 Runbook は以下の構造を守る。

- **検出**: どのアラートがトリガーとなるか、Grafana の具体的なクエリと閾値
- **初動（15 分以内）**: 影響範囲の確定、ブロードキャスト、一次対応
- **復旧（60 分以内 / 重大度別）**: サービスを戻すための具体手順
- **根本原因調査**: ログ・トレース・メトリクスから因果を追う
- **事後処理**: Audit、ポストモーテム、Runbook 自身の更新

---

## Runbook RB-001: PostgreSQL Primary ダウン

### 適用シナリオ

tier1 State API、Workflow API、Audit API が依存する CloudNativePG Primary が応答しない。FMEA RPN 140 対応。

### 検出

- **アラート**: `PgPrimaryDown` (`up{job="cnpg", role="primary"} == 0` が 30 秒継続)
- **Grafana ダッシュボード**: `cnpg / cluster-overview`
- **典型症状**: tier2 からの Write が 5xx、Read は Replica が残っていれば継続

### 初動（15 分以内）

1. Alertmanager で PagerDuty エスカレーション確認、SRE オンコールに連絡
2. Backstage の Service Catalog で該当 tenant の影響範囲確認（どの tier2 アプリが接続しているか）
3. `kubectl -n cnpg get cluster` でクラスタ状態確認
4. 社内ブロードキャスト: `#incident-sev1` チャンネルに影響サービス名と ETA 暫定（30 分）

### 復旧（60 分以内）

CNPG は自動フェイルオーバーが基本。以下は自動フェイルオーバーが失敗または遅延しているケースの手動介入。

```bash
# 1. 現 primary / replica の LSN 確認
kubectl cnpg status <cluster-name> -n <ns>

# 2. 手動 switchover (最新 LSN を持つ replica へ)
kubectl cnpg promote <cluster-name> <new-primary-pod> -n <ns>

# 3. アプリ側 pool をリセット (tier1 Dapr は自動リトライだが念押し)
kubectl -n <app-ns> rollout restart deployment <tier2-app>

# 4. 旧 primary の状態確認 (ディスク障害なら pod delete + PVC 再作成)
kubectl describe pod <old-primary-pod> -n <ns>
kubectl logs <old-primary-pod> -n <ns> --previous
```

復旧判定: `kubectl cnpg status` で Primary 確定 + tier2 の書込み成功率が 99% 超に回復。

### 根本原因調査

- `kubectl logs <old-primary-pod>` / Loki: `{namespace="cnpg"} |= "fatal"`
- Node レベル調査: Node 上の disk iowait、メモリ枯渇、OOM Killer
- Longhorn Volume 状態: Node 障害や Disk 故障なら Longhorn UI で確認
- CNPG メトリクス: WAL 蓄積、Replica Lag、Checkpoint 頻度

### 事後処理

- Audit API に操作記録（手動 switchover は特権操作）
- ポストモーテム作成（テンプレート: `docs/03_要件定義/40_運用ライフサイクル/postmortem_template.md` 予定）
- 再発防止: ディスク障害なら Longhorn Replica 数増加の検討、メモリ枯渇なら resources.limits 見直し
- 本 Runbook の更新（不正確な手順があれば修正）

### 関連

- ADR-DATA-001（CloudNativePG）
- FMEA RPN 140（PG Primary 障害）

---

## Runbook RB-002: Kafka Broker ダウン（3 台中 1 台）

### 適用シナリオ

Strimzi Kafka の 3 broker のうち 1 台が ISR（In-Sync Replicas）から脱落。FMEA RPN 120 対応。

### 検出

- **アラート**: `KafkaUnderReplicated` (`kafka_server_replicamanager_under_replicated_partitions > 0` が 60 秒継続)
- **Grafana ダッシュボード**: `strimzi / kafka-cluster`
- **典型症状**: RF=3 の RF=2 に低下、Consumer/Producer はまだ動作

### 初動（15 分以内）

1. Alertmanager で通知、SRE オンコールが状況確認
2. `kubectl -n kafka get kafka` でクラスタ状態
3. 残 2 broker が健全なら緊急性 P2。全 broker 障害なら即 P1 扱い
4. 該当 broker の Pod / Node 状態確認

### 復旧（60 分以内）

Strimzi Operator が自動で Pod 再起動を試みる。自動復旧が失敗する典型パターン 2 つ:

**パターン A: PVC アクセス不可（Longhorn 障害）**
```bash
# 1. Pod の events 確認
kubectl describe pod <broker-pod> -n kafka

# 2. Longhorn Volume 状態確認
# Longhorn UI or longhorn-manager で対象 PV が Faulted ならレプリカ再構築
kubectl -n longhorn-system get volumes.longhorn.io

# 3. Replica 再構築待ち (数十分)
# 復旧できない場合は新 PVC 払い出し + 再バランシング
```

**パターン B: JBOD ディスク満杯**
```bash
# 1. ディスク使用率確認
kubectl exec -n kafka <broker-pod> -- df -h /var/lib/kafka

# 2. 一時退避 (古いログセグメント削除 or 保持期間短縮)
kubectl -n kafka edit kafka <cluster-name>
# spec.kafka.config.log.retention.hours を 168 → 72 に一時変更

# 3. Broker 再起動 + ISR 復帰待ち
```

復旧判定: `UnderReplicatedPartitions` が 0、ISR が 3 に戻る。

### 根本原因調査

- `kubectl logs <broker-pod> -n kafka`
- Kafka メトリクス: Disk Usage、Network Latency、Request Handler Pool Idle
- Node メトリクス: iowait、Network Drop
- Longhorn の Volume Health

### 事後処理

- Consumer Lag が復旧前に蓄積していたら、該当 tenant に影響範囲通知
- DLQ 滞留があれば RB-005 へ連携
- キャパシティ再評価（[07_負荷試験とキャパシティ.md](07_負荷試験とキャパシティ.md)）

### 関連

- ADR-DATA-002（Strimzi Kafka）
- FMEA RPN 120（Broker 障害）

---

## Runbook RB-003: mTLS 証明書期限切れ / SPIFFE ID 不一致

### 適用シナリオ

Istio Ambient の ztunnel / Waypoint 証明書失効、または SPIRE が新規 SVID 発行を失敗し、サービス間通信が `UNAUTHENTICATED` で全滅。

### 検出

- **アラート**: `IstioMTLSFailureHigh` (`istio_request_total{response_code="401"} > 100/min`)
- **Grafana ダッシュボード**: `istio / security-overview`
- **典型症状**: tier2 ↔ tier1 通信がすべて 401、tier3 UI 画面が真っ白

### 初動（15 分以内）

1. 影響範囲確認: 特定サービス間だけか、全体か
2. Keycloak / 外部 PKI への接続性確認（SPIRE Upstream Authority）
3. `kubectl -n spire get csr` で保留中 CSR の有無

### 復旧（60 分以内）

**パターン A: SPIRE Server 側の問題**
```bash
# 1. SPIRE Server 状態確認
kubectl -n spire logs -l app=spire-server --tail=200

# 2. Upstream CA への接続確認 (社内 PKI)
kubectl -n spire exec <spire-server-pod> -- openssl s_client -connect <ca-host>:443

# 3. SPIRE Agent 再起動 (DaemonSet)
kubectl -n spire rollout restart daemonset spire-agent
```

**パターン B: Istio ztunnel 側**
```bash
# 1. ztunnel 状態確認
kubectl -n istio-system logs -l app=ztunnel --tail=200

# 2. 証明書ローテート強制
kubectl -n istio-system rollout restart daemonset ztunnel

# 3. Waypoint 証明書確認
kubectl -n <app-ns> get waypoint <name> -o yaml
```

**緊急回避**: どうしても復旧できない場合、特定 namespace のみ一時的に mTLS を PERMISSIVE に落として障害影響を切り分け（本番適用は Audit 証跡必須、CISO 承認）。

### 根本原因調査

- SPIRE Server ログ、Upstream CA との通信履歴
- Istio ztunnel / istiod ログ
- 証明書の有効期限（`step certificate inspect` 等）
- 監査視点: 直前の設定変更（GitOps PR 履歴）

### 事後処理

- 緊急 PERMISSIVE にした場合は復旧後に STRICT に戻す、変更ログ Audit
- SPIRE 証明書のローテート周期見直し（デフォルト 1 時間、短すぎれば負荷、長すぎれば失効リスク）
- ADR-SEC-003 に追記

### 関連

- ADR-0001（Istio Ambient）、ADR-SEC-003（SPIFFE/SPIRE）
- FMEA RPN 130（mTLS 失敗）

---

## Runbook RB-004: Temporal Workflow Determinism 違反

### 適用シナリオ

Worker 再起動後に過去の Workflow を Replay した際、コード変更による Non-Determinism が検知され、実行が Failed となる。FMEA RPN 135 対応。

### 検出

- **アラート**: `TemporalNonDeterministicErrors` (`temporal_workflow_task_non_deterministic_total > 0` が 5 分継続)
- **Grafana ダッシュボード**: `temporal / worker-overview`
- **典型症状**: 承認ワークフローが止まる、Temporal Web UI で該当 run が `Failed`

### 初動（15 分以内）

1. Temporal Web UI で `Failed` ワークフロー一覧、対象 run の History を確認
2. `NonDeterministicError` メッセージから差異箇所を特定（期待したイベント vs 実際のイベント）
3. 直近の Worker デプロイ履歴（Argo CD）を確認

### 復旧（60 分以内）

**最速の復旧手段: 旧バージョンの Worker にロールバック**

```bash
# 1. Argo Rollouts で前バージョンへロールバック
kubectl argo rollouts undo <worker-rollout-name> -n <ns>

# 2. 影響範囲の Workflow が再開されることを Temporal Web UI で確認
```

**根本対応: Versioning 戦略**

1. Worker コードに `workflow.GetVersion` を使い、進行中 workflow は旧ロジック、新規 workflow は新ロジックで分岐
2. 再デプロイ、全 workflow が完了するまで旧ロジックを保持
3. 全 workflow 完了後に旧ロジックを削除

### 根本原因調査

- Replay テストの CI/CD 通過状況確認（通過していたのに本番で発生なら、テストケース不足）
- 具体的なコード差分確認（`time.Now()` / ランダム / 外部 API 直接呼出 等の禁止パターンが混入していないか）
- Worker のバージョンタグ、デプロイ履歴

### 事後処理

- Replay テストケースを追加（再発防止）
- ADR-RULE-002 の「Determinism 違反を避けるコーディングガイド」に事例追加
- Workflow 管理者トレーニング（BC-TRN）に反映

### 関連

- ADR-RULE-002（Temporal）
- FMEA RPN 135（Determinism 違反）

---

## Runbook RB-005: DLQ 滞留

### 適用シナリオ

PubSub の DLQ（Dead Letter Queue）にメッセージが一定数以上滞留し、業務処理が止まる。

### 検出

- **アラート**: `DLQDepthHigh` (`kafka_topic_partition_messages{topic=~".*-dlq"} > 100`)
- **Grafana ダッシュボード**: `k1s0 / pubsub-dlq`
- **典型症状**: 特定 tenant / topic の処理遅延、DLQ 監視ダッシュボードで滞留可視化

### 初動（15 分以内）

1. DLQ 対象 topic と tenant 特定
2. DLQ 滞留メッセージのサンプル取得（Backstage の Kafka Explorer プラグイン or kafkactl）
3. 共通エラー原因の仮説（スキーマ変更、依存サービス障害、データ不正）

### 復旧（60 分以内）

**パターン A: 一時的な依存サービス障害**
- 依存サービス復旧確認後、DLQ から再投入

```bash
# DLQ → 元 topic に再投入 (kafkactl 等のツール)
kafkactl consume <topic>-dlq --from-beginning --print-headers | \
  kafkactl produce <topic>
```

**パターン B: スキーマ変更 / コード Bug**
- バグ修正を優先、Worker 再デプロイ後に DLQ 再投入
- 再投入前に内容サンプリング検査必須（機微情報漏洩 / 意図しない副作用回避）

**パターン C: データ不正（復旧不能）**
- Audit 証跡を取り、該当メッセージを MinIO に永久退避
- 業務担当に影響ユーザ通知

### 根本原因調査

- Subscriber 側エラーログ（Loki）
- 原メッセージの payload 構造比較
- 依存サービスのメトリクス / トレース

### 事後処理

- DLQ 再投入の操作ログを Audit API に記録
- 再発防止: 契約テスト（Pact）の強化、スキーマ変更時の Feature Flag 段階リリース
- tenant への影響範囲通知、ポストモーテム

### 関連

- ADR-DATA-002（Strimzi Kafka）
- [20_機能要件/50_tier1_APIシーケンス.md](../20_機能要件/50_tier1_APIシーケンス.md) パターン ②

---

## Runbook 運用ルール

- **鮮度管理**: 各 Runbook は半期に 1 回の訓練で実際に使い、手順の妥当性を検証する
- **Git 管理**: 全 Runbook を本リポジトリで管理、PR レビュー必須
- **アラートとの導線**: 各 Runbook の冒頭 URL を Prometheus アラートの `runbook_url` アノテーションに記載、Alertmanager 通知で即参照可能
- **ポストモーテムからの反映**: インシデント後に必ず Runbook を更新するサイクルを確立

## 関連ドキュメント

- [01_インシデント対応詳細.md](01_インシデント対応詳細.md): 対応フレームワーク
- [06_FMEA分析.md](06_FMEA分析.md): 故障モード一覧
- [02_サポート階層.md](02_サポート階層.md): エスカレーション体系
