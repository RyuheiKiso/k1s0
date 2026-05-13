---
id: plan.tier1.scenario_oss_lifecycle_event
axis: tier1
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier1.tier1_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
---

# OSS ライフサイクルイベント対応

## 一文方針

L1+ 採用 OSS でライセンス変更 / 実質的改廃 / サポート終了が発生した際に、tier2 / tier3 を無改修のまま tier1 facade で透過的に移行し、dual reviewer sign-off と Testcontainers conformance test green を移行完了の客観的証跡とする。

> 朝 9 時、本社 IT 室の tier1 担当者（シニア級）が GitHub PR list を確認し、HashiCorp Vault の BUSL ライセンス変更通知が issue として上がっていることに気付く。手元には `oss_lifecycle.lock.yaml` と Backstage Catalog、Mattermost 越しに ops 軸担当者と dual reviewer 2 名がいる。

## ペルソナ要約

主役: tier1 担当者（シニア級）、目的: L1+ 採用 OSS の EoL / ライセンス変更に対し tier2/tier3 を無改修のまま facade 内部で移行を完結させる

## 現状業務での痛み

- OSS の EOL 到来を事前検知できず、突然「今すぐ全コード書き直し」状態になる
- ライセンス変更（例: BUSL への変更）が notice なく発覚し、法務確認を含む緊急対応が発生する
- 年次 dry-run が文書ベースの確認のみで、実際に移行 toolchain が動くかが未検証のまま EOL を迎える
- dual-write 期間中のデータ不整合を検出する仕組みがなく、移行完了判定が属人的になる

## k1s0 でこう変わる

- `oss_lifecycle.lock.yaml` に登録された OSS のライセンス変更・EOL 通知を CI が監視し、変更を事前検知する
- 年次 dry-run green が移行 PR 作成の物理前提条件となり、「toolchain が動くこと」を毎年証明する
- dual-write 期間中のデータ不整合を observability（metrics / trace）で自動検出し、属人的な判断を排除する
- Testcontainers conformance test が「新 OSS が旧 OSS と等価に動作する」ことを客観的に証明する

## Trigger（発火条件）

L1+ 採用 OSS でライセンス変更 / 実質的改廃 / サポート終了（EOL）が発生した時。

## 想定頻度 / 典型きっかけ

- 想定頻度: イベント駆動（BUSL ライセンス変更等）+ 年次 dry-run
- 典型きっかけ: 「HashiCorp Vault が BUSL に変更し OpenBao fork が L1+ 候補に変わった」「Redis から Valkey への fork 対応が発生した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ ops 軸担当者（Harbor mirror 復旧連携時）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier1）| シニア | 本社 IT 室 | GitHub PR list | イベント分類・移行 toolchain 起動・dual-write 実装・旧 OSS 廃止 |
| 関与（dual reviewer A）| シニア | 本社 IT 室 / リモート | GitHub PR list | 移行 PR レビュー・sign-off |
| 関与（dual reviewer B）| シニア | 本社 IT 室 / リモート | GitHub PR list | 移行 PR レビュー・sign-off |
| 関与（ops 担当者）| 中堅 | 本社 IT 室 | Backstage Catalog | Harbor mirror 復旧・旧 OSS mirror 廃止管理 |

## 個人 KPI / 達成感

- tier2/tier3 への変更件数 0（facade 層での完全吸収）
- 年次 dry-run green 率（L1+ 採用 OSS 全件）
- dual-write 期間中のデータ不整合 0 件（observability 計測）
- 移行完了までの elapsed time ≤ 計画期間

## 工数 / 関与人数 / コスト感

- 初回（ライセンス変更 AGPL 系緊急対応）: 0.5〜1 日、関与 4 名（主役 + dual reviewer 2 名 + security 担当者）
- 平常（計画的 EOL 移行）: 1 週間（toolchain 起動〜dual-write〜廃止）、関与 4〜5 名（+ ops 担当者）
- 失敗時（dual-write 不整合）: +1〜2 日（縮退・調査・再実施）、関与 5〜6 名

## 前提

- `oss_lifecycle.lock.yaml` に対象 OSS が登録済みで、年次 dry-run の実行記録が存在する
- 年次 dry-run が green であることが移行 PR 作成の前提条件
- Testcontainers conformance test が既に存在する

## 流れ

1. **イベント種別の判定と影響範囲スキャン**: イベントを以下に分類し、影響を受ける tier2 / tier3 の範囲を静的解析ツールでスキャンする。
   - ライセンス変更: AGPL / SSPL 系への変更は即時採用停止。Business Source License 型も同様。
   - 改廃（fork / 開発停止 / リポジトリアーカイブ）: 後継 OSS または L2* / L3 への格下げを検討。
   - EOL（公式サポート終了 / CVE 対応打切り）: セキュリティリスクの大きさに応じて移行優先度を設定。

2. **移行 toolchain の起動**: `oss_lifecycle.lock.yaml` に記録済みの移行 toolchain を参照し起動する。
   - schema diff: 旧 OSS と新 OSS の API / スキーマ差分を可視化
   - dual-write: 旧 OSS と新 OSS への同時書き込みで移行期間中のデータ整合性を担保
   - observability 連続性: メトリクス / トレース / ログの名前空間変更を tier1 facade でラップ
   - 業務コード移行ガイド: tier2 / tier3 が影響を受ける場合のみ作成（原則は facade 層で吸収）

3. **dual-write 期間の透過提供**: 旧 OSS と新 OSS の dual-write 期間中は、両 endpoint を tier1 facade で透過的に提供する。tier2 / tier3 は無改修で継続動作することを保証する。facade 内部での routing ロジックを feature flag で制御する。

4. **移行 PR の作成**: 年次 dry-run green が `oss_lifecycle.lock.yaml` に記録されていることを確認してから移行 PR を作成する。dry-run 未実施または fail の場合は移行 PR を作成しない。

5. **Testcontainers conformance test の green 確認**: 旧 OSS と新 OSS の双方に対して conformance test を実行し、両者ともに green であることを確認してから旧 OSS 経路の廃止に進む。新 OSS のみ green で旧 OSS が fail する状態は移行完了を意味する。

6. **旧 OSS 経路の廃止と文書更新**: 旧 OSS の endpoint を tier1 facade から削除し `04_提供機能カテゴリ.md` の Lv 割付と露出概念 allowlist を更新する。dual-write feature flag を無効化する。

7. **dual reviewer sign-off とリリースノート**: 移行完了を dual reviewer（tier1 2 名）が確認し sign-off する。内部向けリリースノートに移行経緯・移行 toolchain の実績・観測した異常を記録する。

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier1 担当者 | イベント種別判定・影響範囲スキャン | `#tier1-lifecycle BUSL 変更検知: HashiCorp Vault / 影響スキャン開始` |
| 1 日 | tier1 担当者 | 移行 toolchain 起動・dry-run 記録確認 | `dry-run green 確認済 / 移行 PR 作成可能` |
| 2〜5 日 | tier1 担当者 | dual-write 実装・facade 透過提供 | `dual-write feature flag ON / tier2 無改修確認中` |
| 5 日 | dual reviewer A/B | 移行 PR sign-off・conformance test green 確認 | `dual sign-off 完了 / conformance green` |
| 6〜7 日 | tier1 担当者 | 旧 OSS 経路廃止・文書更新 | `旧 OSS endpoint 削除完了 / `04_提供機能カテゴリ.md` 更新済` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（tier1 Library / Server は全業務の通信・認証・観測の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA テレメトリ**: シークレット管理・設定配信 OSS のライフサイクルイベントは SCADA テレメトリ収集経路の認証・暗号化に直接波及し、移行中断で設備データが欠落するリスクがある。
- **警報配信**: 依存 OSS の廃止・fork 移行期間中の dual-write 不整合が、警報メッセージの到達保証に影響を与える可能性がある。

## 関連適合仕様 / 関連 OSS

**関連適合仕様**:
- [OSSライフサイクル適合仕様](../../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md)
- [tier1強制機構](../../../04_詳細設計/02_強制機構/01_tier1強制機構.md)

**関連 OSS**:
- Testcontainers: conformance test 実行基盤（旧 OSS / 新 OSS 双方のコンテナ起動）
- Harbor: 新 OSS の mirror 登録と旧 OSS mirror の廃止管理
- Kyverno: 旧 OSS image の admission 拒否ポリシーを廃止タイミングで有効化

## 期待結果 / 観測指標

- `oss_lifecycle.lock.yaml` に移行完了レコードが追記されている
- Testcontainers conformance test が新 OSS で全 green
- `04_提供機能カテゴリ.md` の Lv 割付と露出概念 allowlist が新 OSS に更新されている
- dual reviewer（tier1 2 名）の sign-off が PR に記録されている
- tier2 / tier3 への変更が 0 件（facade 層での完全吸収）
- dual-write 期間中のデータ不整合 0 件（observability で計測）

## 失敗時の挙動 / escalation

- **年次 dry-run 未実施 / fail**: 移行 PR 作成を禁止し、dry-run の実施 / 修正を優先する。CI の dry-run gate が merge を阻止する。escalate 先: tier1 担当者 dual reviewer（SLA: 72 時間以内に dry-run 再実施）。Backstage runbook `oss-lifecycle-dryrun` を参照。
- **dual-write 期間中のデータ不整合検出**: dual-write を即時停止し旧 OSS 経路のみに縮退。postmortem を必須とする。escalate 先: ops 担当者へ Mattermost `#tier1-incident` で即時報告（SLA: 検出後 1 時間以内に縮退完了）。
- **ライセンス変更（AGPL/SSPL 系）**: 依存導入 lint が即時 fail し merge 阻止。OSS を依存から削除し代替を探索する。escalate 先: security 担当者へ Mattermost `#security-incident` で通報（SLA: 検出後 2 時間以内）。
- **Testcontainers conformance test fail（新 OSS）**: 新 OSS の採用計画を見直し、代替 OSS の評価フロー（シナリオ 01）に戻る。escalate 先: tier1 担当者 dual reviewer（SLA: 48 時間以内に代替案提示）。
- **旧 OSS の廃止期限超過**: SRE / ops 軸と連携し、廃止計画を更新する。security 軸に EOL 状況を報告する。escalate 先: ops 担当者 + security 担当者（SLA: 廃止期限 +7 日以内に計画更新）。Backstage runbook `oss-eol-escalation` を参照。

## 失敗パターン (anti-pattern)

- **dry-run 未実施のまま移行 PR 作成**: 移行 toolchain が実際には動かない状態で本番移行を始め、データ不整合が顕在化してから気付く。dry-run green を移行 PR の物理前提条件として CI gate で強制する。
- **dual-write を省いて即切替**: 旧 OSS を削除してから新 OSS の問題が発覚し、rollback 不可になる。dual-write 期間を必ず設け、conformance test green を両 OSS で確認してから旧 OSS 廃止に進む。
- **tier2/tier3 担当者に直接変更を依頼**: facade 層で吸収できているはずの変更が漏れ出し、tier2/tier3 のコードに手が入ってしまう。tier2/tier3 への変更件数 0 を KPI として監視し、facade 実装を修正する。

## 関連参照

- [新規 OSS 採用評価](01_新規OSS採用評価.md)
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
