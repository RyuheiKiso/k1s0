# MVP スコープ

## 目的

Phase 1 (MVP) で **何を含め、何を含めないか** を明示する。バス係数 1 のリスクを低減するため、MVP を 2 段階 (MVP-0 / MVP-1) に分割し、早期に協力者を獲得する構成とする。

---

## 1. MVP の原則

1. **MVP-0 で「動くもの」を決裁者に見せる** — 2 週間以内にデモ可能な最小構成を先行構築する
2. **MVP-1 開始前に協力者を確保する** — MVP-0 のデモで「動く証拠」を作り、机上の企画書では得られない説得力で協力者を獲得する
3. **MVP-1 は 2 名体制で進める** — 知識が最初から 2 人に分散し、バス係数を構造的に 2 以上にする
4. **1 つの小規模業務で実証** — パイロット業務を 1 件選び、そこで価値を示す
5. **「運用可能である」を優先** — 派手な機能より、JTC 情シスが運用できる状態を示す

---

## 2. 2 段階構成: MVP-0 と MVP-1

| 段階 | 目的 | 体制 | 期間目安 | ハードウェア |
|---|---|---|---|---|
| **MVP-0** (デモ構成) | 決裁者に「動くもの」を見せて協力者を獲得 | 起案者 1 名 | 2 週間 | VM 1 台 |
| **MVP-1** (パイロット運用構成) | 1 業務のパイロット運用を開始 | 2 名 | MVP-0 完了後 | VM 3 台 |

### なぜ分割するか

現行の MVP スコープ (20 以上のコンポーネント) を 1 人で構築することは技術的に可能だが、**1 人で運用し続けることは組織的に許容できない**。決裁者が最も警戒するのは「この人がいなくなったらどうなるか」であり、ドキュメント化だけではバス係数の解消にならない。

MVP-0 で動くデモを見せることで **「協力者を得る根拠」を先に作る**。机上の企画書だけで協力者を募るより説得力がある。MVP-1 は 2 名で進めるため、知識が最初から 2 人に分散する。

---

## 3. MVP-0 に含めるもの (デモ構成)

| コンポーネント | スコープ | 備考 |
|---|---|---|
| k3s (1 ノード) | 単一 VM 上の軽量 k8s | 3 ノード構成は MVP-1 で |
| Dapr Control Plane | operator / sidecar-injector | sidecar mode |
| tier1 Go サービス | `k1s0.Log` のみ実装 | 最小ファサード |
| 雛形生成 CLI | 最小テンプレート 1 パターン | デモ用 |
| Keycloak | 単一インスタンス / ローカル DB | SSO 統一のデモ |
| PostgreSQL | 単一インスタンス / HA なし | Keycloak 用 |
| アプリ配信ポータル | アプリ一覧 + SSO + Web アプリ起動リンク | 最小 UI |
| サンプル Web アプリ | パイロット候補業務の簡易版 | デモ対象 |

### MVP-0 に含めないもの

Istio / Envoy Gateway (k3s 内蔵 Traefik で代用)、Kafka、可観測性スタック (Jaeger / Prometheus / Loki / Grafana)、Harbor (k3s 内蔵レジストリで代用)、Backstage、Argo CD / GHA runner、OpenTofu、Valkey、HA 構成。これらは全て MVP-1 で導入する。

### MVP-0 ハードウェア要件

| 項目 | 要件 |
|---|---|
| ノード数 | 1 |
| vCPU | 4 |
| メモリ | 8 GB |
| ディスク | 100 GB SSD |

既存の開発用 VM や起案者の手元マシンでも動作する。**新規の稟議・調達が不要な範囲** で開始できる。

### MVP-0 のゴールと完了条件

| 軸 | ゴール |
|---|---|
| デモ | 決裁者の前で「SSO ログイン → 配信ポータル → サンプルアプリ起動」を実演 |
| 差別化 | オンプレ VM 1 台で完結し、クラウド依存がゼロであることを実証 |
| 説得 | デモを見た決裁者が MVP-1 への VM 確保と協力者 1 名の確保に同意 |

**完了判定**: 決裁者にデモを実施し、MVP-1 用 VM 3 台の確保と協力者 1 名のアサインが得られた時点。

---

## 4. MVP-1 に含めるもの (パイロット運用構成)

### infra 層

| コンポーネント | スコープ | 備考 |
|---|---|---|
| OpenTofu | VM 作成 + k8s bootstrap の HCL | `tofu apply` で環境再現 |
| Kubernetes | 3 ノード構成 | MVP-0 の k3s から移行 |
| Istio + Envoy Gateway | mTLS + API Gateway | — |
| Apache Kafka (Strimzi) | KRaft 3 broker | 最小クラスタ |
| OTel + Jaeger + Prometheus + Loki + Grafana | 可観測性フルスタック | — |
| Valkey | キャッシュ / KV | — |
| CloudNativePG + PostgreSQL | プライマリ 1 + レプリカ 1 | 共有 DB |

### tier1 層

| コンポーネント | スコープ |
|---|---|
| Dapr Control Plane | MVP-0 から移行 |
| tier1 公開 API | `k1s0.Log` / `k1s0.Telemetry` 実装。他はスタブ |
| tier1 内部 Go サービス | Daprファサード 1 サービス |
| 雛形生成 CLI | MVP-0 から拡張 |
| リファレンス実装 | 模範コード 1 本 |

### operation 層

| コンポーネント | スコープ |
|---|---|
| Keycloak | HA 構成へ移行 |
| Backstage | Software Catalog + TechDocs + SSO |
| Argo CD | tier1 向け ApplicationSet |
| Harbor + Trivy | push 時自動スキャン (Critical 拒否) |
| GHA self-hosted runner | actions-runner-controller, 2 Pod |

### tier3 層

アプリ配信ポータルを MVP-0 から拡張 (監査ログ記録を追加)。

### パイプライン

GHA 1 本のワークフロー: Lint / 型 / UT / Build / FS Scan / Push / GitOps 更新。

---

## 5. MVP に含めないもの (明示的な除外)

| 項目 | 除外理由 | 着手時期 |
|---|---|---|
| tier1 API full 実装 (State / PubSub / Workflow 等) | API 契約のみで十分 | Phase 2 |
| tier1 Rust サービス / ZEN Engine 本格実装 | スタブで足りる | Phase 2 |
| tier2 サンプルサービス (複数) | リファレンス 1 本で価値を示せる | Phase 2 |
| ネイティブアプリ配信 (MSIX / ClickOnce) | PWA 優先 | Phase 3 |
| 端末設定コピー | アプリ増加後に現実的 | Phase 3 |
| 申請ワークフロー / 稟議承認 | Dapr Workflow が必要 | Phase 4 |
| レガシー .NET Framework ラップ配信 | 最も複雑 | Phase 4〜5 |
| Cosign / Kyverno | 構築コスト抑制 | Phase 2 |
| マルチクラスタ | 1 クラスタで十分 | Phase 3 |
| AD 連携 | ローカル DB で数十名を運用 | Phase 2 |

---

## 6. MVP-1 ハードウェア要件

### コンポーネント別メモリ概算

| コンポーネント群 | 概算メモリ |
|---|---|
| k8s システム (3 ノード) | 6 GB |
| Istio + Envoy Gateway | 2.5 GB |
| Kafka (3 broker KRaft) | 6 GB |
| 可観測性 (Prometheus / Loki / Jaeger / Grafana / OTel) | 5 GB |
| Valkey + CloudNativePG + PostgreSQL | 2.5 GB |
| Dapr Control Plane | 1 GB |
| Keycloak + Backstage + Harbor + Argo CD + GHA runner | 10 GB |
| tier1 Go + 配信ポータル | 1 GB |
| **合計** | **約 35 GB** (余裕を見て 40 GB 以上推奨) |

### ノード別要件

| 項目 | 最小要件 | 推奨要件 |
|---|---|---|
| ノード数 | 3 | 3 |
| vCPU / ノード | 8 | 16 |
| メモリ / ノード | 16 GB | 32 GB |
| ディスク / ノード | 200 GB SSD | 500 GB SSD |
| ネットワーク | 1 Gbps | 10 Gbps |

最小要件はビルド時に不安定になる可能性がある。推奨要件なら Phase 2 まで余裕あり。SSD 必須。

### Phase 別スケール想定

| Phase | ノード数 | ノードスペック |
|---|---|---|
| MVP-1 | 3 | 8〜16 vCPU / 16〜32 GB / 200〜500 GB SSD |
| Phase 2 | 3〜5 | 16 vCPU / 32 GB / 500 GB SSD |
| Phase 3 | 5〜8 | 同上 |
| Phase 5 | 10+ | 同上 + ストレージノード追加 |

---

## 7. MVP-1 のゴールと完了条件

### ゴール

| 軸 | ゴール |
|---|---|
| 機能 | パイロット業務の Web アプリが配信ポータルから起動できる |
| 開発体験 | 雛形生成 CLI でリファレンスと同等のサービスが立ち上がる |
| 運用 | Backstage から Software Catalog / TechDocs / Argo CD 状態を確認できる |
| 認証 | Argo CD / Harbor / Backstage / 配信ポータルが Keycloak SSO で統一 |
| 再現性 | `tofu apply` で k8s クラスタが再構築できる |
| 差別化 | オンプレ k8s で完結し、クラウド依存が一切ない |

### 完了判定

1. GHA フロー (PR → ビルド → スキャン → Harbor push → GitOps → Argo CD 同期) が疎通
2. パイロット業務の Web アプリがエンドユーザーから起動可能
3. Keycloak SSO が全ツールで機能
4. Backstage Software Catalog にリファレンス実装が登録済み
5. 運用手順書 (再起動 / バックアップ / リストア / ログ閲覧) が TechDocs で公開
6. **起案者以外の協力者が、手順書に従い独立して環境を再構築できる** (バス係数 2 の実証)

項目 6 がバス係数問題への直接的な回答であり、MVP-1 の最重要完了条件とする。

---

## 関連ドキュメント

- [`00_フェーズ計画.md`](./00_フェーズ計画.md) — 全フェーズの俯瞰
- [`02_体制と役割.md`](./02_体制と役割.md) — MVP-0 / MVP-1 の体制
- [`../05_CICDと配信/02_アプリ配信ポータル.md`](../05_CICDと配信/02_アプリ配信ポータル.md) — 配信ポータルの Phase 分離
- [`../06_競合と差別化/03_TCOとBuildVsBuy.md`](../06_競合と差別化/03_TCOとBuildVsBuy.md) — Build リスク低減との関係
