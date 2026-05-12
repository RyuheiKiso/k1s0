---
id: plan.overview.scenario_external_auditor_sigstore_rekor
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

# Sigstore_Rekor_transparency_log検証

## 一文方針

build artifact / audit checkpoint の Rekor inclusion proof を独立検証する。

> 年次監査の 3 日目、外部監査人は k1s0 の build artifact と audit checkpoint が Sigstore Rekor の transparency log に記録されており、事後的に追加されていないことを独立確認する。rekor-cli で inclusion proof を取得し、Rekor の公開ログに対して merkle proof を検証する。audit checkpoint の Rekor ログエントリを確認し、chain-of-custody に記録する。

## ペルソナ要約

主役: 外部監査人（公認会計士 / 監査法人スタッフ）、目的: build artifact と audit checkpoint が Sigstore Rekor の公開 transparency log に正しく記録されており、遡及的な追加 / 改竄がないことを独立確認する

## 現状業務での痛み

- supply chain の artifact が本当に改竄されていないかを内部関係者以外の視点で確認する手段がない
- transparency log の概念を理解した監査人が少なく、Rekor による証明の意味を監査報告書に正確に記載できない
- inclusion proof の検証手順が専門的すぎて監査チームに習得コストがかかる
- audit checkpoint の Rekor 記録と実際の audit log の一致を確認する手順が存在しない

## k1s0 でこう変わる

- build artifact と audit checkpoint が Rekor public log に記録されており、誰でも inclusion proof を検証できる
- rekor-cli による標準的な検証手順が公開ドキュメントに記載されており、外部監査人が手順通りに実施できる
- merkle proof の検証が CLI で自動化されており、手作業での計算が不要
- audit checkpoint の Rekor ログエントリ UUID が公開されているため、外部監査人が独立にアクセスできる

## Trigger

build artifact / audit checkpoint の inclusion proof 検証時（年次監査手続きの supply chain 検証フェーズで実施）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次
- 典型きっかけ: 「年次 supply chain 監査で k1s0 の build artifact の integrity 確認が必要」「audit checkpoint が改竄なく Rekor に記録されていることを規制当局に証明する」

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 外部監査人（主役） | 外部専門家 | 監査現場 / リモート | CLI 環境（rekor-cli） | inclusion proof 取得・merkle proof 検証 |
| security 担当者 | シニア | 開発拠点 | セキュリティダッシュボード | Rekor ログエントリ UUID の提供 |

## 個人 KPI / 達成感

- 対象 artifact / checkpoint の inclusion proof 検証完了（サンプリング）
- merkle proof の独立検証完了
- chain-of-custody への記録完了

## 工数 / 関与人数 / コスト感

- 標準的な検証（サンプリング 20〜50 件）: 2〜4 時間
- 関与人数: 1 名（外部監査人のみ）

## 前提

- 対象 artifact / checkpoint の Rekor ログエントリ UUID が提供されている
- 外部監査人の環境に rekor-cli が準備されている
- Rekor public instance（rekor.sigstore.dev）へのネットワークアクセスが可能

## 流れ

1. security 担当者から対象 artifact / checkpoint の Rekor ログエントリ UUID リストを受け取る
2. rekor-cli で各エントリの inclusion proof を取得する
3. merkle proof の検証が「Verified OK」であることを確認する
4. audit checkpoint エントリの内容と k1s0 から提供された audit hash 値が一致していることを確認する
5. 検証結果を chain-of-custody ドキュメントに記録する
6. 監査報告書に Rekor 独立検証の結果を記載する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 外部監査人 | UUID リスト受領・rekor-cli 設定 | — |
| T+15m | 外部監査人 | inclusion proof 取得・merkle proof 検証開始 | — |
| T+2h | 外部監査人 | サンプリング検証完了 | CLI: 「Verified OK for all 50 entries」 |
| T+2h30m | 外部監査人 | chain-of-custody 記録・監査 Draft 記載 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| SCADA テレメトリ収集 | 中 | SCADA 処理コンポーネントの build artifact integrity が安全性の前提 |
| 計量装置連続データ | 中 | 計量処理 artifact の integrity が計量記録の信頼性に影響する |

## 独立性宣言

外部監査人は本シナリオにおいても独立性宣言を確認済みであること。Rekor は公開 transparency log であるため、UUID さえ知れば誰でも独立に検証可能であることを確認している。

## 証拠保全 chain-of-custody

| 証拠番号 | 証拠種別 | 取得日時 | 取得方法 | 保管場所 | 完全性確認 |
|---------|---------|---------|---------|---------|-----------|
| EVD-030 | Rekor inclusion proof (JSON) | 取得日時を記録 | rekor-cli get | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |
| EVD-031 | merkle proof 検証結果ログ | 検証日時を記録 | CLI 出力 | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |

## 監査人側 tool セット

```bash
# Rekor inclusion proof 取得と merkle proof 検証
rekor-cli get \
  --rekor_server https://rekor.sigstore.dev \
  --uuid <UUID_FROM_K1S0_DOCS> \
  --format json > rekor_entry.json

rekor-cli verify \
  --rekor_server https://rekor.sigstore.dev \
  --uuid <UUID_FROM_K1S0_DOCS>

# 出力例:
# Current Root Hash: <tree_hash>
# Entry Hash: <entry_hash>
# Verified OK
```

## 関連適合仕様

- [OSSライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)

## 期待結果 / 観測指標 / 受入条件

- 全サンプルの merkle proof が「Verified OK」であること
- audit checkpoint の Rekor エントリ内容と提供された hash 値が一致していること
- 受入条件: 外部監査人が Rekor 独立検証の結果を監査報告書に記載できること

## 失敗時の挙動 / escalation

- **merkle proof 検証失敗**: 外部監査人は security 担当者に書面で通知し、原因調査を依頼する
- **UUID が Rekor に存在しない**: artifact が Rekor に記録されていないことを示す。security 担当者に原因確認を依頼する

## 失敗パターン（3 例）

1. **UUID を提供者からのみ受け取り独立確認をしない**: 存在しない UUID を提供された場合に気づかない。rekor-cli で実際に存在することを独立確認する
2. **merkle proof 検証をスキップして「inclusion proof あり」とだけ報告する**: inclusion proof の存在確認と merkle proof による証明は別の手順であり、両方を実施する
3. **Rekor の公開性（誰でも検証可能）を理解せずに「特別なアクセスが必要」と誤解する**: 公開 transparency log は UUID さえあれば誰でも検証可能であることを理解する

## 関連参照

- [外部監査人シナリオ INDEX](README.md)
- [05_SLSA_L3_provenance独立検証](05_SLSA_L3_provenance独立検証.md)
- [10_監査報告書発行_証跡保全](10_監査報告書発行_証跡保全.md)
