---
id: plan.tier1.scenario_public_api_snapshot_violation
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

# 公開 API snapshot 違反対応

## 一文方針

CI の公開 API snapshot 差分検査 fail を起点に、意図的変更と regression を即時に判別し、regression はrevert・意図的変更は SemVer 方針決定と backward 互換確認を経て `public_api_snapshot.lock.yaml` を更新し dual reviewer sign-off を取得する。

## Trigger（発火条件）

CI の公開 API snapshot 差分検査が fail した時（意図しない API 表面変更）。

## 想定頻度 / 典型きっかけ

- 想定頻度: イベント駆動（月数件、CI fail 発生時）
- 典型きっかけ: 「tier2 担当者が tier1 Library を直接 import する PR を出し snapshot 差分 CI が fail した」「Go の exported symbol が意図せず外部 package から見える状態で commit された」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ 変更 PR 作成者（tier2 担当者の場合あり）

## 前提

- `public_api_snapshot.lock.yaml` が公開 API surface の管理ファイルとして存在する
- CI の公開 API snapshot 差分検査が全 PR に対して実行されている
- 4 言語（Rust / C# / Go / TypeScript）に対して独立した snapshot が管理されている
- tier2 / tier3 のコードが公開 API に直接依存しており、破壊的変更が下流に影響することが前提

## 流れ

1. **差分検査 fail レポートの分析**: CI の snapshot 差分検査 fail レポートから以下を特定する。
   - どの言語（Rust / C# / Go / TypeScript）の API 表面が変わったか
   - どのカテゴリ（17 カテゴリのうちどれ）の API か
   - 変更の種別: 追加（新シンボル）/ 変更（シグネチャ変更）/ 削除（既存シンボル削除）
   - 変更のあった PR とコミットの特定

2. **意図的 vs 意図外の判定**: 変更 PR の内容とレビューコメントを確認し意図を判定する。
   - 意図外（regression）: PR の本来の変更意図と無関係に API が変わっている場合。手順 3 へ。
   - 意図的（新機能追加 / breaking change）: PR の変更目的として API 変更が含まれる場合。手順 4 へ。

3. **regression の対処**:
   - 変更を revert し、API 表面を元の状態に戻す
   - root cause を特定する: Rust の pub スコープ漏洩 / C# の internal が public に昇格 / Go の exported symbol の意図外追加 / TypeScript の型 re-export の漏洩
   - 強制機構（visibility linter）の設定を強化し同種の regression を防ぐ
   - 修正後、snapshot 差分検査 green を確認してから PR を再提出する

4. **意図的変更の処理**:
   - SemVer 方針を決定する:
     - 追加のみ（backward 互換）: minor bump
     - シグネチャ変更 / 削除（backward 非互換）: major bump
     - 内部実装変更（API surface 不変）: patch bump
   - `public_api_snapshot.lock.yaml` の更新範囲（言語 / カテゴリ）を明確にする

5. **backward 互換性の検証（意図的変更の場合）**: 4 言語の backward 互換性を Testcontainers 結合テストで検証する。
   - tier2 / tier3 の既存 import が変更前の API で壊れないことを確認
   - major bump の場合: 旧バージョンの API surface も一定期間維持し deprecation notice を付ける
   - minor bump の場合: 追加 API が既存コードに干渉しないことを確認

6. **`public_api_snapshot.lock.yaml` 更新と dual reviewer sign-off**: 確定した API surface を `public_api_snapshot.lock.yaml` に反映し、dual reviewer（tier1 2 名）が sign-off する。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [検証規律適合仕様](../../../04_詳細設計/01_適合仕様/19_検証規律適合仕様.md)
- [スキーマ進化適合仕様](../../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md)

**関連 OSS**:
- cargo-semver-checks（Rust）: SemVer backward 互換性の静的検証
- Microsoft.CodeAnalysis（C#）: Roslyn を用いた API 表面の静的解析
- apidiff（Go）: Go パッケージの API 互換性チェック
- api-extractor（TypeScript）: TypeScript の公開 API surface の管理と差分検出

## 期待結果 / 観測指標

- CI の公開 API snapshot 差分検査が green（意図した変更のみを反映）
- `public_api_snapshot.lock.yaml` が最新の公開 API surface を反映している
- regression の場合: revert 後に root cause が特定され強制機構が強化されている
- 意図的変更の場合: SemVer 方針が決定され backward 互換性が Testcontainers で検証されている
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- tier2 / tier3 の既存 import が破壊されていない

## 失敗時の挙動 / escalation

- **snapshot 不一致のまま merge 試行**: CI merge gate が阻止。snapshot が一致するまで PR は merge 不可。escalate 先: tier1 担当者 dual reviewer（SLA: 4 時間以内に方針決定）。Backstage runbook `api-snapshot-mismatch` を参照。
- **regression の root cause 特定困難**: 変更 PR の diff を言語ごとに精査し、visibility 設定の変更を探す。tier1 内でのペアレビューを実施する。escalate 先: tier1 担当者 dual reviewer（SLA: 8 時間以内に root cause 特定）。
- **backward 非互換変更の下流影響が広範**: major bump を決定し、旧バージョン互換 API を deprecation 期間（最低 1 マイナーバージョン）維持する。tier2 / tier3 担当者に移行計画を通知する。escalate 先: tier1 担当者 dual reviewer + 影響 tier2/tier3 担当者（SLA: 24 時間以内に移行計画通知）。
- **4 言語間で SemVer 方針が一致しない**: 最も厳しい方針（最大 bump）に統一する。言語ごとの independent versioning は 1.0.0 では採用しない。escalate 先: tier1 担当者 dual reviewer（SLA: 5 営業日以内に方針統一）。

## 関連参照

- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
- [Library 新言語 / 新カテゴリ追加](04_Library新言語_新カテゴリ追加.md)
- [proto 二層化スキーマ進化](03_proto二層化スキーマ進化.md)
