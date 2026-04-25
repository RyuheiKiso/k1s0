# WBS と工程表

本書は k1s0 プロジェクトの作業分解構造（WBS: Work Breakdown Structure）と工程表（マイルストーン、依存関係、所要期間）を定める。k1s0 は起案者の個人開発 OSS としてリリース時点で tier1 11 API 全機能を本格実装する一気通貫方針であり、本書は「リリース時点までの開発 WBS」と「採用側の運用工程の参考モデル」の二層構造で記述する。

## 全体構成の思想

k1s0 は段階分割リリースを採らない。リリース時点で tier1 公開 11 API（Service Invoke / State / PubSub / Secrets / Binding / Workflow / Decision / Log / Telemetry / Pii / Feature）の全機能を本格実装し、Apache 2.0 ライセンスで取消不能に GitHub に公開する。これにより採用側は「リリース版を取り込んでそのまま採用」できる状態を提供する。

一方で、採用側の組織が k1s0 を導入する際は、組織規模・業務影響度・既存資産との整合度に応じて段階的にロールアウトすることが現実的である。本書は「OSS としてのリリースまでの WBS」と「採用側の運用ステップの参考モデル」を分けて記述し、両者を混同しないよう構造化する。

## OSS リリースまでの WBS

起案者個人による開発工程を以下の作業パッケージ（WP）に分解する。WP 間の依存関係を明示し、リリース時点で 11 API 全機能本格実装の状態に到達することを目標とする。

### WP-1: 基盤環境構築

- Kubernetes クラスタ構築（kubeadm を中心とした upstream 構成）
- ネットワーク設計（MetalLB L2、Calico CNI、後の Cilium 移行を考慮した抽象化）
- ストレージ構築（Longhorn）
- 観測性最小セット（Prometheus、Loki、Tempo、Grafana、Pyroscope）
- Argo CD 初期化、GitOps パイプライン稼働

### WP-2: tier1 コア API 実装

- Protobuf IDL 策定、tier1 公開 11 API の契約定義
- tier1 Go SDK（Dapr ファサード部分）実装
- tier1 Rust 自作領域（ZEN Engine 統合、crypto、雛形 CLI、日本企業の業務要件に対応するドメイン固有機能）実装
- 公開 11 API の実装: Service Invoke / State / PubSub / Secrets / Binding / Workflow / Decision / Log / Telemetry / Pii / Feature
- tier2 クライアント SDK（Go / Rust / TypeScript）提供

### WP-3: セキュリティ基盤

- Keycloak セットアップ、Realm 設計
- OpenBao セットアップ、Dynamic Secret 動作確認
- SPIFFE / SPIRE 導入、ワークロード ID 発行
- Istio Ambient 導入、mTLS 強制
- Kyverno ポリシー整備、Sigstore 検証設定

### WP-4: データ基盤

- CloudNativePG クラスタ構築
- Strimzi Kafka 構築
- MinIO 構築、AGPL ライセンス分離検証
- Valkey キャッシュ構築

### WP-5: DevEx と Golden Path

- Backstage 構築、Software Template（Golden Path）作成
- Self-Service Onboarding 実装
- ドキュメントポータル（TechDocs）統合
- Scaffold CLI 整備

### WP-6: 運用基盤

- Runbook 整備（OSS 公開時点で 15 本以上）
- SLO 計測、エラーバジェット運用設計
- Chaos Engineering 演習計画整備
- DR / BCP 設計と訓練手順

### WP-7: サプライチェーンとガバナンス

- SLSA Level 2 / 3 ビルドパイプライン
- sigstore / cosign keyless 署名
- CycloneDX SBOM 自動生成
- Kyverno による二分所有モデルポリシー稼働
- ADR / Technology Radar / 脅威モデリング（STRIDE）プロセス整備

## 採用側の運用工程の参考モデル

リリース版を採用した組織が運用を立ち上げる際の参考工程を以下に示す。これは採用側組織の意思決定に従って柔軟に短縮・延長可能であり、k1s0 OSS 自体の責務範囲ではない。

### 採用初期: パイロット業務での実証

- 採用側組織が業務影響度の低い 1〜2 業務を選定
- tier2 側実装、k1s0 API 統合
- 結合試験、性能試験、監査対応
- Argo Rollouts による Progressive Delivery で本番投入
- 推奨期間: 1〜3 か月

### 採用後の運用拡大時: 複数業務アプリへの展開

- 業務影響度を段階拡大しつつ複数業務の並行立ち上げ
- Stream-aligned Team 体制への展開
- 共通基盤の改善フィードバック反映
- 推奨期間: 3〜6 か月

### 採用側のマルチクラスタ移行時: マルチクラスタ・DR 整備

- クラスタ間レプリケーション、DR サイト構築
- フェデレーション設計
- 推奨期間: 6〜12 か月

### 採用側の全社展開期: レガシー移行と全社ロールアウト

- サイドカー方式 / API Gateway 方式によるレガシー段階撤退
- 主要レガシー資産の完全移行
- 全社業務適用
- 推奨期間: 採用側組織規模により 12 か月以上

## 依存関係図（OSS リリースまで）

```
WP-1 基盤環境 ──┐
                ├── WP-4 データ基盤 ──┐
                ├── WP-3 セキュリティ ──┤
                └── WP-2 tier1 API ────┤
                                       ├── WP-5 DevEx ──┐
                                       │                ├── WP-6 運用基盤
                                       │                └── WP-7 サプライチェーン
                                       └── リリース判定 ──── OSS 公開
```

## OSS リリース判定の Go 条件

リリース時点で以下の状態に到達したことをもって OSS 公開を実施する。

- tier1 公開 11 API 全件の本格実装完了
- SLO 計測が稼働し、`tools/sparse/checkout-role.sh` を含む採用側ロール別運用が機能
- Runbook 15 本以上、Chaos Engineering 演習手順整備
- SLSA Level 2 ビルドパイプライン稼働、sigstore 署名・SBOM 自動生成
- ADR の主要意思決定（DIR / TIER1 / STOR / DATA / OBS / SEC / POL / CICD / SUP / REL / OPS / DX / DEVX / DEPLOY）が起票・採択済み
- 設計書・要件定義書・実装ガイドが一貫性を持って整備済み

## 採用側運用ステップでの参考マイルストーン

採用側組織は自組織の状況に応じて以下を参考にステップ判定を組む。これは k1s0 OSS の制約ではなく、運用立ち上げを円滑化する目安である。

- 採用初期完了の目安: パイロット 1 業務が 90 日連続で SLO 99.5% 以上を維持
- 運用拡大完了の目安: 5 業務以上が本番稼働、月次 SLO 継続達成、MFA など追加セキュリティ機能の運用開始
- マルチクラスタ完了の目安: DR サイト切替試験合格、RTO 4 時間・RPO 数秒を訓練で実証
- 全社展開完了の目安: レガシー移行 80% 達成、撤退戦略またはスケール戦略の判断完了

## 工程リスクと対策

OSS 開発側の典型リスクは以下である。リスクレジスタ（[../00_要件定義方針/07_リスクレジスタ.md](../00_要件定義方針/07_リスクレジスタ.md)）と対応づけて管理する。

- **個人開発の継続性**: 起案者の稼働が止まると OSS 維持が滞るリスク。Apache 2.0 取消不能ライセンスでフォーク・自己保守を採用側で実施できる状態を担保。
- **tier1 API 契約の破壊変更**: リリース後の互換破壊は採用側の取り込みコストを直撃。Buf breaking 検出で CI ブロック、後方互換 18 か月猶予をルール化。
- **OSS 依存の AGPL 伝播リスク**: AGPL OSS の伝播性を物理境界・運用境界で遮断する設計（OSS ライセンス運用方式）を維持。
- **サプライチェーン攻撃**: SLSA Level 2/3 + sigstore + SBOM + Kyverno 検証の四重防御で対応。

## 関連ドキュメント

- [01_体制と役割.md](01_体制と役割.md)
- [03_QA計画.md](03_QA計画.md)
- [05_移行計画.md](05_移行計画.md)
- [../00_要件定義方針/07_リスクレジスタ.md](../00_要件定義方針/07_リスクレジスタ.md)
