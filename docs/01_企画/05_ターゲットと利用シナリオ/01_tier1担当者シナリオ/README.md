---
id: plan.tier1.scenario_index
axis: tier1
phase: plan
kind: index
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# tier1 担当者シナリオ index

## 一文方針

tier1 担当者（シニア級）が日常的に踏む 11 シナリオを 1 ファイル 1 シナリオで列挙する。Server 系 5 系統（Gateway / Sidecar+Agent / Backend-for-Library / Control Plane / Operator+Controller）/ Library 3 抽象レベル × 17 機能カテゴリ / Companion / 主要 9 適合仕様（Bidi / 移行 Pair / 観測 / 認証 / 鍵管理 / スキーマ進化 / SLO / OSS ライフサイクル / テナント容量）を tier1 facade として維持する責務を担う。

## 担当者プロフィール

- 級: シニア
- 想定人数: 5-8 名
- 必須スキル: Rust（主言語）/ Go（Operator）/ C# / TypeScript / proto / Buf / OSS lifecycle

詳細は [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md) を参照。

## 本シナリオ群での用語定義

- **dual reviewer**: tier1 担当者 2 名（変更 PR の author 不可、同一人物不可）の組合せ。全 PR に必須。
- **想定頻度の解釈**: 1 名の tier1 担当者が担当する頻度の目安。実際は担当 OSS / カテゴリ数に比例する。
- **supply chain 担当者**: security 担当者のうち supply chain（cosign / SBOM / Harbor mirror）を担当するチームメンバー。Mattermost `#security-incident` または `#supply-chain-incident` で対応。

tier1 担当者は以下の責務を横断的に担う:

- Server 系コンポーネント（Gateway / Sidecar / Agent / Backend-for-Library / Control Plane / Operator / Controller）の設計・実装・維持
- Library の 3 抽象レベル（L3 OSS 中立 / L2* 同族保証 / L1+ 単一深耕）と 17 機能カテゴリ × 4 言語（Rust / C# / Go / TypeScript）の facade 維持
- Companion（.NET Framework 4.8 ERP / レガシー対応）の役割 A・B 管理
- 適合仕様 9 本（スキーマ進化 / OSSライフサイクル / HTTP2 enforcement / supply chain 等）の番人

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|-----------|---------|---------|-----------------|------|
| 01 | [新規OSS採用評価](01_新規OSS採用評価.md) | 新 OSS 採用 / 移行提案が挙がった時 | 四半期〜年次 | OSSライフサイクル適合仕様 / tier1強制機構 | [計画] |
| 02 | [OSSライフサイクルイベント対応](02_OSSライフサイクルイベント対応.md) | L1+ OSS のライセンス変更 / 改廃 / EOL 発生時 | イベント駆動（年次 dry-run は年次） | OSSライフサイクル適合仕様 | [緊急]+[周期] |
| 03 | [proto二層化スキーマ進化](03_proto二層化スキーマ進化.md) | External / Internal Proto の互換確保が必要になった時 | 週次〜月次（External Proto は四半期） | スキーマ進化適合仕様 / apicurio_gitops_sot | [計画] |
| 04 | [Library新言語_新カテゴリ追加](04_Library新言語_新カテゴリ追加.md) | 新言語 or 新機能カテゴリの Library 追加要求が来た時 | 年次〜不定期 | 検証規律適合仕様 | [計画] |
| 05 | [Server系コンポーネント追加](05_Server系コンポーネント追加.md) | Server 系新コンポーネント追加要求が来た時 | 月次〜四半期 | SLO 適合仕様 / tier1強制機構 | [計画] |
| 06 | [Companion役割AB拡張](06_Companion役割AB拡張.md) | レガシー ERP の Companion 要件追加時 | 年次〜不定期 | HTTP2 enforcement / Bidi 適合仕様 | [計画] |
| 07 | [Transport切替_bidi移行](07_Transport切替_bidi移行.md) | HTTP/2 → HTTP/3 / WebTransport 追従が必要になった時 | 年次〜不定期（標準化に追従） | HTTP2 enforcement / Bidi 適合仕様 | [計画] |
| 08 | [公開API_snapshot違反対応](08_公開API_snapshot違反対応.md) | CI の公開 API snapshot 差分検査が fail した時 | イベント駆動（月数件） | 検証規律適合仕様 | [緊急] |
| 09 | [supply_chain障害対応](09_supply_chain障害対応.md) | Harbor / cosign / SBOM / supply chain lint で障害・違反が検出された時 | イベント駆動（障害発生時） | OSSライフサイクル適合仕様 / tier1強制機構 | [緊急] |
| 10 | [SBOM_CVE_月次トリアージ](10_SBOM_CVE_月次トリアージ.md) | 月次 SBOM review cadence 到来または重大 CVE 公開時 | 月次 + イベント駆動 | OSS ライフサイクル適合仕様 | [周期]+[緊急] |
| 11 | [Library_release切り](11_Library_release切り.md) | Library に十分な変更が蓄積し release milestone 達成時 | 月次〜四半期 | 検証規律適合仕様 | [計画] |

## 重要用語早見表

| 用語 | 定義 |
|---|---|
| L1+ | 単一 OSS に深耕し移行 toolchain を継続維持（[Library 方針](../../../03_概要設計/02_tier1設計方針/02_Library.md)） |
| L2* | 同族 OSS 2 実装で Testcontainers conformance test green を merge 条件 |
| L3 | OSS 中立（概念のみ公開し実装を交換可能に保つ） |
| Server 系 5 系統 | Gateway / Sidecar+Agent / Backend-for-Library / Control Plane / Operator+Controller |
| Transport Adapter Layer | bidi semantics を transport 非依存で実現する tier1 内部層 |
| 公開 API snapshot | 各言語の公開型・関数シグネチャの記録。差分 CI で regression を検出 |

## 新規参画者向けオンボーディング

シニア級エンジニアとして着任した際の推奨学習順序:

- **Day 1-3**: README 全体読了 → 01（OSS 採用評価）→ 02（OSS lifecycle）の一文方針と流れを通読
- **Day 4-7**: 03（proto スキーマ進化）→ 08（API snapshot 違反対応）をペア担当者と共に演習
- **Week 2**: 09（supply chain）→ 10（SBOM CVE トリアージ）で実際の月次 review を担当
- **Week 3-4**: 04（Library 追加）→ 05（Server 系追加）→ 11（Library release）を担当 PR として実施
- **Month 2 以降**: 06（Companion）→ 07（Transport 切替）は発生時に担当。01-02 の判断ツリーは月 1 回の OSS review 会で活用

## シナリオ間の依存関係

- **01（OSS 採用）→ 02（lifecycle 対応）**: 採用した L1+ OSS が EoL を迎えると 02 が発火。採用判断は 01、移行実施は 02。
- **02（lifecycle）→ 11（release 切り）**: OSS 移行完了後に新バージョンの Library release を 11 で実施。
- **10（SBOM CVE トリアージ）→ 11（release 切り）**: CVSS 9.0 以上の未対応 CVE がある場合は 11 の release を保留（11 の「前提」参照）。
- **03（proto 進化）→ 04（Library 追加）**: proto スキーマ変更が先行し、4 言語 Library の codegen 成功後に 11 で release を切る順序。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [ターゲットと利用シナリオ index](../README.md)
