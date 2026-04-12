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
| **MVP-1a** | パイロット運用開始 | パイロット業務 1 本が k3s HA 上で稼働し、バス係数 2 を実証する | 2 名 | MVP-0 後 | VM 3 台 |
| **MVP-1b** | 運用品質の確保 | 障害対応・バックアップ・復旧が起案者なしで完結する | 2 名 | MVP-1a 後 | VM 3 台 (同一) |

### なぜ 3 段階か

旧計画では MVP-1 で 20 以上のコンポーネント (Istio / Kafka / Jaeger / Backstage 等) を一括導入していた。これは「動かす」ことはできても「2 名で運用し続ける」ことができない規模である。

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
| k3s (1 ノード) | 単一 VM 上の軽量 k8s | HA 構成は MVP-1a で |
| Dapr Control Plane | operator / sidecar-injector | sidecar mode |
| tier1 Go サービス | `k1s0.Log` のみ実装 | 最小ファサード |
| 雛形生成 CLI | 最小テンプレート 1 パターン | デモ用 |
| Keycloak | 単一インスタンス / ローカル DB | SSO 統一のデモ |
| PostgreSQL | 単一インスタンス / HA なし | Keycloak 用 |
| アプリ配信ポータル | アプリ一覧 + SSO + Web アプリ起動リンク | 最小 UI |
| サンプル Web アプリ | パイロット候補業務の簡易版 | デモ対象 |

### 含めないもの

Istio / Envoy Gateway / Kafka / 可観測性スタック / Harbor / Backstage / Argo CD / GHA runner / OpenTofu / Valkey / HA 構成。これらは MVP-1a 以降で段階的に導入する。

### ハードウェア要件

| 項目 | 要件 |
|---|---|
| ノード数 | 1 |
| vCPU | 4 |
| メモリ | 8 GB |
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

### 設計判断: k3s を継続する

MVP-0 の k3s を kubeadm / kubespray に移行せず、**k3s 3 ノード HA 構成をそのまま採用する**。

理由:

- k3s は CNCF 認定 Kubernetes ディストリビューションであり、本番運用に耐える
- kubeadm への移行は非自明であり、2 名体制で移行リスクを負うべきではない
- k3s のリソースオーバーヘッドが低く (単一バイナリ)、VM 3 台構成でのメモリ余裕が大きい
- k8s API は同一であるため、tier1 / tier2 / tier3 のアプリケーションレイヤに影響しない

k3s からフル k8s への移行は、マルチクラスタやエンタープライズサポートが必要になった Phase 3 以降で検討する。

### 含めるもの

| コンポーネント | スコープ | 備考 |
|---|---|---|
| k3s (3 ノード HA) | embedded etcd、HA 構成 | MVP-0 の k3s から拡張 |
| OpenTofu | VM 作成 + k3s bootstrap の HCL | `tofu apply` で環境再現 |
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
| Testcontainers | tier1 Go サービスの PostgreSQL 統合テス��� | GHA runner Pod (DinD sidecar) + 開発者 PC |
| アプリ配信ポータル | MVP-0 から拡張 (監査ログ記録追加) | tier3 |

### 含めないもの

| 項目 | 除外理由 | 導入時期 |
|---|---|---|
| Istio + Envoy Gateway | サービス数が少なくメッシュの価値が薄い。k3s Traefik + Dapr mTLS で十分 | Phase 2 |
| Kafka (Strimzi) | MVP で非同期イベント駆動を使う tier2/tier3 がない | Phase 2 |
| Jaeger + OTel Collector | Prometheus メトリクス + 構造化ログで MVP の障害調査は賄える | Phase 2 |
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
| Argo Events | Kafka 導入前はイベントソースが不足 | Phase 2 |
| tier1 レート制限 | tier2/tier3 サービスが 1~2 本で暴走リスク低 | Phase 2 |

### メモリ概算

| コンポーネント群 | 概算メモリ |
|---|---|
| k3s システム (3 ノード) | 3 GB |
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
| **合計 (クラスタ側)** | **約 15.3 GB** |

Tilt はクラスタ側ではなく開発者のローカル PC で動作するため、上記メモリ概算には含めない。ローカル環境の必要メモリは約 2.1 GB (詳細は [`../05_CICDと配信/04_ローカル開発環境.md`](../05_CICDと配信/04_ローカル開発環境.md) を参照)。

### ハードウェア要件

| 項目 | 要件 |
|---|---|
| ノード数 | 3 |
| vCPU / ノード | 4 |
| メモリ / ノード | 8 GB (合計 24 GB、余裕 約 8.7 GB) |
| ディスク / ノード | 200 GB SSD |
| ネットワーク | 1 Gbps |

旧計画 (16 vCPU / 32 GB / ノード) と比較して **メモリ要件が 1/4** になり、VM 確保の稟議ハードルが大幅に下がる。

### 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | パイロット業務の Web アプリが配信ポータルから起動可能 | エンドユーザーによる動作確認 |
| 2 | GHA パイプライン (PR → ビルド → GitOps → Argo CD 同期) が疎通 | PR マージ → 自動デプロイの実行 |
| 3 | Keycloak SSO が Argo CD / 配信ポータルで機能 | 複数ツールへのシングルサインオン確認 |
| 4 | `tofu apply` でクラスタが再構築可能 | 協力者が独立して実行 |
| 5 | Prometheus + Grafana でノードと tier1 API の状態を確認可能 | ダッシュボードの表示確認 |
| 6 | **起案者以外の協力者が手順書に従い独立して環境を再構築できる** | 起案者不在で再構築テスト |
| 7 | 協力者がローカル環境で `tilt up` → tier1 サービスが起動し開発可能 | ローカル PC での動作確認 |
| 8 | Longhorn が全 PV をレプリケーションしていることを確認 | `kubectl get volumes.longhorn.io` でレプリカ数を検証 |
| 9 | PgBouncer 経由で全サービスが PostgreSQL に接続可能 | `pgbouncer` の `SHOW POOLS` で接続状況確認 |
| 10 | Renovate が初回実行で依存更新 PR を生成 | GitHub PR 一覧で Renovate ラベルの PR を確認 |
| 11 | Testcontainers ベース統合テストが GHA runner 上で実行され、tier1 Go → PostgreSQL の統合を自動検証 | CI ログで Testcontainers テストの pass を確認 |

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

### 実施するもの (コンポーネント追加以外)

| 項目 | 内容 |
|---|---|
| CI/CD パイプライン完成 | PR → Lint / UT / Build → FS Scan → Harbor push → GitOps → Argo CD 同期 |
| バックアップ手順書 | CloudNativePG (barman-cloud) + etcd スナップショット |
| フルリストア訓練 | 協力者が単独でクラスタ全壊 → 復旧を 1 回実施 |
| 運用 Runbook | 再起動 / スケール / ログ閲覧 / アラート対応の手順書 |
| アラート設定 | Prometheus アラートルール: ノードダウン / Pod 再起動 / ディスク使用率 / tier1 API エラー率 |

### メモリ概算 (MVP-1a からの追加分)

| コンポーネント群 | 概算メモリ |
|---|---|
| MVP-1a 合計 | 15.3 GB |
| Harbor + Trivy | 3 GB |
| Loki | 1 GB |
| Sealed Secrets | 0.2 GB |
| Kyverno (3 replicas) | 0.5 GB |
| MinIO (単一インスタンス) | 1 GB |
| OpenBao (HA 3 Pod Raft) | 0.8 GB |
| **合計** | **約 21.8 GB** |

ハードウェアは MVP-1a と同一 (3 ノード x 8 GB = 24 GB)。余裕 約 2.2 GB。

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

**MVP-1b の全完了条件が満たされた時点で Phase 2 に移行する。**

---

## 6. MVP に含めないもの (Phase 2 以降)

| 項目 | 除外理由 | 導入時期 |
|---|---|---|
| Istio + Envoy Gateway | サービスメッシュはサービス数増加後に価値が出る | Phase 2 |
| Kafka (Strimzi) | 非同期イベント駆動は tier2 開発開始時に必要になる | Phase 2 |
| Jaeger + OTel Collector | 分散トレーシングはサービス間呼び出しが増えてから | Phase 2 |
| Backstage | 開発者ポータルは開発者 3 名以上で効果が出る | Phase 2 |
| Valkey | キャッシュ需要は tier2 のパフォーマンス要件から導入 | Phase 2 |
| tier1 Rust サービス / ZEN Engine 本格実装 | Go のみで MVP の価値証明は十分 | Phase 2 |
| tier1 API full 実装 (State / PubSub / Workflow 等) | スタブで足りる | Phase 2 |
| tier2 サンプルサービス (複数) | リファレンス 1 本で価値を示せる | Phase 2 |
| Cosign (イメージ署名) | Kyverno 署名検証ポリシーと併せて Phase 2 で導入 | Phase 2 |
| OpenFeature + flagd (Feature Flag) | tier2 サービス増加後に段階ロールアウトの価値が出る | Phase 2 |
| Litmus (Chaos Engineering) | 縮退動作の自動検証。tier1 API が揃った Phase 2 で本格導入 | Phase 2 |
| Argo Events | Kafka 導入前はイベントソースが不足 | Phase 2 |
| tier1 レート制限 | tier2/tier3 サービスが 1~2 本で暴走リスク低 | Phase 2 |
| OpenBao Database / Transit Engine | 動的認証情報と PII 暗号化は Phase 2 | Phase 2 |
| ネイティブアプリ配信 (MSIX / ClickOnce) | PWA 優先 | Phase 3 |
| 端末設定コピー | アプリ増加後に現実的 | Phase 3 |
| 申請ワークフロー / 稟議承認 | Dapr Workflow が必要 | Phase 4 |
| レガシー .NET Framework ラップ配信 | 最も複雑 | Phase 4〜5 |
| マルチクラスタ | 1 クラスタで十分 | Phase 3 |
| AD 連携 | ローカル DB で数十名を運用 | Phase 2 |

---

## 7. Phase 別スケール想定

| Phase | ノード数 | ノードスペック | 備考 |
|---|---|---|---|
| MVP-0 | 1 | 4 vCPU / 8 GB / 100 GB SSD | k3s single |
| MVP-1a / 1b | 3 | 4 vCPU / 8 GB / 200 GB SSD | k3s HA |
| Phase 2 | 3〜5 | 8 vCPU / 16 GB / 500 GB SSD | Istio / Kafka 追加のためスケールアップ |
| Phase 3 | 5〜8 | 16 vCPU / 32 GB / 500 GB SSD | — |
| Phase 5 | 10+ | 同上 + ストレージノード追加 | — |

---

## 関連ドキュメント

- [`00_フェーズ計画.md`](./00_フェーズ計画.md) — 全フェーズの俯瞰
- [`02_体制と役割.md`](./02_体制と役割.md) — MVP-0 / MVP-1a / MVP-1b の体制
- [`../05_CICDと配信/04_ローカル開発環境.md`](../05_CICDと配信/04_ローカル開発環境.md) — Tilt によるローカル開発環境
- [`../05_CICDと配信/02_アプリ配信ポータル.md`](../05_CICDと配信/02_アプリ配信ポータル.md) — 配信ポータルの Phase 分離
- [`../06_競合と差別化/03_TCOとBuildVsBuy.md`](../06_競合と差別化/03_TCOとBuildVsBuy.md) — Build リスク低減との関係
