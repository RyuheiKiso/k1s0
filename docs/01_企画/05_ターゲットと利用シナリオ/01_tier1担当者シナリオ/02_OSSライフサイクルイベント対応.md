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
  defense_in_depth_layers: []
  proof_classes: []
---

# OSS ライフサイクルイベント対応

## 一文方針

L1+ 採用 OSS でライセンス変更 / 実質的改廃 / サポート終了が発生した際に、tier2 / tier3 を無改修のまま tier1 facade で透過的に移行し、dual reviewer sign-off と Testcontainers conformance test green を移行完了の客観的証跡とする。

## Trigger（発火条件）

L1+ 採用 OSS でライセンス変更 / 実質的改廃 / サポート終了（EOL）が発生した時。

## 想定頻度 / 典型きっかけ

- 想定頻度: イベント駆動（BUSL ライセンス変更等）+ 年次 dry-run
- 典型きっかけ: 「HashiCorp Vault が BUSL に変更し OpenBao fork が L1+ 候補に変わった」「Redis から Valkey への fork 対応が発生した」

## 主役 / 関与者

- 主役: tier1 担当者（シニア級）
- 関与: dual reviewer（tier1 担当者 2 名）/ ops 軸担当者（Harbor mirror 復旧連携時）

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

## 関連参照

- [新規 OSS 採用評価](01_新規OSS採用評価.md)
- [tier1 設計方針](../../../03_概要設計/02_tier1設計方針/README.md)
- [tier1 担当者シナリオ index](README.md)
