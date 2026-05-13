---
id: plan.overview.scenario_external_auditor_audit_hash_chain
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

# audit_hash_chain独立検証

## 一文方針

pgaudit 出力と DB row の hash 連鎖を独立 reconstruct して改竄不在を確認する。

> 年次監査の初日、外部監査人は監査対象テナントの audit hash chain の独立検証から作業を開始する。k1s0 から提供された audit log export（pgaudit 出力 + DB row hash 連鎖データ）を自身のラップトップに取得し、hash 連鎖を最初のレコードから最終レコードまで独立に reconstruct する。全 row の hash が連鎖して一致することを確認し、chain-of-custody ドキュメントに検証結果を記録する。

## ペルソナ要約

主役: 外部監査人（公認会計士 / 監査法法人スタッフ）、目的: audit hash chain の改竄不在を内部関係者とは独立に技術的に証明する

## 現状業務での痛み

- 監査対象のシステムから提供された audit ログが本当に改竄されていないかを独立に検証する手段がなく、提供者の主張を信頼するしかない
- hash 連鎖の再計算を手作業で行う必要があり、大量レコードの検証に数日を要することがある
- 提供された audit ログと実際の DB に保存されたデータの整合性を確認する手順が標準化されていない
- 監査証跡が複数フォーマット（CSV / DB dump / ログファイル）に分散しており、統合検証が困難

## k1s0 でこう変わる

- pgaudit 出力と DB row の hash 連鎖が標準フォーマットで export されるため、独立 reconstruct の手順が確立している
- hash 連鎖の計算方式（アルゴリズム / seed / 連鎖方法）が公開ドキュメントとして提供されており、外部監査人が独立に再計算できる
- audit_ingest_gap_monitor が連続性を保証しており、ギャップの有無が export 時に明示される
- 外部監査人専用の読み取り専用アクセスが短期 credential で提供されるため、独立性を損なわない形での DB 参照が可能

## Trigger

監査契約開始時 / 年次監査実施時（監査契約に基づく年次実施スケジュールをトリガーとする）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次 1〜2 回
- 典型きっかけ: 「年次 ISO 9001 外部審査の初日に audit chain 検証を実施する」「financial audit で操作履歴の改竄不在証明が必要になった」
- 頻度根拠: 外部監査は年次 1〜2 回が標準

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 外部監査人（主役） | 外部専門家 | 監査現場 / リモート | 自身の CLI 環境 | audit log 取得・hash 再計算・検証結果記録 |
| 業務管理者 | シニア | 事務所 | Backstage プラグイン | audit export の提供・質問対応 |
| security 担当者 | シニア | 開発拠点 | セキュリティダッシュボード | 必要時の技術的説明 |

## 個人 KPI / 達成感

- hash 連鎖の reconstruct 完了率（全対象レコードを検証できること）
- 検証完了から chain-of-custody 記録完了までの所要時間（目標: 1 営業日以内）
- hash 不一致が発見された場合の escalation 報告所要時間（目標: 2 時間以内）
- 監査報告書に独立検証の結果を明記できたこと

## 工数 / 関与人数 / コスト感

- 標準規模テナント（10 万件以下のレコード）: 2〜4 時間
- 大規模テナント（100 万件超）: 半日〜1 日
- hash 不一致発見時: +半日〜1 日（詳細調査）
- 関与人数: 1〜2 名（外部監査人のみ、または補助者 1 名）

## 前提

- 外部監査人の短期 credential が発行されており、audit export API への読み取りアクセスが可能
- k1s0 の hash 連鎖計算方式（アルゴリズム / seed / 連鎖フォーマット）が公開ドキュメントとして提供されている
- 外部監査人の環境に Python / Node.js などの hash 計算が可能なランタイムが準備されている
- 独立性宣言が監査契約開始時に提出されている

## 流れ

1. 独立性宣言を確認し、外部監査人専用の短期 credential で Backstage 外部監査人ポータルにアクセスする
2. 対象テナント・対象期間の audit log export（pgaudit 出力 + hash 連鎖データ）をダウンロードする
3. export ファイルを自身のラップトップに保存し、chain-of-custody ログに取得日時・ファイルハッシュを記録する
4. k1s0 の公開ドキュメントに記載された hash 連鎖計算方式を参照し、独立に hash 連鎖を reconstruct するスクリプトを実行する
5. 全レコードの hash が連鎖して一致することを確認する
6. audit_ingest_gap_monitor の gap 有無レポートを確認する
7. 検証結果（一致 / 不一致 / gap 有無）を chain-of-custody ドキュメントに記録する
8. 検証結果を監査報告書の Draft に記載する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 外部監査人 | 独立性確認・credential でポータルログイン | — |
| T+10m | 外部監査人 | audit log export ダウンロード | — |
| T+15m | 外部監査人 | chain-of-custody に取得記録 | — |
| T+20m | 外部監査人 | hash 連鎖 reconstruct スクリプト実行 | — |
| T+2h | 外部監査人 | hash 連鎖検証完了（全レコード一致確認） | — |
| T+2h30m | 外部監査人 | gap レポート確認 | — |
| T+3h | 外部監査人 | chain-of-custody 記録・監査 Draft 記載 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 品質検査結果配信 | 高 | 品質検査記録の audit chain が ISO 9001 / GMP 監査の主要対象 |
| FA 生産指示 | 中 | 生産指示承認の audit chain が内部統制監査の対象 |
| 警報配信 | 中 | 警報対応記録の audit chain が安全管理記録として監査対象 |

## 独立性宣言

外部監査人は本シナリオ開始前に以下を確認・宣言する。

- 監査対象テナントの内部業務との利害関係を持たないこと
- 取得した audit 証跡を監査目的以外に使用しないこと
- 監査期間終了後に短期 credential を即座に返却 / 無効化に同意すること
- 独立検証の結果を改変せずに報告書に記載すること

## 証拠保全 chain-of-custody

外部監査人が取得する証拠の保全記録。

| 証拠番号 | 証拠種別 | 取得日時 | 取得方法 | 保管場所 | 完全性確認 |
|---------|---------|---------|---------|---------|-----------|
| EVD-001 | audit log export (CSV) | 取得日時を記録 | Backstage 外部監査人ポータル | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |
| EVD-002 | hash 連鎖検証結果 | 検証完了日時 | 独立スクリプト実行出力 | 監査人ラップトップ（暗号化） | スクリプト出力の hash を記録 |
| EVD-003 | gap レポート | 取得日時 | Backstage 外部監査人ポータル | 監査人ラップトップ（暗号化） | SHA-256 hash を記録 |

## 監査人側 tool セット

```bash
# hash 連鎖の independent reconstruct（例: SHA-256 を使用した連鎖検証）
python3 verify_hash_chain.py \
  --input audit_export.csv \
  --algorithm sha256 \
  --chain-doc k1s0_hash_chain_spec.md

# 検証結果の出力例
# Verifying row 1/100000... OK
# Verifying row 2/100000... OK
# ...
# All 100000 rows verified. Chain intact.
```

## 関連適合仕様

- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/11_データ保全適合仕様.md)
- audit_ingest_gap_monitor

## 期待結果 / 観測指標 / 受入条件

- 全対象レコードの hash 連鎖が一致している（改竄なし）
- gap レポートで対象期間の連続性が確認できている
- chain-of-custody ドキュメントに取得日時・ファイル hash・検証結果が記録されている
- 受入条件: 外部監査人が「改竄不在」を独立に確認できたことを監査報告書に記載できること

## 失敗時の挙動 / escalation

- **hash 不一致が検出された**: 外部監査人は即座に業務管理者と security 担当者に書面で通知する（SLA: 2 時間以内）。監査手続きを一時中断し、data 担当者 + security 担当者が調査に着手する
- **gap が検出された**: gap の範囲と原因を audit_ingest_gap_monitor のレポートで確認する。gap が意図的でない場合は security インシデントとして扱う
- **credential アクセス拒否**: 業務管理者に credential 再発行を依頼する

## 失敗パターン（3 例）

1. **公開ドキュメントを参照せず独自の hash 計算方式で検証する**: 連鎖方式の差異で「不一致」が誤検出される。必ず k1s0 の公開ドキュメントに記載された計算方式を使用する
2. **ダウンロードしたファイルの完全性を確認せずに検証を開始する**: ダウンロード中の bit error が「hash 不一致」として誤検出される。取得直後に SHA-256 hash を記録して完全性を確認する
3. **検証結果を chain-of-custody に記録せずに口頭で報告する**: 監査報告書に証拠能力のある形で記載できなくなる。検証完了と同時に chain-of-custody ドキュメントに記録することを必須ステップとする

## 関連参照

- [外部監査人シナリオ INDEX](README.md)
- [03_RFC3161_trusted_timestamp検証](03_RFC3161_trusted_timestamp検証.md)
- [10_監査報告書発行_証跡保全](10_監査報告書発行_証跡保全.md)
- `arch.security.security_index`
