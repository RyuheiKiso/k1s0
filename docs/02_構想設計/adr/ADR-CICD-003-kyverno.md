# ADR-CICD-003: ポリシー適用に Kyverno を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / セキュリティチーム / 運用チーム

## コンテキスト

k1s0 では以下のポリシー適用が必要になる。

- **セキュリティポリシー**: Pod Security Standards、privileged コンテナ禁止、runAsRoot 禁止
- **イメージ署名検証**: Sigstore cosign 署名のないイメージを拒否（NFR-H-INT-001）
- **リソース制限**: CPU/Memory requests/limits 必須化
- **命名規則**: namespace / label の命名強制
- **監査ログ**: 誰が何を作ったかの証跡
- **ベストプラクティス**: `latest` タグ禁止、readiness/liveness probe 必須

Kubernetes 標準の PodSecurity admission や ValidatingAdmissionPolicy だけでは表現力が限定的で、複雑なポリシー（例: 「テナント X の Pod は node selector を必須」）を書きにくい。

候補は Kyverno、OPA/Gatekeeper、kube-policy-validator など。

## 決定

**ポリシー適用は Kyverno（CNCF Graduated、Apache 2.0）を採用する。**

- Kyverno 1.12+
- ClusterPolicy / Policy CRD で宣言的にポリシー記述
- enforce モードと audit モードを段階適用（新規ポリシーは audit → 監視 → enforce）
- イメージ署名検証（verifyImages）で Sigstore cosign 署名チェック
- Mutating Policy で自動補完（例: 欠落した label の自動付与）
- Generate Policy で リソース自動生成（例: namespace 作成時に NetworkPolicy 自動生成）
- Policy Exception で例外管理、監査ログで例外理由を記録

## 検討した選択肢

### 選択肢 A: Kyverno（採用）

- 概要: CNCF Graduated、Nirmata 発
- メリット:
  - Kubernetes YAML ライクなポリシー記述、学習曲線緩やか
  - Validating / Mutating / Generating の 3 モード標準対応
  - verifyImages で Sigstore 署名検証がネイティブ対応
  - Policy Report で監査トレイル
  - UI（Kyverno Dashboard）あり
- デメリット:
  - OPA/Gatekeeper と比べるとコミュニティ規模がやや小さい（ただし 2023 CNCF Incubating → 2024 Graduated で急成長）

### 選択肢 B: OPA / Gatekeeper

- 概要: CNCF Graduated、Rego 言語で記述
- メリット: 多様な入力（K8s 以外）に対応、業界実績豊富
- デメリット:
  - Rego 言語の学習曲線が急（関数型・宣言的）
  - Mutating / Generating モードの成熟度が Kyverno より劣る
  - イメージ署名検証は別ツール（Cosigned 等）連携が必要

### 選択肢 C: PodSecurity admission のみ

- 概要: Kubernetes 標準
- メリット: 追加ツール不要
- デメリット:
  - Pod Security Standards 3 段階（restricted/baseline/privileged）のみ
  - カスタムポリシーが書けない

### 選択肢 D: ValidatingAdmissionPolicy (VAP)

- 概要: Kubernetes 1.30 GA の標準機能、CEL 式でポリシー
- メリット: 標準機能、追加ツール最小
- デメリット:
  - 2024 年 GA で成熟度まだ
  - Mutating 非対応、用途が検証のみ

## 帰結

### ポジティブな帰結

- セキュリティポリシー違反を admission 段階で阻止、incident 事前予防
- Sigstore 署名検証が CI/CD の BC-SC-001 SLSA Level 3 達成に貢献
- Policy Report で監査対応（NFR-H-COMP-002 J-SOX）が容易
- Generate Policy で namespace 作成時の定型リソース自動化、DX 向上

### ネガティブな帰結

- admission webhook 障害時の failurePolicy 設計が重要（FMEA RPN 32）
- ポリシー更新時の影響範囲評価を慎重に（誤設定で正当操作を拒否するリスク、FMEA RPN 72）
- Policy Exception の申請ワークフローが運用設計として必要

## 実装タスク

- Kyverno Helm Chart バージョン固定、Argo CD 管理、HA 構成（kyverno-admission-controller x3）
- 共通ポリシーセット（PSS restricted、imageVerify、resource limits、naming）を GitOps で配布
- 新規ポリシーは audit モードで導入、1 週間監視 → enforce 昇格プロセスを Runbook 化
- Policy Report を Loki に連携、違反ダッシュボード整備
- Policy Exception 申請フロー（Jira / GitHub Issue 経由）を BC-GOV-005 に組込み
- Sigstore cosign 公開鍵を ConfigMap 化、ローテーション手順 Runbook

## 参考文献

- Kyverno 公式: kyverno.io
- CNCF Graduated Projects
- Pod Security Standards
- Sigstore: sigstore.dev
