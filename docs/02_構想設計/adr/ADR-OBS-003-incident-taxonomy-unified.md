# ADR-OBS-003: 可用性・セキュリティインシデントを単一分類体系で統合管理

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: SRE / セキュリティチーム / 運用チーム / 法務部 / 契約レビュー担当 / Product Council

## コンテキスト

k1s0 の運用ライフサイクルは、Runbook 15 本（[docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md](../../04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) 参照）をカテゴリ横断で運用する前提だが、「インシデントをどう分類するか」の基盤定義をリリース時点で確定する必要がある。

業界慣例では可用性インシデント（availability incident）とセキュリティインシデント（security incident）を別系統で管理する組織が多い。ITIL の Incident Management と、ISO/IEC 27035 の Information Security Incident Management は歴史的背景が異なるため、SRE とセキュリティチームで分類体系が分岐しがちである。採用側組織の情報システム部門は J-SOX・個人情報保護法・経済安全保障推進法を横断で見るため、「可用性事象がセキュリティ事象の前兆だった」「セキュリティパッチ適用が可用性劣化を誘発した」という境界事象の取りこぼしが重大な監査指摘に直結する。

採用側の運用が小規模から拡大する過程で、以下の運用劣化パターンが具体的に予見される。

- **分類盲点**: DDoS 攻撃は「可用性劣化」として SRE が対応するか「セキュリティ事象」として CSIRT が対応するかが分岐し、連絡経路・証跡保管・対外通告の要否判定に遅延が生じる
- **SLO 不整合**: 可用性系の SLO（RTO / RPO）とセキュリティ系の SLA（CVSS 9.0+ は 48 時間以内に緩和、等）が別々に運用され、同一の実インシデントに対して複数の時計が走る
- **Runbook 網羅性の欠落**: Runbook 15 本は各個撃破で整備されているが、分類体系に紐付かないため「このカテゴリに該当する Runbook が 1 本も存在しない」盲点を機械検知できない
- **エラーバジェット連動の欠如**: セキュリティ事象の頻度と深刻度がエラーバジェット消費と無関係に積み上がり、事業継続判断の全体像が欠ける

一方、CVSS v4.0 は「脆弱性の深刻度」を数値化するが、可用性事象（非脆弱性起因の停止）を直接扱わない。ITIL の優先度マトリクス（影響度 × 緊急度）は可用性事象向けで、CVSS との機械的接続が弱い。いずれか片方だけでは k1s0 の横断運用に不足する。

本 ADR は可用性インシデントとセキュリティインシデントを単一の分類体系（Incident Taxonomy）で統合管理し、CVSS 値を分類軸の一部として取り込む。深刻度クラスごとに SLO を明示し、Runbook 網羅性を本分類の各カテゴリに対して少なくとも 1 本が対応することをリリース時点で保証する。

## 決定

**k1s0 では、可用性インシデントとセキュリティインシデントを単一の Incident Taxonomy で管理する。**

### 分類体系の 2 軸

1. **Category 軸**: 事象の性質による分類
   - `availability`: 可用性劣化（停止・性能劣化・容量超過）
   - `integrity`: データ改ざん・データ損失・整合性破壊
   - `confidentiality`: 情報漏洩・認可回避
   - `supply-chain`: ビルドサプライチェーン事件（SLSA 関連）
   - `compliance`: 規制違反・監査指摘
   - `operational`: 運用手順の逸脱（人為ミス・変更管理違反）

2. **Severity 軸**: 深刻度クラス
   - `sev1` (Critical): CVSS 9.0+ の脆弱性、またはサービス全停止、または顧客データ漏洩確認
   - `sev2` (High): CVSS 7.0〜8.9、または主要機能停止、または顧客データ漏洩の疑い
   - `sev3` (Medium): CVSS 4.0〜6.9、または部分機能劣化、または SLO 違反に至らない揺らぎ
   - `sev4` (Low): CVSS 0.1〜3.9、または監視ノイズ、または内部改善事項

可用性事象でも CVSS 参照が可能な場合は CVSS を副次情報として記録する（例: Kubernetes API の CVE を原因とする停止）。可用性のみに起因する事象は CVSS なしで Severity 軸のみで判定する。

### Severity クラスごとの SLO

- **sev1**: 検知から 48 時間以内に緩和または撤退完了。エラーバジェットの残量にかかわらず最優先、他作業は中断
- **sev2**: 検知から 7 日以内に緩和または撤退完了。エラーバジェットの月次残量の 50% 以上を消費する場合は sev1 相当に格上げ検討
- **sev3**: 検知から 30 日以内に緩和または計画された Roadmap に組込。エラーバジェット連動で月次レビューにて優先度調整
- **sev4**: 四半期の改善バックログに計上。エラーバジェット連動対象外

いずれも「緩和」は Workaround を含み、「撤退」は根本原因除去を意味する。sev1/sev2 は Workaround で SLO を満たしつつ、別途根本解を 90 日以内にクローズする。

### Runbook 網羅性保証

Runbook 15 本（[09_Runbook目録方式.md](../../04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) 参照）を本分類の 6 カテゴリ × 4 severity = 24 セルに割り当てる。各 Category の sev1 / sev2 セルには少なくとも 1 本の Runbook が対応することをリリース時点で保証する。sev3 / sev4 は「該当時に Runbook 化検討」の扱い。

対応表は以下を既定とする（k1s0 リリース時点の状態）。

- `availability × sev1`: RB-API-001, RB-DB-001, RB-DB-002, RB-MSG-001, RB-DR-001
- `availability × sev2`: RB-NET-001, RB-NET-002, RB-BKP-001
- `integrity × sev1`: RB-SEC-003, RB-AUD-001
- `integrity × sev2`: RB-BKP-001
- `confidentiality × sev1`: RB-SEC-004
- `confidentiality × sev2`: RB-SEC-001, RB-AUTH-001, RB-SEC-002
- `supply-chain × sev1`: RB-SEC-005（[ADR-SUP-001](ADR-SUP-001-slsa-staged-adoption.md) に基づく image hash 逆引き）
- `supply-chain × sev2`: Renovate fallback Runbook（[ADR-DEP-001](ADR-DEP-001-renovate-central.md)）
- `compliance × sev1`: 未整備、リリース時点で RB-COMP-001 起票
- `compliance × sev2`: RB-AUD-001
- `operational × sev1`: RB-OPS-001
- `operational × sev2`: RB-OPS-002

`compliance × sev1` の Runbook 未整備がリリース後の優先改善項目として明示される。

### エラーバジェット連動

- sev1 / sev2 の発生頻度をエラーバジェット月次消費として集計
- セキュリティ起因の sev1 は可用性起因の sev1 と同じバジェットを消費する（単一バジェット制）
- バジェット枯渇時は新規機能開発を停止し、信頼性回復作業に切り替える基準を [NFR-A 系] の SLO ドキュメントに明記

### 記録フォーマット

全インシデントは `ops/incidents/YYYY/MM/INC-XXXX.md` として記録する。メタデータ必須項目は以下。

- `id`: `INC-XXXX`
- `category`: 6 カテゴリのいずれか
- `severity`: sev1〜sev4
- `cvss_base`: 任意、CVSS v4.0 ベーススコア
- `cvss_vector`: 任意、CVSS ベクタ文字列
- `detected_at` / `mitigated_at` / `resolved_at`: 各タイムスタンプ
- `slo_met`: boolean、SLO を満たしたか
- `runbook_used`: 使用した Runbook ID（該当なしの場合は null）
- `blameless_postmortem`: sev1 / sev2 は必須、sev3 以下は推奨

## 検討した選択肢

### 選択肢 A: 統合 Incident Taxonomy（採用）

- 概要: 可用性とセキュリティを 1 つの分類体系で管理、Category × Severity の 2 軸
- メリット:
  - 境界事象（DDoS / セキュリティパッチ副作用 / 供給側改ざん）の取りこぼしを構造的に排除
  - SLO 時計が事象ごとに単一化、エラーバジェットの月次集計が整合
  - Runbook 網羅性を 6×4 マトリクスで機械検証可能
  - CVSS 値と可用性判断を同一レコード内で並記でき、監査対応が容易
  - 採用側の運用が小規模でも拡大期でも同一運用が可能、拡大による再設計が不要
- デメリット:
  - 初期の Category / Severity 判定に学習コスト（Incident Commander 研修が必要）
  - セキュリティチームと SRE の慣習を統一する文化的摩擦が初期に発生
  - 既存の業界テンプレート（ITIL / ISO 27035 それぞれ単独）とは異なるため外部参照時に翻訳が必要

### 選択肢 B: 可用性とセキュリティを別系統で管理

- 概要: SRE は可用性事象、CSIRT はセキュリティ事象を各々の分類で管理
- メリット: 各チームの慣習を維持、初期移行コストゼロ
- デメリット:
  - 境界事象で連絡経路・所有権の対立が発生、対応遅延を誘発
  - SLO 時計が複数走り、エラーバジェット集計が不整合
  - Runbook 網羅性の機械検証が不可能
  - 10 年保守の途中で必ず「どっちが見るのか」問題が発現

### 選択肢 C: CVSS のみで統合

- 概要: 全事象を CVSS ベクタで表現、CVSS 値で severity を決定
- メリット: 業界標準（CVSS v4.0）に準拠、外部文献との整合
- デメリット:
  - 可用性起因の事象（非脆弱性起因の停止）を CVSS で表現することが不自然
  - CVSS の attack vector / attack complexity 等の軸は純粋可用性事象に無意味
  - 可用性事象の影響分析が CVSS 枠では十分にできない

### 選択肢 D: ITIL 分類のみ（影響度 × 緊急度マトリクス）

- 概要: ITIL の P1〜P5 優先度マトリクスで全事象を管理
- メリット: 採用側組織の情シス部門の馴染みのある分類
- デメリット:
  - CVSS 連動が不在、セキュリティ事象の深刻度判定が主観的になる
  - 脆弱性起因の事象をセキュリティ研究者の外部指標（GHSA / OSV）と突合できない
  - ISO 27035 系の監査で不利

## 帰結

### ポジティブな帰結

- 境界事象の取りこぼしを構造的に排除、J-SOX 監査・個人情報保護法対応での説明可能性が担保
- Runbook 網羅性が 6×4 マトリクスで機械検証され、`compliance × sev1` のような盲点がリリース時点で可視化
- エラーバジェット連動が単一時計で動き、Product Council の意思決定（機能開発 vs 信頼性回復）が明確
- CVSS 値と可用性判断が同一レコードに並び、監査対応の説明負荷が最小
- インシデントレコードの一貫性により、10 年保守で蓄積するインシデントデータから傾向分析・FMEA 見直しが可能

### ネガティブな帰結

- Incident Commander（SRE / セキュリティ兼任）への分類判断研修が必要
- 既存の ITIL / ISO 27035 テンプレを直輸入できないため、初期の分類ガイドライン整備コストが発生
- Category 判定が分岐する境界事象（例: DDoS は availability か confidentiality の前兆か）で議論コストが発生、判断ログを残す規律が必要
- `compliance × sev1` の Runbook 未整備が明示的な負債として残り、リリース時点で RB-COMP-001 の整備が必須

### 移行・対応事項

- `docs/04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md` に本分類体系を追記、6×4 マトリクスと SLO クラスを明記
- Runbook 目録（`09_Runbook目録方式.md`）の各 Runbook メタデータに `category` / `severity` タグを追加
- `ops/incidents/` ディレクトリ構造と `INC-XXXX.md` テンプレートを `docs/00_format/` 配下に整備
- Grafana ダッシュボード（[ADR-OBS-001](ADR-OBS-001-grafana-lgtm.md)）に Incident Severity 別の月次件数・SLO 達成率パネルを追加
- Prometheus AlertManager ルールに `severity` ラベルを必須化し、severity に応じた通知チャネル（sev1→PagerDuty、sev2→Slack、sev3/4→Email）へ振り分け
- リリース時点で RB-COMP-001（コンプライアンス違反対応 Runbook）を新規起票、`compliance × sev1` の Runbook 空白を埋める
- Blameless Postmortem テンプレを `docs/00_format/` 配下に整備、sev1 / sev2 の事後レビュー工程を BC-GOV-005 に組込み
- [ADR-SUP-001](ADR-SUP-001-slsa-staged-adoption.md) の RB-SEC-005 を `supply-chain × sev1` カテゴリに明示的に紐付け

## 参考資料

- [ADR-OBS-001: 観測性に Grafana LGTM スタックを採用](ADR-OBS-001-grafana-lgtm.md)
- [ADR-OBS-002: OpenTelemetry Collector 採用](ADR-OBS-002-otel-collector.md)
- [ADR-SUP-001: SLSA Level 2 達成と Level 3 到達計画](ADR-SUP-001-slsa-staged-adoption.md)
- [ADR-DEP-001: 依存更新中枢に Renovate を採用](ADR-DEP-001-renovate-central.md)
- [ADR-REL-001: Progressive Delivery 必須化](ADR-REL-001-progressive-delivery-required.md)
- [Runbook 目録方式](../../04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md)
- [インシデント対応方式](../../04_概要設計/55_運用ライフサイクル方式設計/02_インシデント対応方式.md)
- [CLAUDE.md](../../../CLAUDE.md)
- CVSS v4.0 仕様: [first.org/cvss/v4.0](https://www.first.org/cvss/v4-0)
- ITIL 4 Incident Management プラクティス
- ISO/IEC 27035:2023 Information Security Incident Management
- Google SRE Book "Managing Incidents" 章 / SRE Workbook "Incident Response"
- [ADR-TEST-006: 観測性 E2E を 5 検証で構造化](ADR-TEST-006-observability-e2e.md) — SLO 分類軸が SLO burn rate alert 発火検証 (検証 4) で機械検証される
- PagerDuty Incident Response Documentation
- NIST SP 800-61r2 Computer Security Incident Handling Guide
