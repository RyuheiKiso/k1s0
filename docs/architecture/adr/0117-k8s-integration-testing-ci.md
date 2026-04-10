# ADR-0117: K8s 統合テスト CI 環境の整備

## ステータス
提案

## コンテキスト
外部監査（LOW-006）で、Docker Desktop K8s（EOF エラー）および kind クラスター（CNI 未設定）が
接続不能のため、実際の Pod 起動と NetworkPolicy の動作確認ができなかった。
現状は `kubectl apply --dry-run` と `kubectl kustomize` のみで検証している。

実際の Pod 起動テストができないため以下のリスクが残る:
- NetworkPolicy の実際の通信制御動作が未検証
- PSS（Pod Security Standards）適合の実際の適用が未確認
- Helm チャートのデプロイ動作が未検証

## 決定
CI に kind + Calico CNI を使用した K8s 統合テストを追加する。
具体的には:
1. `.github/workflows/k8s-integration.yaml` を新規作成
2. kind クラスターに Calico CNI をインストール
3. Helm チャートをデプロイして Pod 起動を確認
4. NetworkPolicy の疎通テストを実施

## 理由
- kubectl dry-run はマニフェストの文法検証にとどまり、実際の動作を保証しない
- kind は GitHub Actions 上で動作し、フリープランでも利用可能
- Calico CNI により NetworkPolicy の実際の通信制御を検証できる

## 影響

**ポジティブな影響**:

- K8s マニフェストの実際の動作が CI で保証される
- デプロイ前に問題を検出できる

**ネガティブな影響・トレードオフ**:

- CI 実行時間が増加する（kind クラスター起動: 約 3-5 分）
- 初期セットアップ工数が必要

## 代替案

| 案 | メリット | デメリット |
|----|---------|----------|
| kind + Calico（推奨） | 完全な K8s 動作確認 | CI 時間増加 |
| minikube | ローカル開発と同一 | GitHub Actions での動作が不安定な場合がある |
| kubectl dry-run のみ（現状） | 高速 | 実際の動作保証なし |

## 参考

- `infra/kubernetes/` — K8s マニフェスト
- `infra/helm/` — Helm チャート

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-10 | 初版作成（外部監査 LOW-006 対応） | @k1s0 |
