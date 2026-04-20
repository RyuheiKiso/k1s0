# WBS と工程表

本書は k1s0 プロジェクトの作業分解構造（WBS: Work Breakdown Structure）と工程表（マイルストーン、依存関係、所要期間）を定める。Phase 1〜3 の各段階で「何を、いつまでに、誰が」作るかを明文化し、予算・体制・リスクを合致させる。

## Phase 設計の思想

k1s0 は以下の 3 Phase で段階進行する。各 Phase は独立して稟議・投資判断の対象になり、次 Phase 進行可否は Product Council が評価する Gate で判断する。Big Bang 方式ではなく段階的ロードマップを採る理由は、1）投資リスクの分散、2）早期に実価値検証、3）Phase 間で要件・優先度を再評価できる柔軟性確保、の 3 つ。

- **Phase 1（M1〜M9）**: Foundation — tier1 公開 6 API（Service Invoke / State / PubSub / Log / Telemetry / Audit）、単一クラスタ、内製 Platform Team 立上げ、最初の 1 業務アプリで実証
- **Phase 2（M10〜M18）**: Expansion — 残り 5 API（Secrets / Binding / Workflow / Decision / Pii / Feature）、複数業務アプリ本番、Self-Service オンボーディング
- **Phase 3（M19〜M30）**: Maturity — マルチクラスタ、レガシー段階撤退、AI/ML 基盤追加、外販検討

## Phase 1 WBS（M1〜M9 = 9 ヶ月）

### WP1.1: 基盤環境構築（M1〜M3）

- 物理サーバー調達・設置（リードタイム 2 ヶ月）
- Kubernetes クラスタ構築（kubeadm / Rancher 検討）
- ネットワーク設計（MetalLB BGP、CNI 選定）
- ストレージ構築（Longhorn）
- 観測性最小セット（Prometheus、Loki、Grafana）
- Argo CD 初期化

### WP1.2: tier1 コア API 実装（M2〜M7）

- Protobuf IDL 策定、全 API の契約定義
- tier1 Go SDK（Dapr ファサード部分）実装
- tier1 Rust 自作領域（ZEN Engine 統合、crypto、雛形 CLI）実装
- 公開 API 6 本の実装: Service Invoke / State / PubSub / Log / Telemetry / Audit
- tier2 クライアント SDK（Go/Rust/TypeScript）提供

### WP1.3: セキュリティ基盤（M3〜M6）

- Keycloak セットアップ、Realm 設計
- OpenBao セットアップ、Dynamic Secret 動作確認
- SPIFFE/SPIRE 導入、ワークロード ID 発行
- Istio Ambient 導入、mTLS 強制
- Kyverno ポリシー整備、Sigstore 検証設定

### WP1.4: データ基盤（M2〜M5）

- CloudNativePG クラスタ構築（dev / staging / prod）
- Strimzi Kafka 構築
- MinIO 構築、AGPL 分離検証
- Valkey キャッシュ構築

### WP1.5: DevEx とテナント管理（M4〜M8）

- Backstage 構築、SSO 統合
- Software Template（Golden Path）作成
- Self-Service Onboarding（最小実装）
- ドキュメントポータル（TechDocs）統合

### WP1.6: 最初の 1 業務アプリ実証（M6〜M9）

- パイロット業務部門選定、要件確定
- tier2 側実装、k1s0 API 統合
- 結合試験、性能試験、監査対応
- 本番リリース（Argo Rollouts Canary）

### WP1.7: 運用体制立上げ（M7〜M9）

- Runbook 整備（最低 10 本）
- オンコールローテーション開始
- 初期トレーニング（BC-TRN）実施
- SLO 計測、エラーバジェット運用開始

### Phase 1 Gate（M9）

- パイロット業務アプリの 90 日本番稼働実績
- SLO 達成率 99.5% 以上
- 監査部門レビュー合格
- Phase 2 予算承認

## Phase 2 WBS（M10〜M18 = 9 ヶ月）

### WP2.1: 残り 5 API 実装（M10〜M14）

- Secrets API（OpenBao 連携）
- Binding API（外部連携）
- Workflow API（Temporal 統合）
- Decision API（ZEN Engine 統合）
- Pii API（PII 自動判定 + Masking）
- Feature API（flagd / OpenFeature 統合）

### WP2.2: 複数業務アプリ展開（M11〜M18）

- 3〜5 業務アプリの並行立ち上げ
- Stream-aligned Team の立ち上げ支援
- 共通基盤の改善フィードバック反映

### WP2.3: Self-Service テナント管理（M12〜M15）

- Backstage 経由の完全 Self-Service フロー
- ポリシー自動適用（ZEN Engine 決定表）
- 利用量メトリクス、課金ベース計測開始

### WP2.4: 観測性・セキュリティ拡充（M13〜M17）

- Grafana LGTM 全コンポーネント本番化
- 脅威モデリング更新、ペネトレーションテスト
- SBOM / Sigstore 強制化

### WP2.5: レガシー移行パイロット（M14〜M18）

- サイドカー方式（ADR-MIG-001）で 1 つのレガシー資産を移行
- API Gateway 方式（ADR-MIG-002）で 1 つの外部公開 API を段階切替
- 挙動一致検証フレームワーク稼働

### Phase 2 Gate（M18）

- tier1 全 11 API 本番稼働
- 5 業務アプリ以上の本番稼働
- レガシー 2 パイロット完了
- Phase 3 計画承認

## Phase 3 WBS（M19〜M30 = 12 ヶ月）

- マルチクラスタ（Phase 3a: M19〜M24）: クラスタ間レプリケーション、DR サイト、フェデレーション
- レガシー段階撤退（Phase 3b: M21〜M30）: 主要レガシー資産の完全移行、Windows Node 撤退判定
- AI/ML 基盤追加（Phase 3c: M25〜M30）: GPU ノード、Kubeflow 等の検討
- 外販検討（Phase 3d: M28〜M30）: ライセンス戦略、サポート体制、顧客候補 PoC

## 依存関係図（概要）

```
WP1.1 基盤環境 ──┐
                 ├── WP1.4 データ基盤 ──┐
                 ├── WP1.3 セキュリティ ──┤
                 └── WP1.2 tier1 API ────┤
                                         ├── WP1.5 DevEx ──┐
                                         │                  ├── WP1.6 実証
                                         │                  └── WP1.7 運用
                                         └── Phase 1 Gate ──┐
                                                            ├── Phase 2 へ
                                                            └── ...
```

## マイルストーンと節目

マイルストーンは Phase ゲート判定の具体的なチェックポイントである。判定基準をラベルに閉じ込めず、「なぜこの基準がゲートなのか」「崩れた場合に何が起きるか」を各節目で明示する。この記述がないと、M3 到達時に「dev/staging/prod が Argo CD 同期しただけで良いのか」の判断が PMO と技術リードでブレ、Phase 1 の着地が後ろにずれる。

- **M3: 基盤 Kubernetes 稼働**（Phase 1a 後半）— dev / staging / prod の 3 環境で Argo CD が同期を完了し、Kyverno ポリシーの Audit モード稼働、SPIRE が SVID を払い出せる状態を合格とする。この節目が遅れると tier1 API 実装（M4 以降）が開始できず、全体の臨界経路がシフトする
- **M5: tier1 コア 6 API α 版**（Phase 1a 末）— State / PubSub / Service Invoke / Secrets / Log / Telemetry の 6 API が社内検証環境で動作し、Go SDK と IDL α 版が公開済み、API ドキュメントの α 版 が Backstage に掲載されていることを合格とする。崩れると tier2 先行チームが契約テストを書けず、Phase 1b のパイロット参加が先送りになる
- **M7: パイロット業務アプリ staging 稼働**（Phase 1b 前半）— パイロット 1 本が結合試験合格、負荷試験で想定トラフィックの 1.5 倍で SLO 維持、Runbook 10 本整備を合格条件とする。崩れた場合の影響は大きく、稟議で約束した「Phase 1b パイロット 1Q 運用」のタイムラインが成立しなくなる
- **M9: Phase 1 本番リリース**（Phase 1b 末）— Argo Rollouts の Canary を 100% 昇格、SLO 達成率 30 日間 99.5% 以上、Critical CVE ゼロ、監査・法務・セキュリティの明示承認を合格とする。本ゲートで未達があれば、本番受入を延期してもエラーバジェット回復と Runbook 整備を優先する
- **M14: Phase 2 α リリース**— 全 11 API の α 公開、3 業務アプリが staging で稼働、ZEN Engine / Temporal の本番採用判断完了を合格とする。未達は P-BIZ-003 の規模拡大判定（U-001）の延期に直結する
- **M18: Phase 2 本番拡大完了**— 5 業務アプリ以上が本番稼働、SLO 継続達成、MFA（NFR-E-AC-005）本番運用開始を合格とする
- **M24: マルチクラスタ稼働**— DR サイト切替試験合格、RTO 4 時間・RPO 数秒を訓練で実証することを合格とする
- **M30: Phase 3 完了**— レガシー移行 80% 達成、撤退戦略またはスケールアウト戦略の外販判断を合格とする

各マイルストーンは PMO が月次で進捗を追い、Product Council が Phase ゲートで承認する。M7 以降の合格条件はリリース承認チェックリスト（70_プロジェクト管理/03_QA計画 のゴーノーゴー基準）と対応する。

## 工程リスクと対策

WBS の各工程には以下の典型リスクがある。リスクレジスタ（[00_要件定義方針/06_リスクレジスタ.md](../00_要件定義方針/06_リスクレジスタ.md)）と対応づけ、Phase ゲートで再評価する。

- **物理サーバ調達遅延**: M1〜M3 のクリティカルパス。2 ヶ月バッファを積む、代替クラウド一時利用の検討
- **スキル不足**: Rust / SRE / PKI の人材確保。Phase 0 採用を前倒し、外部研修 3 ヶ月
- **tier1 API 設計のやり直し**: Phase 1 中盤での大幅変更は致命的。Phase 0 で IDL α 版を確定、tier2 と早期に契約テスト共有
- **レガシー移行の予想外の複雑性**: Phase 2 パイロットで 1 本だけ先行、学びを Phase 3 本格移行に反映

## 関連ドキュメント

- [01_体制と役割.md](01_体制と役割.md)
- [03_QA計画.md](03_QA計画.md)
- [05_移行計画.md](05_移行計画.md)
- [../00_要件定義方針/06_リスクレジスタ.md](../00_要件定義方針/06_リスクレジスタ.md)
