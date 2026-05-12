---
id: plan.tier1.scenario_server_component_addition
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# Server 系コンポーネント追加

## 一文方針

新 Server 系コンポーネントを 5 系統（Gateway / Sidecar+Agent / Backend-for-Library / Control Plane / Operator+Controller）に正確に位置付けた上で、proto 定義・Kyverno policy・Tilt dev loop・SLO 定義を同時に整備し、dual reviewer sign-off と全言語 CI green を揃えて merge する。

## Trigger（発火条件）

5 系統（Gateway / Sidecar+Agent / Backend-for-Library / Control Plane / Operator+Controller）のいずれかに新しいコンポーネントを追加する要求が来た時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 四半期
- 典型きっかけ: 「フィーチャーフラグの動的配信 Server として新 Control Plane コンポーネントが必要になった」「マルチリージョン対応で新 Gateway コンポーネントの追加が要求された」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ infra 軸担当者（Kyverno policy レビュー）/ ops 軸担当者（SLO 定義連携）

## 前提

- Server 系の 5 系統（Gateway / Sidecar+Agent / Backend-for-Library / Control Plane / Operator+Controller）の設計方針が確立済み（Sidecar と Agent は同一系統、Operator と Controller は同一系統）
- Tilt による local dev loop 環境が稼働している
- `slo_catalog.lock.yaml` が SLO / SLI 定義の管理ファイルとして存在する
- Rust（Operator のみ Go + controller-runtime / kubebuilder）が実装言語として採用済み

## 流れ

1. **コンポーネント分類**: 追加コンポーネントを以下の分類のいずれに該当するか明確に位置付ける。
   - Gateway: 外部トラフィックの受付・ルーティング・認証・レート制限
   - Sidecar: アプリ Pod に注入され observability / ネットワーク透過処理を担う
   - Agent: 非同期ジョブ実行 / スケジューラ / イベントドリブン処理
   - Backend-for-Library: Library からのバックエンド呼び出しを受け付ける専用サービス
   - Control Plane: 分散システム全体の設定・状態管理・調停
   - Operator: カスタムリソース（CRD）を通じた Kubernetes リソースのライフサイクル管理
   - Controller: 既存 Kubernetes リソースの調停ループ（Operator サブコンポーネント）

2. **実装言語の確定**: 以下の方針で実装言語を確定する。
   - Operator / Controller: Go + controller-runtime / kubebuilder（kubebuilder scaffold を起点）
   - 上記以外: Rust（tokio / tonic を基盤として使用）
   - 言語の例外申請は dual reviewer + tier1 全体合意が必要

3. **Internal proto 定義の追加**: 新コンポーネントが公開するエンドポイントの Internal proto を追加し Buf によるコード生成を確認する。External proto が必要な場合は [シナリオ 03](03_proto二層化スキーマ進化.md) を先行して実施する。

4. **Kyverno admission policy の宣言**: 新コンポーネントの権限・ネットワーク policy を Kyverno で宣言する。
   - ServiceAccount の権限範囲（RBAC）を最小権限原則で設定
   - NetworkPolicy で許可する ingress / egress を明示
   - Pod security context（runAsNonRoot / readOnlyRootFilesystem 等）を設定
   - policy は infra 軸担当者がレビューする

5. **Tilt dev loop への追加**: `Tiltfile` に新コンポーネントを追加し inner loop 動作確認を実施する。
   - ライブリロード（ファイル変更検出 → 自動ビルド・デプロイ）が機能することを確認
   - 隣接コンポーネントとの通信が local 環境で成立することを確認

6. **SLO 定義の追加**: `slo_catalog.lock.yaml` に新コンポーネント分の SLI / SLO を追加する。
   - SLI: 可用性・レイテンシ・エラー率の計測方法を定義（Prometheus query で表現）
   - SLO: 月次目標値と error budget を設定
   - ops 軸担当者と連携して Alertmanager / Grafana dashboard を設定

7. **dual reviewer sign-off と CI green**: dual reviewer（tier1 2 名）の sign-off を取得する。全言語（Rust / C# / Go / TypeScript）の CI が green であることを確認する。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [SLO 適合仕様](../../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
- [tier1強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)
- [HTTP2 enforcement 適合仕様](../../../04_詳細設計/03_クロスカッティング適合仕様/01_HTTP2_enforcement.md)

**関連 OSS**:
- tokio / tonic: Rust 非同期 runtime / gRPC 実装
- kubebuilder / controller-runtime: Go Operator 開発フレームワーク
- Tilt: local dev loop 環境
- Kyverno: Kubernetes admission policy エンジン
- Prometheus / Grafana: SLI 計測と可視化

## 期待結果 / 観測指標

- 新コンポーネントが 5 系統のいずれかに明確に位置付けられ設計書に記載されている
- Internal proto が追加され Buf コード生成が全言語で成功している
- Kyverno policy が追加され infra 軸担当者のレビューが完了している
- Tilt dev loop で新コンポーネントが正常に起動し隣接コンポーネントと通信できる
- `slo_catalog.lock.yaml` に新コンポーネントの SLI / SLO が記録されている
- 全言語の CI が green
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている

## 失敗時の挙動 / escalation

- **分類が曖昧**: 5 系統のいずれにも適合しない場合、tier1 全体での設計議論に escalate する。新系統の追加は設計方針改定として扱い別 PR を作成する。escalate 先: tier1 担当者 dual reviewer（SLA: 5 営業日以内に方針決定）。
- **Kyverno policy 過剰権限検出**: infra 軸が policy レビューで拒否。最小権限原則に基づいて RBAC を修正する。escalate 先: infra 担当者へ Mattermost `#tier1-incident` で連絡（SLA: 8 時間以内に修正案提示）。Backstage runbook `kyverno-policy-overpermission` を参照。
- **Tilt dev loop 動作不良**: 隣接コンポーネントとの依存関係が未設定の可能性がある。Tiltfile の依存定義を修正する。escalate 先: tier1 担当者 dual reviewer（SLA: 24 時間以内に修正）。
- **SLO 定義なしの merge 試行**: `slo_catalog.lock.yaml` 更新なしの PR は CI gate で merge 阻止。ops 軸と SLO を合意してから merge する。escalate 先: ops 担当者（SLA: 48 時間以内に SLO 合意）。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [proto 二層化スキーマ進化](03_proto二層化スキーマ進化.md)
