---
id: plan.overview.scenario_external_auditor_index
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.target_use_case
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E, F]
  proof_classes: []
---

# 外部監査人シナリオ INDEX

## 一文方針

外部監査人（公認会計士 / 監査法人 / 業界規制監査機関）が内部関係者でない独立性を維持しながら、audit chain と外部公証で改竄不可能性を独立検証する 10 シナリオを 1 ファイル 1 シナリオで列挙する。pgaudit hash chain / RFC 3161 / Sigstore Rekor / WORM Object Lock / SLSA L3+ / cosign verify / KEK shamir ceremony が本軸の核心技術である。

## 担当者プロフィール

| 属性 | 内容 |
|------|------|
| 職種 | 公認会計士 / 監査法人スタッフ / 業界規制監査機関の審査員 |
| IT スキル | 業界規制監査経験 + audit hash chain / RFC 3161 / Sigstore Rekor / WORM Object Lock / SLSA L3+ / cosign verify / openssl ts / rekor-cli |
| 業務知識 | ISO 9001 / GMP / HACCP / ISO 14001 などの業界規制監査 |
| 想定人数 | 監査契約期間中 1〜3 名（年次 1〜2 回） |
| 主要デバイス | ラップトップ PC（監査人自身の機器）|
| UI 接点 | Backstage の外部監査人ポータル（読み取り専用ダッシュボード）+ CLI ツール（rekor-cli / openssl / cosign） |
| 認証手段 | 外部監査人専用の短期 credential（監査契約期間限定）|
| 責務 | 内部関係者でない独立性を維持しながら、audit chain と外部公証で改竄不可能性を独立検証 |

## 本シナリオ群の特徴と注意点

- **外部監査人は内部関係者（業務管理者 / tier担当者）とは完全に独立した役割。** 監査の独立性を担保するため、外部監査人は証跡の閲覧のみ可能であり、データの変更・削除権限を一切持たない。
- 本シナリオは「外部監査人の目線」で記述する。k1s0 内部実装の記述は最小限にとどめ、独立検証の手順・証拠保全の方法・監査人側の tool セットに集中する。
- 外部監査人は自身の tool セット（rekor-cli / openssl / cosign）で独立に検証を実施することが前提であり、k1s0 の UI に依存しない検証経路が確保されている。
- chain-of-custody 形式の証拠保全が全シナリオを貫く原則であり、監査人が取得した証跡は改竄不可能な形式で監査人自身の管理下に置かれる。

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連技術 | 種別 |
|---|-----------|---------|---------|--------------|------|
| 01 | [audit_hash_chain独立検証](01_audit_hash_chain独立検証.md) | 監査契約開始時 / 年次監査実施時 | 年次 1〜2 回 | pgaudit hash chain | [計画] |
| 02 | [WORM_Object_Lock検証](02_WORM_Object_Lock検証.md) | WORM 保存期間中の物理書込不可能性確認時 | 年次 | WORM Object Lock | [計画] |
| 03 | [RFC3161_trusted_timestamp検証](03_RFC3161_trusted_timestamp検証.md) | タイムスタンプの信頼性独立検証時 | 年次 | RFC 3161 TSA | [計画] |
| 04 | [Sigstore_Rekor_transparency_log検証](04_Sigstore_Rekor_transparency_log検証.md) | build artifact / audit checkpoint の inclusion proof 検証時 | 年次 | Sigstore Rekor | [計画] |
| 05 | [SLSA_L3_provenance独立検証](05_SLSA_L3_provenance独立検証.md) | SLSA L3+ provenance の独立検証時 | 年次 | cosign + Rekor | [計画] |
| 06 | [KEK_shamir_ceremony観察](06_KEK_shamir_ceremony観察.md) | KEK shamir M-of-N ceremony 実施時 | 年次〜不定期 | KEK shamir M-of-N | [計画] |
| 07 | [ISO9001_GMP_HACCP_業界規制監査](07_ISO9001_GMP_HACCP_業界規制監査.md) | ISO 9001 / GMP / HACCP / ISO 14001 の外部審査時 | 年次 | audit hash chain + RFC 3161 | [計画] |
| 08 | [商用PenTest結果_独立review](08_商用PenTest結果_独立review.md) | 商用 PenTest 報告書が提出された時 | 年次〜半年次 | PenTest 結果独立 review | [計画] |
| 09 | [data_breach発生時の独立調査](09_data_breach発生時の独立調査.md) | data breach 発生 / 疑義発生時 | イベント駆動 | audit chain 再構成 | [緊急] |
| 10 | [監査報告書発行_証跡保全](10_監査報告書発行_証跡保全.md) | 監査手続き完了時 | 年次 1〜2 回 | chain-of-custody | [計画] |

## 外部監査人の独立性原則

外部監査人は以下の原則を厳守する。

| 原則 | 具体的な制約 |
|------|------------|
| 読み取り専用アクセス | k1s0 のいかなるデータも変更・削除できない |
| 独立検証経路 | Backstage UI に依存せず CLI ツールで独立に検証できる |
| 証跡の独立管理 | 取得した証跡は監査人自身のストレージに chain-of-custody 形式で保管する |
| 独立性宣言 | 監査契約開始時に利害関係不在の独立性宣言を提出する |
| 監査人 credential の短期性 | 外部監査人専用 credential は監査契約期間のみ有効 |

## 外部監査人の tool セット

外部監査人が使用する CLI ツールの概要。

| ツール | 用途 |
|--------|------|
| rekor-cli | Sigstore Rekor transparency log の inclusion proof 検証 |
| openssl ts | RFC 3161 タイムスタンプの署名検証 |
| cosign verify | SLSA L3+ provenance / supply chain artifact の署名検証 |
| pgaudit / psql | audit hash chain の DB row 参照・hash 再計算 |
| aws s3api / azure storage | WORM Object Lock の retention policy 確認 |

## 新規参画者向けオンボーディング

1. まず 01 (audit_hash_chain 独立検証) を読み、k1s0 の監査証跡の基本構造を把握する。
2. 次に 03 (RFC 3161 タイムスタンプ検証) で改竄不在の証明方法を確認する。
3. 04 (Sigstore Rekor 検証) と 05 (SLSA L3+ 独立検証) で supply chain の独立検証手順を習得する。
4. 07 (業界規制監査) で製造業固有の監査証跡の構造を確認する。
5. 09 (data breach 独立調査) で緊急時の audit chain 再構成手順を確認する。
6. 10 (監査報告書発行_証跡保全) で chain-of-custody 形式の証拠保全手順を確立する。

## シナリオ間の依存関係

```
01 (audit_hash_chain 独立検証)
  ├─► 03 (RFC 3161 タイムスタンプ検証) — hash chain の時刻証明を独立検証
  ├─► 07 (業界規制監査) — audit chain を業界規制チェックリストにマッピング
  └─► 09 (data breach 独立調査) — breach 発生時に audit chain を再構成

02 (WORM Object Lock 検証)
  └─► 10 (監査報告書発行_証跡保全) — WORM 検証結果を監査報告書に添付

04 (Sigstore Rekor 検証)
  └─► 05 (SLSA L3+ provenance 独立検証) — Rekor で artifact を検証してから provenance を検証

06 (KEK shamir ceremony 観察)
  └─► 10 (監査報告書発行_証跡保全) — ceremony 観察記録を監査報告書に添付

08 (商用 PenTest 結果 独立 review)
  └─► 10 (監査報告書発行_証跡保全) — PenTest review 結果を監査報告書に添付
```

## 関連参照

- `docs/01_企画/05_ターゲットと利用シナリオ/08_業務管理者シナリオ/` — 業務管理者が準備した証跡を外部監査人が独立検証する
- `docs/01_企画/05_ターゲットと利用シナリオ/README.md` — ターゲット・利用シナリオ全体 index
- `arch.security.security_index` — セキュリティアーキテクチャ index
- `req.team.tier_engineer_requirement` — tier 担当者要件
