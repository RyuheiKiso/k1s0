# 01. ADR 運用プロセス設計

本ファイルは k1s0 における Architecture Decision Record（ADR）の起票・レビュー・承認・改訂・廃止の運用プロセスを実装フェーズ確定版として確定する。90 章方針の IMP-POL-POL-002（全技術決定の ADR 化）と IMP-POL-POL-007（緊急例外の 72 時間事後 ADR）を、`docs/02_構想設計/adr/` の物理配置、ADR 接頭辞体系、CODEOWNERS 強制、MADR テンプレート、関係性リンク（Supersedes / Superseded-by / Related-to）で具体化する。

![ADR ライフサイクル状態遷移](img/adr_ライフサイクル状態遷移.svg)

10 年保守サイクルにおいて「なぜこの技術を選んだのか」が辿れなくなる瞬間、コードは負債へと転落する。3 年後の担当者が Redis の代わりに Valkey が採用されている背景を知らず、うっかり Redis に戻せば BSL 問題が再発する。ADR は「未来の自分 / 後任者への置き手紙」であり、合議が口伝で消えないための唯一の仕組みである。本節はその起票・レビューが面倒すぎず、かつ抜け漏れも生じないラインを設計する。

崩れると、ADR プロセスが形骸化して「書いた人が決めた」という状態が生まれ、結果として再び同じ議論が半年おきに発生する。緊急パッチ時に ADR を迂回するルートを恒久化すれば、平常時の ADR プロセスも相対的に無視されるようになり、統制全体が沈黙のうちに解体される。

## Phase 確定範囲

- Phase 1a: ADR テンプレート確定、接頭辞体系、CODEOWNERS 強制、20 本規模で定常運用
- Phase 1b: D-ADR（事後 ADR）プロセス確立、Radar との相互リンク自動化
- Phase 1c: ADR と IMP-* ID の双方向リンク整合チェック自動化、廃止 ADR の参照検出

## ADR 接頭辞体系

ADR ID は接頭辞 + 3 桁で構成する（IMP-POL-ADR-020）。接頭辞は領域ごとに固定し、採番は領域内で連番とする。採番重複を防ぐため、`docs/02_構想設計/adr/README.md` に予約済 ID 一覧を維持する。

- `ADR-0XXX`: 全体横断（ADR-0001=アーキテクチャ基本、ADR-0002=drawio 図解レイヤ、ADR-0003=AGPL 分離）
- `ADR-TIER1-XXX`: tier1 関連（ADR-TIER1-001=Go+Rust ハイブリッド、ADR-TIER1-002=Protobuf gRPC）
- `ADR-CICD-XXX`: CI/CD（ADR-CICD-003=Kyverno）
- `ADR-OBS-XXX`: 観測性（ADR-OBS-001=Grafana LGTM）
- `ADR-SEC-XXX`: セキュリティ / Identity
- `ADR-DATA-XXX`: データ基盤（ADR-DATA-001=CloudNativePG）
- `ADR-DEP-XXX`: 依存管理（ADR-DEP-001=Renovate 中心運用）
- `ADR-POL-XXX`: ガバナンス / ポリシー（ADR-POL-001=Kyverno 二分所有）
- `ADR-SUP-XXX`: サプライチェーン（ADR-SUP-001=SLSA L2→L3）
- `ADR-MIG-XXX`: 移行（ADR-MIG-001=.NET Framework ラップ）
- `ADR-DIR-XXX`: ディレクトリ設計（ADR-DIR-001〜003）
- `D-ADR-XXX`: Deferred ADR（緊急パッチ後 72 時間以内の事後起票、接頭辞と連番独立）

接頭辞追加は ADR-0001（メタ ADR）の改訂で行う。領域が曖昧な場合は `ADR-0XXX` で採番し、後に領域 ADR への改名は禁止する（ID 不変原則、IMP-POL-ADR-021）。

## MADR テンプレートと必須項目

ADR のフォーマットは MADR（Markdown Any Decision Records）v3.0 をベースとし、k1s0 固有項目を追加した `docs/02_構想設計/adr/template.md` を使用する（IMP-POL-ADR-022）。必須項目は以下で、欠落した PR は CI で fail させる。

- **Title**: 「ADR-XXX-YYY: <決定事項の 1 文要約>」形式
- **Status**: `Proposed` / `Accepted` / `Deprecated` / `Superseded by ADR-XXX`
- **Date**: 起票日 + 最終改訂日
- **Context**: 背景 / 現状の問題 / この決定を迫られた理由
- **Decision**: 決定事項（1-3 行の簡潔な記述）
- **Consequences**: 帰結 / トレードオフ（Positive / Negative / Neutral に分けて記述）
- **Alternatives**: 検討した代替案 + 不採用理由
- **Relates-to / Supersedes / Superseded-by**: 他 ADR への相互リンク
- **対応 IMP-\* ID**: 本 ADR が生む実装 ID の列挙（双方向リンク）

相互リンクは `Supersedes: ADR-XXX` / `Superseded-by: ADR-YYY` / `Related-to: [ADR-AAA, ADR-BBB]` の 3 種類のみ認め、自由形式の言及を避ける（機械パース可能性のため）。

## CODEOWNERS 強制と ADR なし merge 拒否

技術的に重要な変更（OSS 追加・アーキテクチャ変更・ポリシー追加・廃止判断）に ADR なしで merge される事態を防ぐため、CODEOWNERS と PR template で二重に検出する（IMP-POL-ADR-023）。

```
# .github/CODEOWNERS 抜粋
/src/tier1/rust/Cargo.toml  @k1s0/security-team @k1s0/platform-team
/src/tier1/go/go.mod        @k1s0/security-team @k1s0/platform-team
/deploy/kyverno/validate/   @k1s0/security-team
/docs/02_構想設計/adr/      @k1s0/architecture-team
```

PR template には「この PR は新規 OSS / アーキテクチャ変更 / policy を含むか」のチェックボックスを置き、Yes の場合は対応 ADR の PR 番号を記入する欄を必須化する。GitHub Actions の `verify-adr-reference.yml` が該当ファイル変更を検知したら ADR 参照の存在を確認し、未記入の場合は PR を fail させる。

ADR 自体の merge には architecture-team（Security + SRE + DX の 3 名で構成）のうち 2 名以上の approve が必須となる。1 名の独断で technical decision が通る経路を構造的に塞ぐ。

## 事後 ADR（D-ADR）と 72 時間ルール

緊急インシデント対応・事故復旧等で事前 ADR プロセスを迂回した変更は、72 時間以内に `D-ADR-XXX` として事後起票する（IMP-POL-ADR-024）。`D-ADR-` 接頭辞は通常の ADR と別連番で管理し、事後追認された変更が可視化される。

- 72 時間以内の追認: architecture-team の 1 名以上の approve で「追認」、D-ADR は `Status: Accepted (deferred)` となる
- 72 時間超過: インシデントとして扱い、NFR-E-SIR-001（インシデント対応 Runbook）に則った事後対応
- 追認されなかった D-ADR: 1 週間以内に変更を revert、技術負債として DX メトリクス（95 章連動）に計上
- 半期レビュー: D-ADR の起票傾向を Technology Radar（30 節）更新時に分析、恒常的に発生するパターンは平常 ADR プロセス改善対象

D-ADR の必須項目は通常 ADR と同じだが、追加で「**Emergency Context**（なぜ事前 ADR を迂回せざるを得なかったか）」と「**Root Cause Analysis**（緊急対応を要した根本原因）」を記述する。これにより「緊急と称した実質平常の抜け道」を監視可能にする。

## ADR のライフサイクル状態遷移

ADR は `Proposed` → `Accepted` → `Deprecated` / `Superseded` の状態を持つ（IMP-POL-ADR-025）。各状態の定義を以下で固定する。

- `Proposed`: 提案された段階、architecture-team レビュー中。merge 時点では本状態
- `Accepted`: 2 名以上の approve で merge された後、本状態に更新。IMP-* 実装が追従する
- `Deprecated`: 決定事項自体が不要となった（例: 廃止された機能に関する ADR）。理由を追記して残す
- `Superseded`: 後継 ADR で置き換えられた。`Superseded-by: ADR-YYY` リンクで系譜を保持

ADR は **削除しない**（IMP-POL-ADR-026）。情報価値は「今現在の正解」だけでなく「過去の試行錯誤」にある。Deprecated / Superseded 状態の ADR も検索性を維持し、将来の担当者が「なぜこの選択肢は却下されたか」を追跡できる状態を守る。

## Technology Radar との相互リンク

ADR と Technology Radar（30 節）は双方向リンクで結合する（IMP-POL-ADR-027）。Radar の各エントリ（Adopt / Trial / Assess / Hold）は対応 ADR を参照し、ADR は採用判断時の Radar 配置を記録する。

- Adopt 昇格: 該当 ADR の `Consequences` に「Radar: Trial → Adopt 昇格（YYYY-MM-DD）」を追記
- Hold 降格: 該当 ADR に `Superseded-by` または `Deprecated` を明示、Radar と同時更新
- 新規採用判断: ADR 起票時に Radar 上の現在位置（Assess / Trial）を `Context` に記載

Radar 半期更新時には全 ADR を逆引きし、Adopt 昇格・Hold 降格候補を architecture-team + Security + SRE + DX の合議で判定する。

## 受け入れ基準

- `docs/02_構想設計/adr/template.md` が MADR v3.0 + k1s0 拡張で確定、必須項目 9 個を満たす
- 接頭辞体系 12 種が `docs/02_構想設計/adr/README.md` に予約済 ID 一覧として維持
- CODEOWNERS で architecture-team が ADR merge に 2 名 approve を強制
- 新規 OSS / アーキテクチャ変更 PR が対応 ADR なしでは CI fail する
- D-ADR プロセスが稼働し、72 時間以内追認率 / revert 率が DX メトリクスに計上
- Technology Radar との双方向リンクが半期更新で維持されている

## RACI

| 役割 | 責務 |
|---|---|
| Architecture team（Security + SRE + DX 3 名、共同 A） | ADR レビュー、2 名以上 approve、メタ ADR（ADR-0XXX）の更新 |
| 起票者（提案者） | ADR 下書き、Alternatives 列挙、Consequences 記述 |
| Security（D） | ADR-SEC / ADR-POL / ADR-SUP の主担当承認 |
| Platform/SRE（B） | ADR-CICD / ADR-OBS / ADR-DATA の主担当承認 |
| DX（C） | ADR 導線の開発者体験、テンプレート改善、Radar 連動 |

## 対応 IMP-POL-ADR ID

| ID | 主題 | Phase |
|---|---|---|
| IMP-POL-ADR-020 | ADR 接頭辞体系 12 種の固定と予約 ID 一覧維持 | 1a |
| IMP-POL-ADR-021 | ID 不変原則（接頭辞変更の改名禁止） | 1a |
| IMP-POL-ADR-022 | MADR v3.0 + k1s0 拡張テンプレートと必須項目 9 個 | 1a |
| IMP-POL-ADR-023 | CODEOWNERS 強制と ADR なし merge 拒否の CI ゲート | 1a |
| IMP-POL-ADR-024 | D-ADR 72 時間ルールと追認プロセス | 1b |
| IMP-POL-ADR-025 | ライフサイクル 4 状態（Proposed / Accepted / Deprecated / Superseded） | 1a |
| IMP-POL-ADR-026 | ADR 削除禁止、系譜保持による試行錯誤の可視化 | 1a |
| IMP-POL-ADR-027 | Technology Radar との双方向リンク運用 | 1b |

## 対応 ADR / DS-SW-COMP / NFR

- [ADR-0001](../../../02_構想設計/adr/ADR-0001-architecture-basics.md)（アーキテクチャ基本）/ [ADR-0002](../../../02_構想設計/adr/ADR-0002-diagram-layer-convention.md)（図解レイヤ規約）/ [ADR-CICD-003](../../../02_構想設計/adr/ADR-CICD-003-kyverno.md)（Kyverno）
- DS-SW-COMP: 全体横断（特定 ID なし）
- NFR-C-MGMT-001（設定 Git 管理）/ NFR-C-MGMT-002（Flag/Decision Git 管理）/ NFR-H-AUD-001（監査ログ完整性）/ NFR-C-ENV-002（運用ドキュメント鮮度）

## 関連章

- `10_Kyverno_Policy/` — Kyverno policy 追加時の ADR 起票要件
- `30_Technology_Radar/` — Radar との双方向リンク運用
- `40_脅威モデル_STRIDE/` — STRIDE 改訂時の ADR 起票
- `../95_DXメトリクス設計/` — D-ADR 起票率・追認率のメトリクス
