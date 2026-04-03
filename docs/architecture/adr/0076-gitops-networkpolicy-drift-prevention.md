# ADR-0076: GitOps 導入による NetworkPolicy ドリフト防止

## ステータス

提案

## コンテキスト

K8S-CRIT-002 の調査により、クラスターに適用されている NetworkPolicy と Git リポジトリのマニフェスト間でドリフト（乖離）が発覚した。
具体的には、手動での `kubectl apply` による緊急対応や設定変更が蓄積した結果、
リポジトリに存在しない NetworkPolicy がクラスターに適用されている、またはリポジトリの最新版がクラスターに反映されていないという状態が生じた。

現状、NetworkPolicy の更新は手動運用に依存しており、変更のたびに担当者が以下を実行する運用となっている:

```bash
kubectl apply -f infra/kubernetes/network-policies/system.yaml
```

この手動手順では適用漏れや適用タイミングのズレが発生しやすく、セキュリティポリシーの信頼性を損なうリスクがある。

## 決定

GitOps ツール（ArgoCD または Flux CD）を導入し、`infra/kubernetes/network-policies/` 配下のマニフェストを
クラスターの Desired State として継続的に同期・管理する。

また、GitOps 導入前の移行期間および継続的な検証として、CI パイプラインに定期的な `kubectl diff` による乖離検出を組み込む。

### 即時対応（現在の運用手順）

NetworkPolicy の乖離を検出した場合は以下のコマンドで即時更新できる:

```bash
kubectl apply -f infra/kubernetes/network-policies/system.yaml
```

### CI への乖離検出組み込み

```yaml
# .github/workflows/networkpolicy-drift.yaml
- name: NetworkPolicy ドリフト検出
  run: |
    kubectl diff -f infra/kubernetes/network-policies/system.yaml
```

差分がある場合は CI を失敗させ、手動確認を強制する。

## 理由

- GitOps によりリポジトリが Single Source of Truth となり、クラスター状態との一貫性が保証される
- 変更履歴が Git にすべて残るため、監査証跡の確保が容易になる
- `kubectl diff` による CI 組み込みは既存インフラへの影響なしに即時適用できる
- ArgoCD / Flux は k1s0 の Helm チャート構成と親和性が高い

## 影響

**ポジティブな影響**:

- NetworkPolicy のドリフトを自動検出・修正可能になる
- セキュリティポリシーの変更がコードレビューを経て適用されるため、誤設定のリスクが低減する
- CI により乖離が即座に検出される

**ネガティブな影響・トレードオフ**:

- ArgoCD または Flux の導入・運用コストが発生する
- GitOps エージェントがクラスターへの書き込み権限を持つため、エージェント自体のセキュリティ対策が必要
- 既存の手動デプロイフローからの移行作業が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 手動での定期適用（現状） | 担当者が定期的に `kubectl apply` を実行する | 適用漏れ・ドリフトが発生しやすく、根本的な解決にならない |
| Terraform による管理 | Terraform の kubernetes provider で NetworkPolicy を管理する | Helm/Kustomize との二重管理になりオペレーション複雑性が増す |
| カスタムコントローラー実装 | GitOps 相当の機能を自前実装する | 開発コストが高く、成熟した OSS を使う方が合理的 |

## 参考

- [ADR-0063: PSS NetworkPolicy default-deny 導入](0063-pss-network-policy-default-deny.md)
- [infra/kubernetes/network-policies/system.yaml](../../../infra/kubernetes/network-policies/system.yaml)
- [ArgoCD 公式ドキュメント](https://argo-cd.readthedocs.io/)
- [Flux CD 公式ドキュメント](https://fluxcd.io/docs/)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（K8S-CRIT-002 対応） | @kiso |
