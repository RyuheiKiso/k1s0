---
id: plan.overview.scenario_external_auditor_slsa_l3_provenance
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# SLSA_L3_provenance独立検証

## 一文方針

SLSA L3+ provenance を cosign + Rekor で独立検証する。

> 年次監査の 3 日目後半、外部監査人は k1s0 の build artifact が SLSA L3+ の要件を満たす provenance とともに供給されていることを独立検証する。cosign verify-attestation で provenance の署名を確認し、Rekor での記録を照合する。builder identity が許容された builder のみであることを確認し、build environment の隔離性が SLSA L3 要件を満たしていることを provenance の内容から確認する。

## ペルソナ要約

主役: 外部監査人（公認会計士 / 監査法人スタッフ）、目的: k1s0 の build artifact が SLSA L3+ の supply chain 整合性要件を満たしていることを cosign + Rekor で独立検証する

## 現状業務での痛み

- SLSA レベルの概念が監査人に浸透しておらず、supply chain integrity の監査が形式的な書類確認にとどまる
- cosign による署名検証の手順が専門的で、監査人が自力で実施するには学習コストが高い
- builder identity の確認手順が標準化されていないため、意図しない builder から build された artifact を見逃す可能性がある
- SLSA provenance の内容（builder / source / dependencies）を監査報告書で正確に説明できない

## k1s0 でこう変わる

- SLSA L3+ provenance が全 artifact に cosign で署名されており、外部監査人が CLI で独立検証できる
- 許容された builder identity リストが公開ドキュメントとして提供されており、外部監査人が照合できる
- Rekor に provenance が記録されているため、04 のシナリオと組み合わせて二重の独立検証経路が確保される
- cosign verify-attestation の標準的な実行手順が公開ドキュメントに記載されている

## Trigger

SLSA L3+ provenance の独立検証時（年次監査の supply chain 検証フェーズで 04 の Rekor 検証と連続して実施）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次
- 典型きっかけ: 「年次 supply chain 監査で SLSA L3+ 準拠の独立証明が必要」「規制当局から software supply chain の integrity 証明を求められた」

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 外部監査人（主役） | 外部専門家 | 監査現場 / リモート | CLI 環境（cosign） | provenance 署名検証・builder identity 確認 |
| security 担当者 | シニア | 開発拠点 | セキュリティダッシュボード | 許容 builder リスト提供・技術説明 |

## 個人 KPI / 達成感

- 対象 artifact の provenance 署名検証完了（サンプリング）
- builder identity の許容リストとの照合完了
- chain-of-custody への記録完了

## 工数 / 関与人数 / コスト感

- 標準的な検証（サンプリング 20〜50 件）: 2〜3 時間
- 関与人数: 1 名（外部監査人のみ）

## 前提

- 対象 artifact の image digest または artifact hash が提供されている
- 許容された builder identity リストが公開ドキュメントとして提供されている
- 外部監査人の環境に cosign が準備されている
- 04 のシナリオ（Rekor inclusion proof 検証）が完了している

## 流れ

1. security 担当者から対象 artifact の image digest / artifact hash リストと許容 builder identity リストを受け取る
2. cosign verify-attestation で provenance の署名を検証する
3. provenance の builder identity が許容リストに含まれていることを確認する
4. provenance の source（git repository / commit hash）が公開されたソースと一致していることを確認する
5. Rekor に provenance が記録されていることを 04 の検証結果と照合する
6. 検証結果を chain-of-custody ドキュメントに記録する
7. 監査報告書に SLSA L3+ 独立検証の結果を記載する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 外部監査人 | artifact リスト・builder リスト受領 | — |
| T+15m | 外部監査人 | cosign verify-attestation 実行開始 | — |
| T+2h | 外部監査人 | サンプリング検証完了 | CLI: 「Verified OK: builder matches expected identity」 |
| T+2h30m | 外部監査人 | chain-of-custody 記録・監査 Draft 記載 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| SCADA テレメトリ収集 | 高 | SCADA 処理コンポーネントの SLSA L3+ 準拠が安全性の前提 |
| FA 生産指示 | 中 | 生産指示処理コンポーネントの supply chain integrity が生産記録の信頼性に影響 |

## 独立性宣言

外部監査人は本シナリオにおいても独立性宣言を確認済みであること。cosign verify は公開鍵で実施するため、内部関係者の介入なく独立検証が可能であることを確認している。

## 証拠保全 chain-of-custody

| 証拠番号 | 証拠種別 | 取得日時 | 取得方法 | 保管場所 | 完全性確認 |
|---------|---------|---------|---------|---------|-----------|
| EVD-040 | cosign verify-attestation 出力 | 検証日時を記録 | CLI 出力 | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |
| EVD-041 | 許容 builder identity リスト | 受領日時を記録 | security 担当者から受領 | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |

## 監査人側 tool セット

```bash
# SLSA L3+ provenance 署名検証
cosign verify-attestation \
  --type slsaprovenance \
  --certificate-identity-regexp "https://github.com/k1s0/.*" \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  ghcr.io/k1s0/k1s0-server@sha256:<DIGEST>

# 出力例（期待される結果）:
# Verification for ghcr.io/k1s0/k1s0-server@sha256:<DIGEST>
# The following checks were performed on each of these signatures:
#   - The cosign claims were validated
#   - Existence of the claims in the transparency log was verified offline
# {"payloadType":"application/vnd.in-toto+json","payload":"..."}
```

## 関連適合仕様

- [OSSライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)

## 期待結果 / 観測指標 / 受入条件

- 全サンプルの provenance 署名が有効で builder identity が許容リストと一致していること
- Rekor 記録との照合が完了していること
- 受入条件: 外部監査人が SLSA L3+ 準拠の独立証明を監査報告書に記載できること

## 失敗時の挙動 / escalation

- **builder identity が許容リストに含まれない**: 許可されていない builder からの artifact が存在することを示す。security 担当者に書面で通知し、当該 artifact の使用を停止するよう勧告する
- **provenance 署名検証失敗**: security 担当者に書面で通知し、監査手続きを一時中断する

## 失敗パターン（3 例）

1. **builder identity の確認をスキップして署名の有効性だけ確認する**: 署名が有効でも許可されていない builder から build された artifact である可能性が残る
2. **SLSA L3 と L2 の違いを理解せずに「SLSA 準拠」と報告する**: L2 は hermetic build を要求しないため L3 より保護強度が低い。レベルを明示して報告する
3. **04 の Rekor 検証と本シナリオを別個に実施せず一方だけ行う**: 独立した二重検証経路を確保することで証明強度が増す。両シナリオを必ず実施する

## 関連参照

- [外部監査人シナリオ INDEX](README.md)
- [04_Sigstore_Rekor_transparency_log検証](04_Sigstore_Rekor_transparency_log検証.md)
- [10_監査報告書発行_証跡保全](10_監査報告書発行_証跡保全.md)
