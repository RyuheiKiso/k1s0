# ADR-CICD-002: Progressive Delivery に Argo Rollouts を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / tier1 開発チーム / 運用チーム

## コンテキスト

tier1 / tier2 / tier3 の本番デプロイで、Kubernetes 標準の Deployment（RollingUpdate）のみを使うとリスクが大きい。特に以下の場面で Progressive Delivery が必要。

- **tier1 のような基盤更新**: 不具合が全テナントに一斉伝播するリスク
- **ZEN Engine ルール評価器の更新**: 業務判定の正誤に直結（FMEA RPN 126）
- **Temporal ワークフロー実装の更新**: Determinism 違反で既存ワークフローが失敗する可能性（FMEA RPN 135）

段階的リリース（Canary / Blue-Green）でメトリクス監視付きの自動 Rollback を実現する必要がある。

## 決定

**Progressive Delivery ツールは Argo Rollouts（CNCF Graduated 候補、Apache 2.0）を採用する。**

- Argo Rollouts 1.7+
- Canary 戦略を標準、全 tier1 / tier2 の本番デプロイで適用
- AnalysisTemplate で Prometheus / Mimir のメトリクス（エラー率、レイテンシ）を判定条件化
- 失敗時は自動 Rollback、成功時は次の Canary Weight へ昇格
- Istio Ambient（ADR-0001）の waypoint proxy とトラフィック分割を連携
- DX-FM（Feature Management、flagd）と役割分担: Argo Rollouts は「コードデプロイの段階的公開」、flagd は「機能の段階的公開」

## 検討した選択肢

### 選択肢 A: Argo Rollouts（採用）

- 概要: Argo プロジェクト傘下、Canary/Blue-Green 特化
- メリット:
  - Argo CD との親和性最大（同じ運用 UX）
  - AnalysisTemplate で Prometheus メトリクスを判定条件に
  - Istio / NGINX / AWS ALB / Traefik 等のトラフィック分割をネイティブ対応
  - 自動 Rollback のロジックが成熟
- デメリット:
  - CRD（Rollout）が Deployment と構造異なり、学習コスト
  - tier2/tier3 開発者が Rollout を書く必要（Backstage テンプレ化で緩和）

### 選択肢 B: Flagger

- 概要: Flux 傘下、Progressive Delivery
- メリット: Flux との統合が自然、軽量
- デメリット: Argo CD 採用の k1s0 では親和性が Argo Rollouts より劣る

### 選択肢 C: Istio VirtualService 直接制御

- 概要: Istio のトラフィック分割を手動制御
- メリット: ツール追加なし
- デメリット:
  - 段階的 Weight 変更を手動、ミス時の Rollback も手動
  - メトリクス判定が手動

### 選択肢 D: Kubernetes Deployment のみ（Progressive Delivery なし）

- 概要: 標準の RollingUpdate のみ
- メリット: シンプル
- デメリット:
  - 不具合デプロイ時の影響範囲が大きい
  - Rollback が手動、時間を要する

## 帰結

### ポジティブな帰結

- 本番デプロイのリスク低減、不具合を早期検知
- SLO 違反時の自動 Rollback でエラーバジェット消費を最小化
- DORA Four Keys の Change Failure Rate 改善（Elite 15% 未満）
- ZEN Engine / Temporal のような高 RPN コンポーネントの段階公開が可能

### ネガティブな帰結

- Rollout CRD 学習コスト、Backstage テンプレ化で軽減が必要
- AnalysisTemplate のメトリクス設計をコンポーネント別に要検討
- Canary 時のトラフィック分割は Istio 側の設定（DestinationRule）と整合性必要

## 実装タスク

- Argo Rollouts Helm Chart バージョン固定、Argo CD 管理
- Rollout テンプレート（Canary / Blue-Green）を Backstage Software Template 化
- AnalysisTemplate 共通セット（error rate、latency p99、CPU 使用率）を整備
- Istio Ambient Mesh との Traffic Split 連携を POC で検証（ADR-0001 と合流）
- tier1 新 API / ルール更新のデプロイに Canary 必須化を CI/CD ポリシーで強制（Kyverno）

## 参考文献

- Argo Rollouts 公式: argoproj.github.io/rollouts
- Progressive Delivery Principles
- DORA State of DevOps Report（Change Failure Rate）
