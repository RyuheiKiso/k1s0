---
id: plan.tier1.scenario_supply_chain_incident
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - E
  proof_classes: []
---

# supply chain 障害対応

## 一文方針

Harbor mirror / cosign 署名 / SBOM 生成 / supply chain lint のいずれかで障害・違反が検出された際に、障害種別を即時分類し production への影響を Kyverno admission で物理遮断しながら復旧を完結させ、`supply_chain_incident.lock.yaml` に全記録を残す。

> 深夜 2 時、自宅 on-call の tier1 担当者（シニア級）が Mattermost のアラート通知で起床し、Harbor mirror の cosign 検証 fail アラートを確認する。手元には Kyverno admission ログと `supply_chain_incident.lock.yaml`、Mattermost 越しに ops 軸担当者と security 軸担当者・dual reviewer がいる。

## Trigger（発火条件）

Harbor mirror / cosign 署名 / SBOM 生成 / supply chain lint のいずれかで障害・違反が検出された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: イベント駆動（障害発生時）
- 典型きっかけ: 「Harbor mirror の disk がフルになり image pull が失敗し始めた」「OSS に重大な CVE が公開されて cosign verification が fail した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: ops 軸担当者（Harbor mirror 復旧 / Kyverno policy 変更）/ security 軸担当者（cosign 検証 fail 時の postmortem 必須参加）/ dual reviewer（tier1 担当者 2 名）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 自宅 on-call / 本社 IT 室 | Mattermost アラート | 障害種別分類・対処指揮・incident log 記録・PR 提出 |
| 関与（dual reviewer A）| シニア | リモート / 本社 IT 室 | Mattermost アラート | 対処方針確認・sign-off |
| 関与（dual reviewer B）| シニア | リモート / 本社 IT 室 | Mattermost アラート | 対処方針確認・sign-off |
| 関与（ops 担当者）| 中堅 | 自宅 on-call / 本社 IT 室 | Backstage Catalog | Harbor mirror 復旧・Kyverno policy 変更実施 |
| 関与（security 担当者）| 中堅 | リモート / 本社 IT 室 | security alert dashboard | cosign 鍵確認・postmortem 参加・影響範囲調査 |

## 前提

- Kyverno admission policy が全 production cluster で有効であり、cosign 未署名 image は拒否される（defense-in-depth 層 E）
- Harbor mirror が supply chain の単一 ingress として機能しており、upstream registry への直接アクセスは通常 policy で禁止されている
- Syft / trivy が SBOM 生成パイプラインに組み込まれている
- `supply_chain_incident.lock.yaml` が障害記録の管理ファイルとして存在する
- CI の依存導入 lint が AGPL/SSPL 系 OSS の混入を検出する

## 流れ

1. **障害種別の分類**: 検出されたアラート / CI fail を以下の 4 種別に分類し、対応フローを選択する。
   - Harbor mirror 到達不可: ネットワーク障害 / Harbor 自体の障害 / disk 容量枯渇等
   - cosign 検証 fail: image の署名が存在しない / 署名が改竄されている / 署名鍵が revoke されている
   - SBOM 欠落: SBOM 生成パイプラインの障害 / Syft / trivy のバグ
   - 依存導入 lint fail: AGPL/SSPL 系 OSS の混入 / 新規 OSS の未登録 / ライセンス変更

2. **Harbor mirror 到達不可の対処**:
   - Kyverno policy を一時的に修正し、fallback 経路（upstream registry 直引き）を許可する（ops 軸担当者が実施、tier1 の承認が必要）
   - 許可スコープは特定 namespace / 特定 image のみに最小化する
   - ops 軸担当者と連携して Harbor の復旧を最優先で実施する（disk 拡張 / Pod restart / ネットワーク疎通確認）
   - Harbor 復旧後、Kyverno policy の fallback 許可を即時に閉じる（許可を開いたままにしない）
   - 復旧後、直引きされた全 image を Harbor mirror 経由で再 push し cosign 署名を実施する

3. **cosign 検証 fail の対処**:
   - 疑わしい image（署名なし / 署名改竄）を production cluster から即時隔離する。Kyverno が既に admission で拒否しているため新規展開は阻止済み。既存 Pod を drain する。
   - 署名鍵の状態を確認: 鍵が revoke されていない場合は re-sign を実施。鍵が漏洩した場合は鍵ローテーションを security 軸と連携して実施する。
   - postmortem を必須とする。security 軸担当者が参加し、原因・影響範囲・対策を文書化する。
   - 全 image が cosign signed になるまで production への新規展開を停止する。

4. **SBOM 欠落の対処**:
   - SBOM 生成パイプライン（Syft / trivy の CI ステップ）を確認し、エラーログから障害原因を特定する
   - パイプラインの修正後、影響を受けた image の SBOM を retrospectively 生成する
   - SBOM が存在しない image は Harbor mirror 上で quarantine タグを付ける
   - CI の rerun で SBOM 生成 green を確認してから quarantine タグを解除する

5. **依存導入 lint fail の対処**:
   - fail レポートから混入した OSS と混入経路（直接依存 / 推移的依存）を特定する
   - AGPL/SSPL 系 OSS が混入している場合: 当該 deps を依存グラフから削除し PR を revert する。merge 阻止が維持される。
   - 未登録 OSS の場合: [シナリオ 01](01_新規OSS採用評価.md) の採用評価フローを経てから改めて PR を作成する
   - ライセンス変更（既存 OSS が AGPL/SSPL に変更）の場合: [シナリオ 02](02_OSSライフサイクルイベント対応.md) の OSS ライフサイクルイベント対応フローを起動する

6. **incident log の記録と dual reviewer sign-off**: 全ケースで以下を `supply_chain_incident.lock.yaml` に追記する。
   - 検出日時 / 種別 / 影響範囲
   - 対処内容と実施者
   - 復旧確認の方法と結果
   - 再発防止策
   - dual reviewer（tier1 2 名）の sign-off

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **警報配信**: cosign 未署名 image が production に混入した場合、警報配信 Pod が差し替えられ、アラート経路が改竄されるリスクがある。Kyverno による物理遮断が最前線の防護となる。
- **FA 生産指示・設備操作**: Harbor mirror 停止中は設備制御コンポーネントの image pull が停止し、設備操作システムのデプロイ・更新が全停止する可能性がある。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [OSSライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [tier1強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)

**関連 OSS**:
- Harbor: supply chain の内部 mirror / image registry / NuGet / cargo proxy
- cosign（Sigstore）: image の署名 / 検証（keyless or key-based）
- Syft: SBOM 生成（SPDX / CycloneDX 形式）
- trivy: vulnerability scan と SBOM 生成の統合ツール
- Kyverno: admission policy エンジン（defense-in-depth 層 E として機能）

## 期待結果 / 観測指標

- supply chain の完全復旧（Harbor mirror 正常稼働 / cosign signed tag が全 image で green / SBOM 全件存在）
- Kyverno admission policy で cosign 未署名 image の拒否が継続して有効
- `supply_chain_incident.lock.yaml` に incident 記録が追記されている
- AGPL/SSPL 系 OSS の混入が 0 件（依存導入 lint green）
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- postmortem が作成されている（cosign 検証 fail の場合のみ必須）

## 失敗時の挙動 / escalation

- **production への未署名 image 展開試行**: Kyverno admission で拒否（defense-in-depth 層 E）。拒否ログを security 軸に即時通知する。escalate 先: security 担当者へ Mattermost `#security-incident` で即時報告（SLA: 検出後 15 分以内）。Backstage runbook `kyverno-unsigned-image-block` を参照。
- **Harbor mirror 復旧不能（長時間）**: SLA 内に復旧できない場合は DR 計画を発動し、backup registry への切替を ops 軸と連携して実施する。escalate 先: ops 担当者へ Mattermost `#tier1-incident` で報告（SLA: 障害検出後 2 時間以内に DR 計画発動）。Backstage runbook `harbor-dr-activation` を参照。
- **cosign 鍵漏洩疑い / 確定**: 全署名済み image の信頼性が失われる。鍵ローテーション後に全 image を re-sign するまで production 展開を全停止する。security 軸が incident commander として対応を統括する。escalate 先: security 担当者へ Mattermost `#security-incident` で即時通報（SLA: 検出後 30 分以内）。Backstage runbook `supply-chain-key-rotation` を起動。postmortem は 5 営業日以内に提出。
- **AGPL/SSPL 混入の推移的依存（自動削除困難）**: 依存グラフを解析し直接依存を削除することで推移的依存を排除する。削除できない場合は当該機能の提供を一時停止する。escalate 先: tier1 担当者 dual reviewer + security 担当者（SLA: 検出後 4 時間以内に削除または機能停止）。

## Timeline

| T+ | actor | action | Mattermost 投稿例 |
|---|---|---|---|
| 0 | tier1 担当者 | アラート検知・障害種別宣言 | `@security-oncall cosign 検証 fail 検出 / 対象: harbor.internal/k1s0/api-gateway:v1.2.3 / Kyverno 遮断中` |
| 5 分 | tier1 担当者 | 疑わしい image を production cluster から drain 開始 | `既存 Pod drain 実施中 / 新規展開は Kyverno により阻止済み` |
| 15 分 | security 担当者 | SBOM 照合・影響範囲調査開始 | `SBOM 照合中 / 影響 image 候補: 3 件 / 鍵 revoke 確認待ち` |
| 30 分 | tier1 担当者 | 署名鍵の状態確認・re-sign または鍵ローテーション判断 | `鍵 revoke なし確認 / 対象 image を re-sign 実施中 / Harbor push 完了後 Kyverno 確認予定` |
| 60 分 | tier1 担当者 | 全 image re-sign 完了・Harbor mirror 正常確認 | `re-sign 完了 / Kyverno admission green / production 展開再開可` |
| 1 営業日 | tier1 担当者 | postmortem 着手 | `#postmortem supply_chain_incident postmortem PR 作成済 / 根本原因: CI pipeline の cosign step skipped 条件の誤設定` |

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [新規 OSS 採用評価](01_新規OSS採用評価.md)
- [OSS ライフサイクルイベント対応](02_OSSライフサイクルイベント対応.md)
