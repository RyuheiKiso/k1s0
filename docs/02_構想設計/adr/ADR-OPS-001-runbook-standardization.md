# ADR-OPS-001: Runbook を 8 セクション + YAML frontmatter + Chaos Drill で標準化する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: SRE / 運用チーム / 採用検討組織 / 起案者

## コンテキスト

k1s0 は「採用側の小規模運用で成立する」（NFR-C-NOP-001）ことを企画コミットの中核に据えており、起案者が不在の夜間・休日に協力者が単独で SEV1（応答 SLA 15 分 / RTO 4 時間、NFR-A-CONT-001）に対応できることを **バス係数 2** の実現条件としている。判断要素を最小化した機械的手順 = Runbook がなければ、夜間休日対応は属人化し、起案者の単一障害点化を避けられない。

NFR-A-REC-002 は「リリース時点で Runbook 15 本整備」をコミット値として持ち、`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md` で形式・粒度・検証方法を DS-OPS-RB-001〜009 として設計済である。一方、これらの **設計判断（Markdown + Backstage TechDocs + 8 セクション固定 + YAML frontmatter + Chaos Drill）** は ADR として未起票で、なぜ他形式を採らなかったかの判断根拠が docs に残っていなかった（2026-05-02 #6 の docs-orphan 監査で `ADR-OPS-001` の cite が `docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md` から発見されたが ADR ファイル不在）。

Runbook 形式の選定は採用検討組織が「10 年保守する」前提で評価する **two-way door に近いが移行コストが高い** 決定である。形式を後から変えると、既存 Runbook の全書き換え + TechDocs ビルド設定 + Alert manager の `runbook_url` ラベル整合 + Chaos Drill シナリオの再構築が必要になる。リリース時点で形式を確定させ、採用組織が世代交代しても保守できる構造を残す。

形式選定では以下を満たす必要がある:

- **Git 管理可能**（PR レビュー / コードレビューと同じ動線、バージョン履歴を一次情報源にできる）
- **読み手の認知負荷が低い**（夜間休日に緊張下で読むため、慣れた位置に情報を見つけられる順序）
- **陳腐化を構造的に防ぐ**（書きっぱなしで放置されない仕組み）
- **採用組織の運用エンジニアが普通に学ぶスキルで保守できる**（独自 DSL / ツールに閉じない）
- **Alert / Backstage / 自動化基盤と接続可能**（運用プロセス基盤 NFR-C-OPS-001 と整合）

## 決定

**Runbook は Markdown 形式で `k1s0/runbooks` Git リポジトリに格納し、Backstage TechDocs で公開する。すべての Runbook は以下を必須とする。**

1. **8 セクション固定**（順序固定、省略禁止、該当なしの場合は「該当なし」を明記）
   1. 前提条件（権限・ツール・環境）
   2. 対象事象（アラート発火条件 / 観測症状）
   3. 初動手順（5 分以内）
   4. 原因特定手順
   5. 復旧手順（一次復旧 + 恒久復旧、各ステップ 3 分以内、合計 1 時間以内目標）
   6. 検証手順（具体的確認項目 + Grafana / Loki クエリ）
   7. 予防策（再発防止 + Jira チケット化）
   8. 関連 Runbook
2. **YAML frontmatter 必須**: `runbook_id / title / category / severity / owner / automation / alertmanager_rule / fmea_id / estimated_recovery / last_updated`
3. **命名規則**: `RB-<カテゴリ>-<通番>-<簡潔名>.md`、カテゴリは `API / DB / NET / SEC / OPS / AUTH / MSG / WF / COMP / INC / AUD / DR` 等
4. **粒度**: 1 Runbook = 1 事象 / 1 手順ステップ ≤ 3 分 / 1 Runbook 全体 ≤ 1 時間
5. **検証**: 四半期に 1 回 Chaos Drill（計画的 / 抜き打ち / 自動化の 3 パターン併用）で復旧可能性を実機検証
6. **Alert 連動**: Alertmanager 各ルールに `runbook_url` ラベル必須、未整備は `TBD`、TBD 10 件超で Product Council 見直し
7. **品質指標**: カバー率（≥ 90%）、実行成功率（≥ 80%）、所要時間、更新頻度を Grafana で月次計測

物理配置は `ops/runbooks/{incidents,daily,weekly,monthly,postmortems,templates}/` とし、テンプレート `ops/runbooks/templates/runbook-template.md` を雛形として運用する。

## 検討した選択肢

### 選択肢 A: Markdown + Backstage TechDocs + 8 セクション固定 + YAML frontmatter + Chaos Drill（採用）

- 概要: Git で Markdown を管理、Backstage TechDocs で公開、8 セクションの順序を固定、YAML frontmatter で機械可読 metadata を強制、四半期 Chaos Drill で陳腐化を防ぐ
- メリット:
  - PR レビュー / 履歴 / 検索 / 静的サイト生成が Git エコシステムで完結（追加ツール不要）
  - 8 セクション固定で読み手の認知負荷が下がり、夜間休日でも情報を機械的に拾える
  - YAML frontmatter で `automation: argo-workflow|manual` / `severity: sev1|sev2` 等を集計可能（カバー率の機械計測が成立）
  - Chaos Drill で「書いた手順で本当に復旧できるか」を構造的に検証、書きっぱなし放置を防ぐ
  - Backstage TechDocs は ADR-BS-001 で採用済、追加ツール選定が不要
- デメリット:
  - 8 セクション必須化で軽微な定期運用 Runbook も同フォーマットを強要され、初期作成コストが上がる
  - YAML frontmatter の整合性チェック（`alertmanager_rule` が実在するか等）に CI 整備が要る
  - Chaos Drill のシナリオ作成 + staging 環境維持コストが恒常的にかかる

### 選択肢 B: Confluence / Notion（SaaS Wiki）

- 概要: 商用 SaaS Wiki に Runbook を集約、WYSIWYG 編集、組織単位のアクセス制御
- メリット:
  - 非エンジニアでも編集しやすい WYSIWYG
  - SaaS なので運用基盤を自社で持たなくてよい
  - コメント / メンション機能でディスカッションが Wiki 内で完結
- デメリット:
  - **Git 管理外**: PR レビュー動線・履歴一元化・テキスト検索が IDE / `grep` で不可能
  - **アラート連動が弱い**: Alertmanager の `runbook_url` で深いリンクは可能だが、変更履歴 / レビュー / IaC との整合は別系統になる
  - 商用 SaaS なのでオンプレ完結要件（NFR-F-SYS-001）に整合しない採用組織が選択不能
  - ベンダーロックイン、エクスポート品質はベンダー次第

### 選択肢 C: 自由形式（起案者判断に委ねる）

- 概要: Runbook の形式・粒度・必須セクションを規定せず、起案者が事象に応じて判断
- メリット:
  - 起案者の判断で軽い Runbook はコンパクトに、重い Runbook は厚く書ける（柔軟性）
  - 初期作成コストが最小、規約整備の前置き作業が不要
- デメリット:
  - **読み手の認知負荷が爆発**: Runbook ごとにセクション順序 / 粒度 / 用語が異なり、夜間休日の緊張下で読み解けない
  - 起案者ごとの暗黙ルールが乱立し、レビュー観点が固定化できない（PR レビューが属人化）
  - 機械処理（カバー率計測 / `runbook_url` 自動 lint / metadata 集計）が成立しない
  - **バス係数 2 が成立しない**: 起案者 / 協力者の経験則に依存し、新規メンバーが読み解けない

### 選択肢 D: ITIL 準拠の重厚 Runbook（既存企業標準）

- 概要: ITIL v4 の Service Operation プロセスに準拠、Incident Management / Problem Management / Change Management の 3 分類で重厚な Runbook を作成、変更管理委員会（CAB）で承認
- メリット:
  - ITIL は世界標準で、採用組織のエンタープライズ部門が既に運用ノウハウを持つ可能性
  - プロセス分離が明確（Incident / Problem / Change）
  - 監査対応 / コンプライアンス報告に流用しやすい
- デメリット:
  - **小規模運用前提と矛盾**: ITIL は大規模 IT 組織前提のフレームワークで、CAB 等の重い承認プロセスが NFR-C-NOP-001 の小規模運用と衝突
  - 1 Runbook あたりのテンプレートが厚く、3 分粒度 / 1 時間制約で機械的に実行する SEV1 初動と相性が悪い
  - Markdown 親和性が低く、Confluence / 専用 ITSM ツールに引き寄せられがち
  - 採用組織が「k1s0 採用 = ITIL 採用」と誤認すると採用ハードルが急上昇

## 決定理由

選択肢 A を採用する根拠は以下。

- **バス係数 2 の構造的担保**: 8 セクション固定 + 3 分粒度 + Markdown + Git の組み合わせは、夜間休日に新規協力者が単独で SEV1 対応できる前提条件をすべて満たす。選択肢 C（自由形式）は経験則依存で属人化、選択肢 D（ITIL）は重厚すぎて初動 5 分制約に合わない
- **既存 ADR との整合**: ADR-BS-001（Backstage 採用）/ ADR-OBS-001（Grafana LGTM）/ ADR-OBS-003（インシデント分類体系）/ ADR-CICD-001（Argo CD GitOps）と組み合わさることで、Alert → Runbook → 自動化（ArgoWorkflows / Temporal）→ ポストモーテムの一気通貫フローが成立する。選択肢 B（Confluence）は ADR-BS-001 と二重管理で SoT が割れる
- **オンプレ完結要件との整合**: 選択肢 B（SaaS Wiki）は NFR-F-SYS-001（オンプレ完結）違反で採用組織により選択不能。選択肢 A は Backstage を自前ホストする限りオンプレで完結
- **陳腐化防止の構造化**: Chaos Drill 四半期実施は「書いた手順で本当に復旧できるか」を実機で検証する仕組みで、選択肢 C / D には欠ける構造的なフィードバックループ。Runbook が古くなる前に必ず修正 PR が起こる
- **機械処理可能性**: YAML frontmatter + 8 セクション固定により、カバー率 90%（`alertmanager_rule` ↔ `runbook_id` の左外結合）/ 実行成功率（`automation: argo-workflow` の成功 metric）/ 所要時間（`estimated_recovery` と Grafana 実測の比較）を機械集計できる。選択肢 C はこれが成立しない
- **採用組織のスキル流用性**: Markdown / YAML / Git / PR レビューは世間で標準的に学ぶスキルで、採用組織の世代交代後も保守できる。ITIL（D）の独自プロセス学習コストを回避

## 影響

### ポジティブな影響

- バス係数 2 が構造的に担保され、起案者不在でも夜間休日 SEV1 対応が成立する
- 8 セクション固定で読み手の認知負荷が下がり、新規協力者のオンボーディング教材として機能（2 週間で SEV2 単独対応レベル到達）
- Chaos Drill で陳腐化が構造防止され、書きっぱなしの形骸 Runbook を発生させない
- Alert → Runbook → 自動化のフローが GitOps 一元で版管理され、Argo CD と整合
- Runbook カバー率 / 実行成功率 / 所要時間 / 更新頻度を Grafana で機械計測でき、品質を主観に頼らず可視化できる
- Backstage TechDocs / OpenBao SOPS-AGE 鍵 / PagerDuty escalation / Jira ticketing の 5 ツール統合（DS-OPS-RB-014）と素直に接続する

### ネガティブな影響 / リスク

- 8 セクション必須化で、軽微な定期運用 Runbook（毎朝 health check 等）も同フォーマット強要となり、初期作成コスト + レビュー時間が増える
- YAML frontmatter の整合性 lint（`alertmanager_rule` 実在 / `fmea_id` 実在 / `last_updated` 6 ヶ月以内）は別途 CI 整備が必要で、`tools/lint/runbook-frontmatter.sh` 等の実装が要る
- Chaos Drill の staging 環境維持 + シナリオ作成は四半期ごとに恒常的工数を要し、SRE のキャパシティ計画に組み込む必要
- 採用組織が ITIL 既存運用と衝突する場合、k1s0 Runbook と組織標準 Runbook の 2 系統並走となる移行期間が発生
- 「該当なし」明記の運用が緩むと、見せかけの 8 セクション充足になり質を担保できない（PR レビューでの厳格運用が要）

### 移行・対応事項

- `ops/runbooks/templates/runbook-template.md` を 8 セクション + YAML frontmatter の雛形として整備（DS-OPS-RB-002）
- リリース時点で 15 本（NFR-A-REC-002 のコミット値）の Runbook を整備、`docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md` の目録に対応
- Alertmanager の全ルールに `runbook_url` ラベル付与を強制（未整備は `TBD`、TBD 10 件超で Product Council escalation）
- Chaos Drill 四半期計画を `ops/chaos/workflows/` に配置（採用ツールは ADR-OPS-002 で別途決定）
- Runbook 品質指標 4 種を Grafana ダッシュボード `runbook-quality.json` として `infra/observability/grafana/dashboards/` に配置
- 新規協力者オンボーディング 2 週間プロセス（1 週目: 全 Runbook 読破 + 改善 PR / 2 週目: staging で 3 本実演）を `docs/40_運用ライフサイクル/` に文書化
- 既存 docs の `ADR-OPS-001` cite（`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md:278`）が本 ADR を指すことを `tests/audit/test_audit_lib.sh` Test 19 で監視（docs-orphan watchlist から本 ID を外す）

## 参考資料

- `docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md` — 設計項目 DS-OPS-RB-001〜009 の詳細
- `docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md` — 物理配置と Chaos / DR との接続
- ADR-BS-001（Backstage 開発者ポータル）— TechDocs の採用根拠
- ADR-OBS-001（Grafana LGTM）— 監視基盤と品質指標計測
- ADR-OBS-003（インシデント分類体系）— Severity / Category 軸との整合
- ADR-CICD-001（Argo CD GitOps）— Runbook 自動化の配信経路
- NFR-A-REC-002 / NFR-A-CONT-001 / NFR-C-NOP-001 / NFR-C-OPS-001 — 関連要件
- 関連 ADR（採用検討中）: ADR-OPS-002（Chaos Engineering ツール選定、別 PR で起票）
- ADR-TEST-004（LitmusChaos 採用）— Chaos Drill 四半期実施で使用するツール
- ADR-TEST-005（Upgrade drill + DR drill）— 四半期 Chaos Drill とローテーション枠を共有
