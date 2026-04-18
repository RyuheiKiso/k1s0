# MVP スコープ

## 目的

Phase 1 (MVP) で **何を含め、何を含めないか** を明示する。MVP を 3 段階 (MVP-0 / MVP-1a / MVP-1b) に分割し、各段階で 1 つの明確な価値を証明する構成とする。

---

## 1. MVP の原則

1. **最小から積み上げる** — 完成形から逆算して削るのではなく、必要になった時点でコンポーネントを追加する
2. **各段階に 1 つのゴール** — 目的が散漫な段階を作らない
3. **2 名で運用し続けられる範囲に制限する** — 「動かせる」と「運用できる」は異なる
4. **完了条件は定量化する** — 曖昧な「できた」を認めない
5. **バス係数 2 の実証が MVP 最重要ゴール** — ドキュメント化だけではバス係数の解消にならない

---

## 2. 3 段階構成: MVP-0 / MVP-1a / MVP-1b

| 段階 | 目的 | ゴール (1 文) | 体制 | 期間目安 | ハードウェア |
|---|---|---|---|---|---|
| **MVP-0** | デモ | 決裁者に「動くもの」を見せ、協力者を獲得する | 1 名 | 3〜4 週間 | VM 1 台 |
| **MVP-1a** | パイロット運用開始 | パイロット業務 1 本が kubeadm HA 上で稼働し、バス係数 2 を実証する | 2 名 | MVP-0 後 | VM 3 台 |
| **MVP-1b** | 運用品質の確保 | 障害対応・バックアップ・復旧が起案者なしで完結する | 2 名 | MVP-1a 後 | VM 3 台 (同一) |

### なぜ 3 段階か

旧計画では MVP-1 で 20 以上のコンポーネント (Istio / Kafka / Grafana Tempo / Backstage 等) を一括導入していた。これは「動かす」ことはできても「2 名で運用し続ける」ことができない規模である。

3 段階に分割することで:

- **MVP-0**: 決裁者への説得材料を最速で作る
- **MVP-1a**: パイロット業務に必要な最小構成のみで開始する (コンポーネント数 15)
- **MVP-1b**: 運用上の不安要素 (障害対応・バックアップ) を解消してから Phase 2 に進む

各段階のコンポーネント数を 2 名で管理可能な範囲に制限する。

---

## 3. MVP-0: デモ構成

### 含めるもの

| コンポーネント | スコープ | 備考 |
|---|---|---|
| kubeadm (1 ノード) | 単一 VM 上の single control-plane k8s。kubeadm の運用負荷が過大な場合は k3s にフォールバック | HA 構成は MVP-1a で |
| Kustomize | 自製マニフェスト (tier1 / 配信ポータル) の base/overlay 管理 | kubectl 組み込み、追加インストール不要 |
| Helm | サードパーティ OSS (Keycloak / PostgreSQL 等) の Helm Chart インストール | CNCF Graduated |
| Dapr Control Plane | operator / sidecar-injector | sidecar mode |
| tier1 Go サービス | `k1s0.Log` のみ実装 | 最小ファサード |
| 雛形生成 CLI | 最小テンプレート 1 パターン | デモ用 |
| Keycloak | 単一インスタンス / ローカル DB | SSO 統一のデモ |
| PostgreSQL | 単一インスタンス / HA なし | Keycloak 用 |
| アプリ配信ポータル | アプリ一覧 + SSO + Web アプリ起動リンク | 最小 UI |
| サンプル Web アプリ | パイロット候補業務の簡易版 | デモ対象 |

### 含めないもの

Istio / Envoy Gateway / Kafka / 可観測性スタック / Harbor / Backstage / Argo CD / GHA runner / OpenTofu / Valkey / HA 構成。これらは MVP-1a 以降で段階的に導入する。k3s (サブプラン) への切り替え判断も MVP-0 構築時に実施する。

### ハードウェア要件

| 項目 | 要件 |
|---|---|
| ノード数 | 1 |
| vCPU | 4 |
| メモリ | 16 GB |
| ディスク | 100 GB SSD |

既存の開発用 VM や起案者の手元マシンでも動作する。**新規の稟議・調達が不要な範囲**で開始できる。

### 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | SSO ログイン → 配信ポータル → サンプルアプリ起動のデモ実演 | 決裁者の前で実演 |
| 2 | オンプレ VM 1 台で完結、クラウド依存ゼロを実証 | ネットワーク切断状態で動作確認 |
| 3 | 決裁者から MVP-1a 用 VM 3 台の確保と協力者 1 名のアサイン同意 | 合意の記録 |

**期間目安: 3〜4 週間**。Keycloak OIDC 設定 (1〜2 日)、Dapr Component 設定 (1〜2 日)、配信ポータル UI (3〜5 日)、サンプルアプリ (3〜5 日)、統合テスト・デモ準備 (3〜5 日) を見込む。

---

## 4. MVP-1a: パイロット運用開始

### 設計判断: kubeadm をメインプランとする

MVP-0 から **kubeadm (kubespray による自動化) を採用する**。k3s はサブプラン (フォールバック) として位置付ける。

メインプラン (kubeadm) の理由:

- ローカル開発環境 (Docker Desktop) と本番クラスタの k8s ディストリビューションを統一できる
- Phase 2 以降でのディストリビューション移行というリスクイベントが発生しない
- kubeadm の運用知識が蓄積され、将来のスケールに備えられる
- kubespray で構築を自動化すれば、2 名体制でも管理可能

サブプラン (k3s) への切り替え条件:

- MVP-0 の構築時に kubeadm のセットアップ工数が想定を大幅に超過した場合
- 2 名体制での運用負荷が許容範囲を超えると判断された場合

k3s は CNCF 認定 Kubernetes ディストリビューションであり、k8s API は完全互換のため、切り替え時にアプリケーションレイヤへの影響はない。

### 含めるもの

| コンポーネント | スコープ | 備考 |
|---|---|---|
| kubeadm (3 ノード HA) | stacked etcd、HA 構成。kubespray で自動化。サブプランとして k3s (embedded etcd) も選択可 | MVP-0 から拡張 |
| OpenTofu | VM 作成 + kubeadm bootstrap の HCL (kubespray 連携) | `tofu apply` で環境再現 |
| Kustomize + Helm | MVP-0 から移行。Argo CD が Kustomize overlay を直接参照。Helm で CloudNativePG / cert-manager / MetalLB / Longhorn 等をインストール | マニフェスト管理の主軸 |
| Dapr Control Plane | MVP-0 から移行 | sidecar mode |
| tier1 Go サービス | `k1s0.Log` + `k1s0.Telemetry` 実装、他はスタブ | Go のみ、Rust なし |
| 雛形生成 CLI | MVP-0 から拡張 | — |
| リファレンス実装 | 模範コード 1 本 | — |
| Tilt | ローカル開発環境。Tiltfile + tier1 ローカル構成を雛形生成 CLI で生成 | `tilt up` で開発環境起動 |
| Keycloak HA | PostgreSQL バックエンド、2 レプリカ | MVP-0 から移行 |
| CloudNativePG | プライマリ 1 + レプリカ 1 | 共有 DB |
| Argo CD | tier1 向け ApplicationSet | GitOps |
| GHA self-hosted runner | actions-runner-controller、2 Pod | CI |
| Prometheus + Grafana | ノードヘルス + tier1 API ヘルスのダッシュボード | 最小限の可観測性 |
| cert-manager | SelfSigned → CA Issuer チェーンで内部 CA 構築 | TLS 証明書の自動発行・更新 |
| Longhorn | 分散ストレージ、レプリカ数 2 | CloudNativePG / MinIO / Harbor の PV を保護 |
| CloudNativePG Pooler | PgBouncer 接続プーリング | PostgreSQL 接続枯渇防止 |
| Renovate | 依存パッケージ自動更新 PR 生成、週次実行 | CVE Critical 48h 対応の自動化 |
| Testcontainers | tier1 Go サービスの PostgreSQL 統合テスト | GHA runner Pod (DinD sidecar) + 開発者 PC |
| MetalLB | L2 モード。Service type=LoadBalancer に VIP を払い出し | オンプレミス LB |
| kube-vip | Control Plane VIP。kubespray `kube_vip_enabled: true` で自動構成 | API Server HA |
| OTel Collector | Agent モード (DaemonSet)。OTLP 受信 → Prometheus 転送 | テレメトリパイプライン |
| kube-state-metrics | k8s オブジェクト状態メトリクスの公開 (Deployment / Pod / Node / PVC 等) | Prometheus の k8s 監視を補完 |
| Buf | Protobuf スキーマの lint / 後方互換検証 / コード生成 CLI | tier1 契約管理 |
| Kubeshark | API トラフィックビューア (eBPF)。オンデマンドでデプロイし調査後撤去。起案者・協力者の PC にのみ CLI インストール | デバッグ・障害調査用 |
| Headlamp | Kubernetes Web UI。Keycloak OIDC 連携。運用者向け読み取り専用ビュー | JTC 運用者に kubectl 不要の GUI 提供 |
| アプリ配信ポータル | MVP-0 から拡張 (監査ログ記録追加) | tier3 |

### 含めないもの

| 項目 | 除外理由 | 導入時期 |
|---|---|---|
| Istio + Envoy Gateway | サービス数が少なくメッシュの価値が薄い。Dapr mTLS で十分 | Phase 2 |
| Kafka (Strimzi) | MVP で非同期イベント駆動を使う tier2/tier3 がない | Phase 2 |
| Grafana Tempo | 分散トレーシングはサービス間呼び出しが増えてから。OTel Collector は MVP-1a で先行導入済み | Phase 2 |
| Grafana Pyroscope | Continuous Profiling は Tempo との Traces-to-Profiles 連携が前提。Tempo と同時に導入 | Phase 2 |
| Backstage | 開発者が 2 名の段階で開発者ポータルは不要 | Phase 2 |
| Harbor + Trivy | MVP-1b で導入 | MVP-1b |
| Loki | MVP-1b で導入 | MVP-1b |
| Kyverno | MVP-1b で導入 (PSS + イメージ制御 + ラベル強制) | MVP-1b |
| Apicurio Registry | Kafka 導入前はイベントスキーマ管理が不要 | Phase 2 |
| Valkey | MVP でキャッシュが必要なユースケースがない | Phase 2 |
| tier1 Rust サービス / ZEN Engine | Go のみで MVP の価値証明は可能 | Phase 2 |
| Cosign (イメージ署名) | Kyverno 署名検証ポリシーと併せて Phase 2 で導入 | Phase 2 |
| OpenFeature + flagd (Feature Flag) | tier2 サービス増加後に段階ロールアウトの価値が出る | Phase 2 |
| Litmus (Chaos Engineering) | 縮退動作の自動検証。tier1 API が揃った Phase 2 で本格導入 | Phase 2 |
| Argo Rollouts | Istio 未導入のためカナリア / ブルーグリーンの価値が出ない | Phase 2 |
| Argo Events | Kafka 導入前はイベントソースが不足 | Phase 2 |
| KEDA (イベント駆動オートスケーラー) | Kafka / Temporal 未導入のためイベント駆動スケーリングの価値が出ない | Phase 2 |
| tier1 レート制限 | tier2/tier3 サービスが 1~2 本で暴走リスク低 | Phase 2 |

### メモリ概算

| コンポーネント群 | 概算メモリ |
|---|---|
| kubeadm システム (3 ノード) | 5 GB |
| Dapr Control Plane | 1 GB |
| Keycloak HA (2 レプリカ) + CloudNativePG | 4 GB |
| Argo CD | 1.5 GB |
| GHA runner (2 Pod) + DinD sidecar | 1.5 GB |
| Prometheus + Grafana | 2 GB |
| cert-manager | 0.2 GB |
| tier1 Go + 配信ポータル | 0.5 GB |
| Longhorn (3 ノード DaemonSet) | 1.5 GB |
| PgBouncer (Pooler) | 0.1 GB |
| Renovate (GHA runner 内で実行) | 0 GB (runner Pod に含まれる) |
| MetalLB (speaker DaemonSet + controller) | 0.4 GB |
| kube-vip (static Pod × 3 Control Plane) | 0.2 GB |
| OTel Collector (Agent DaemonSet) | 0.5 GB |
| kube-state-metrics (1 レプリカ) | 0.1 GB |
| Headlamp (1 レプリカ) | 0.1 GB |
| Buf (GHA runner 内で実行) | 0 GB (runner Pod に含まれる) |
| **合計 (クラスタ側)** | **約 18.6 GB** |

Tilt はクラスタ側ではなく開発者のローカル PC で動作するため、上記メモリ概算には含めない。ローカル環境の必要メモリは約 2.1 GB (詳細は [`../../02_構想設計/04_CICDと配信/04_ローカル開発環境.md`](../../02_構想設計/04_CICDと配信/04_ローカル開発環境.md) を参照)。

### ハードウェア要件

| 項目 | 要件 |
|---|---|
| ノード数 | 3 |
| vCPU / ノード | 4 |
| メモリ / ノード | 16 GB (合計 48 GB、余裕 約 29.4 GB) |
| ディスク / ノード | 200 GB SSD |
| ネットワーク | 1 Gbps |

旧計画 (16 vCPU / 32 GB / ノード) と比較して **メモリ要件が 1/2** になり、VM 確保の稟議ハードルが大幅に下がる。サブプラン (k3s) に切り替えた場合はさらに軽量化される。

### 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | パイロット業務の Web アプリが配信ポータルから起動可能 | エンドユーザーによる動作確認 |
| 2 | GHA パイプライン (PR → ビルド → GitOps → Argo CD 同期) が疎通 | PR マージ → 自動デプロイの実行 |
| 3 | Keycloak SSO が Argo CD / 配信ポータルで機能 | 複数ツールへのシングルサインオン確認 |
| 4 | `tofu apply` + kubespray でクラスタが再構築可能 | 協力者が独立して実行 |
| 5 | Prometheus + Grafana でノードと tier1 API の状態を確認可能 | ダッシュボードの表示確認 |
| 6 | **起案者以外の協力者が手順書に従い独立して環境を再構築できる** | 起案者不在で再構築テスト |
| 7 | 協力者がローカル環境で `tilt up` → tier1 サービスが起動し開発可能 | ローカル PC での動作確認 |
| 8 | Longhorn が全 PV をレプリケーションしていることを確認 | `kubectl get volumes.longhorn.io` でレプリカ数を検証 |
| 9 | PgBouncer 経由で全サービスが PostgreSQL に接続可能 | `pgbouncer` の `SHOW POOLS` で接続状況確認 |
| 10 | Renovate が初回実行で依存更新 PR を生成 | GitHub PR 一覧で Renovate ラベルの PR を確認 |
| 11 | Testcontainers ベース統合テストが GHA runner 上で実行され、tier1 Go → PostgreSQL の統合を自動検証 | CI ログで Testcontainers テストの pass を確認 |
| 12 | MetalLB が Envoy Gateway の Service に VIP を払い出し、クラスタ外部からアクセス可能 | `kubectl get svc` で EXTERNAL-IP が付与されていることを確認 |
| 13 | kube-vip が Control Plane VIP を提供し、任意のノード停止時に VIP がフェイルオーバー | 1 ノード停止 → `kubectl get nodes` が VIP 経由で応答することを確認 |
| 14 | OTel Collector Agent が全ノードで稼働し、tier1 サービスのテレメトリを Prometheus に転送 | Grafana で OTel Collector 経由のメトリクスが表示されることを確認 |
| 15 | kube-state-metrics が k8s オブジェクト状態メトリクスを Prometheus に公開 | Grafana で Deployment replicas / Pod phase / Node condition が表示されることを確認 |
| 16 | `buf lint` + `buf breaking` が CI パイプラインで tier1 Protobuf 契約を検証 | 意図的に後方互換性を破る `.proto` 変更で CI が失敗することを確認 |

項目 6 が MVP 全体の最重要完了条件 (バス係数 2 の実証)。項目 7 は協力者の開発効率を保証する。

---

## 5. MVP-1b: 運用品質の確保

MVP-1a で「動く」状態を作った後、**運用に耐える状態** に引き上げる段階。新機能の追加はせず、運用基盤の補強に集中する。

### 追加するもの

| コンポーネント | スコープ | 備考 |
|---|---|---|
| Harbor + Trivy | push 時自動スキャン (Critical 拒否) | サプライチェーンセキュリティ |
| Loki | ログ集約 | 構造化ログの一元検索 |
| Sealed Secrets | Secret の暗号化 Git 管理 | OpenBao の bootstrap 設定等、Git 管理が必要な Secret に限定 |
| OpenBao | シークレット管理 (HA 3 Pod Raft)。KV Engine で静的 Secret を格納 | Dapr Secret Store バックエンドは Phase 2 で移行 |
| Kyverno | PSS Restricted 相当 + イメージソース制限 + `:latest` 禁止 + 必須ラベル・リソース制限の強制 | Admission レベルの最終防衛線 |
| MinIO | S3 互換オブジェクトストレージ (単一インスタンス) | Harbor バックエンド / CloudNativePG バックアップ先 / OpenTofu State 保存先 |
| Velero | k8s リソースバックアップ (CRD / RBAC / ConfigMap 等) | MinIO を保存先に使用。Longhorn (PV) と補完関係 |
| External Secrets Operator | OpenBao → k8s Secret の自動同期 (ExternalSecret CRD)。refreshInterval: 5m | OpenBao を Single Source of Truth にし、インフラ層の k8s Secret を自動管理 |
| Syft + Grype | SBOM 生成 + 脆弱性スキャン (CI ステップ) | Harbor に SBOM を保管。Trivy のイメージスキャンと補完 |

### 実施するもの (コンポーネント追加以外)

| 項目 | 内容 |
|---|---|
| CI/CD パイプライン完成 | PR → Lint / UT / Build → FS Scan → SBOM 生成 (Syft) → Harbor push → GitOps → Argo CD 同期 |
| バックアップ手順書 | CloudNativePG (barman-cloud) + etcd スナップショット + Velero (k8s リソース) |
| フルリストア訓練 | 協力者が単独でクラスタ全壊 → 復旧を 1 回実施 |
| 運用 Runbook | 再起動 / スケール / ログ閲覧 / アラート対応の手順書 |
| アラート設定 | Prometheus アラートルール: ノードダウン / Pod 再起動 / ディスク使用率 / tier1 API エラー率 |

### メモリ概算 (MVP-1a からの追加分)

| コンポーネント群 | 概算メモリ |
|---|---|
| MVP-1a 合計 | 18.6 GB |
| Harbor + Trivy | 3 GB |
| Loki | 1 GB |
| Sealed Secrets | 0.2 GB |
| Kyverno (3 replicas) | 0.5 GB |
| MinIO (単一インスタンス) | 1 GB |
| OpenBao (HA 3 Pod Raft) | 0.8 GB |
| Velero | 0.5 GB |
| External Secrets Operator | 0.25 GB |
| Syft + Grype (GHA runner 内で実行) | 0 GB (runner Pod に含まれる) |
| **合計** | **約 25.75 GB** |

ハードウェアは MVP-1a と同一 (3 ノード x 16 GB = 48 GB)。余裕 約 22.25 GB。

### 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | Harbor push 時に Critical 脆弱性を自動拒否 | 意図的に脆弱なイメージで拒否確認 |
| 2 | Loki + Grafana でアプリケーションログを横断検索可能 | correlation ID でのログ追跡テスト |
| 3 | バックアップ → フルリストアを協力者が単独で完遂 | 起案者不在で復旧テスト |
| 4 | アラート発火 → Runbook 参照 → 対処が起案者なしで完結 | 障害シミュレーション (Pod kill) |
| 5 | 全 Secret が Sealed Secrets / OpenBao で管理されている。リポジトリ上に平文 Secret がない | リポジトリスキャン + OpenBao KV Engine の Secret 一覧確認 |
| 6 | Kyverno が `:latest` タグ / 未承認レジストリのイメージを含む Pod を拒否 | 意図的に違反 Pod を apply し拒否確認 |
| 7 | cert-manager が管理する全証明書の有効期限が正常 | `kubectl get certificates` で Ready 状態を確認 |
| 8 | MinIO に Harbor イメージ / PostgreSQL バックアップが正常に保存されている | MinIO Client (`mc`) で保存内容を確認 |
| 9 | OpenTofu State が MinIO S3 バックエンドに保存され、協力者から参照可能 | 協力者が `tofu plan` を実行し State を取得できることを確認 |
| 10 | Velero が k8s リソース (CRD / RBAC / ConfigMap 等) のバックアップを MinIO に保存 | `velero backup get` でバックアップ成功を確認 |
| 11 | CI パイプラインで SBOM が生成され Harbor に保管されている | Harbor UI で SBOM アーティファクトの存在を確認 |
| 12 | External Secrets Operator が OpenBao KV から k8s Secret を自動同期している | ExternalSecret CRD の status が SecretSynced であることを確認 |

**MVP-1b の全完了条件が満たされた時点で Phase 2 に移行する。**

---

## 6. MVP に含めないもの (Phase 2 以降)

| 項目 | 除外理由 | 導入時期 |
|---|---|---|
| Istio + Envoy Gateway | サービスメッシュはサービス数増加後に価値が出る | Phase 2 |
| Kafka (Strimzi) | 非同期イベント駆動は tier2 開発開始時に必要になる | Phase 2 |
| Grafana Tempo | 分散トレーシングはサービス間呼び出しが増えてから。OTel Collector は MVP-1a で先行導入済み。バックエンドは MinIO | Phase 2 |
| Grafana Pyroscope | Continuous Profiling は Tempo との Traces-to-Profiles 連携が前提。Tempo と同時に Phase 2 で導入。バックエンドは MinIO | Phase 2 |
| OTel Collector Gateway モード | tail-based sampling / PII マスキング等の高度処理。MVP-1a の Agent モードで十分 | Phase 2 |
| Backstage | 開発者ポータルは開発者 3 名以上で効果が出る | Phase 2 |
| Valkey | キャッシュ需要は tier2 のパフォーマンス要件から導入 | Phase 2 |
| tier1 Rust サービス / ZEN Engine 本格実装 | Go のみで MVP の価値証明は十分 | Phase 2 |
| tier1 API full 実装 (State / PubSub / Workflow 等) | スタブで足りる | Phase 2 |
| tier2 サンプルサービス (複数) | リファレンス 1 本で価値を示せる | Phase 2 |
| Cosign (イメージ署名) | Kyverno 署名検証ポリシーと併せて Phase 2 で導入 | Phase 2 |
| OpenFeature + flagd (Feature Flag) | tier2 サービス増加後に段階ロールアウトの価値が出る | Phase 2 |
| Litmus (Chaos Engineering) | 縮退動作の自動検証。tier1 API が揃った Phase 2 で本格導入 | Phase 2 |
| Argo Rollouts | カナリア / ブルーグリーンデプロイ。Istio トラフィック分割と連携 | Phase 2 |
| Argo Events | Kafka 導入前はイベントソースが不足 | Phase 2 |
| KEDA (イベント駆動オートスケーラー) | Kafka / Temporal 未導入のためイベント駆動スケーリングの価値が出ない | Phase 2 |
| tier1 レート制限 | tier2/tier3 サービスが 1~2 本で暴走リスク低 | Phase 2 |
| OpenBao Database / Transit Engine | 動的認証情報と PII 暗号化は Phase 2 | Phase 2 |
| ネイティブアプリ配信 (MSIX / ClickOnce) | PWA 優先 | Phase 3 |
| 端末設定コピー | アプリ増加後に現実的 | Phase 3 |
| Temporal (ワークフローエンジン) | `k1s0.Workflow` の Saga / 長期実行バックエンド。tier1 API の本格実装と同時に導入 | Phase 2 |
| 申請ワークフロー / 稟議承認 | Temporal + tier1 Workflow API が必要 | Phase 4 |
| レガシー .NET Framework ラップ配信 | 最も複雑 | Phase 4〜5 |
| マルチクラスタ | 1 クラスタで十分 | Phase 3 |
| AD 連携 | ローカル DB で数十名を運用 | Phase 2 |

> **サブプラン (k3s)**: 上記スコープは kubeadm をメインプランとして策定している。MVP-0 構築時に kubeadm の運用負荷が 2 名体制で許容できないと判断した場合は、k3s に切り替える。k3s は CNCF 認定の軽量 k8s ディストリビューション (単一バイナリ、embedded etcd) であり、k8s API 完全互換のためアプリケーションレイヤへの影響はない。k3s に切り替えた場合、kubeadm システムのメモリ消費 (5 GB) が約 3 GB に減少し、クラスタ全体のメモリ余裕が増える。

---

## 7. Phase 別スケール想定

| Phase | ノード数 | ノードスペック | 備考 |
|---|---|---|---|
| MVP-0 | 1 | 4 vCPU / 16 GB / 100 GB SSD | kubeadm single (サブ: k3s) |
| MVP-1a / 1b | 3 | 4 vCPU / 16 GB / 200 GB SSD | kubeadm HA (サブ: k3s HA) |
| Phase 2 | 3〜5 | 8 vCPU / 16 GB / 500 GB SSD | Istio / Kafka 追加のためスケールアップ |
| Phase 3 | 5〜8 | 16 vCPU / 32 GB / 500 GB SSD | — |
| Phase 5 | 10+ | 同上 + ストレージノード追加 | — |

---

## 関連ドキュメント

- [`00_フェーズ計画.md`](./00_フェーズ計画.md) — 全フェーズの俯瞰
- [`02_体制と役割.md`](./02_体制と役割.md) — MVP-0 / MVP-1a / MVP-1b の体制
- [`../../02_構想設計/04_CICDと配信/04_ローカル開発環境.md`](../../02_構想設計/04_CICDと配信/04_ローカル開発環境.md) — Tilt によるローカル開発環境
- [`../../02_構想設計/04_CICDと配信/02_アプリ配信ポータル.md`](../../02_構想設計/04_CICDと配信/02_アプリ配信ポータル.md) — 配信ポータルの Phase 分離
- [`../02_競合と差別化/03_TCOとBuildVsBuy.md`](../02_競合と差別化/03_TCOとBuildVsBuy.md) — Build リスク低減との関係
