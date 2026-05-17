---
id: plan.overview.scenario_business_admin_alert_escalation_policy
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D]
  proof_classes: []
---

# 警報escalation_政策変更

## 一文方針

警報の escalation 政策（担当者 / 時間帯 / severity 別 routing 先）を Backstage プラグインから変更し、ops 担当者の runbook との整合を確認する。

> 月初め、業務管理者は夜間当直担当者の異動通知を受け取る。Backstage プラグインの警報 escalation 政策画面を開き、夜間帯の routing 先を現担当者から新担当者に変更する。severity P1 の routing 先に新担当者のセキュリティキー登録状況を確認し、変更を保存する。ops 担当者に runbook の担当者欄更新を依頼し、テスト警報を送信して新担当者への routing を確認する。

## ペルソナ要約

主役: 業務管理者（シニア級）、目的: 担当者異動・体制変更に合わせて警報 escalation 政策を正確に更新し、未通知警報が発生しない体制を維持する

## 現状業務での痛み

- 担当者異動時の警報 routing 先変更が担当者間の口頭連絡に依存し、変更の反映漏れが発生する
- 夜間・休日の警報 routing 先が最新状態か確認する手段がなく、緊急時に連絡が届かないことがある
- 警報の severity と routing 先の対応関係がスプレッドシートと実システムで一致していない場合がある
- routing 先変更後のテスト実施が省略され、変更が正しく反映されているかを緊急時まで確認できない

## k1s0 でこう変わる

- Backstage の警報 escalation 政策画面から業務管理者が単独で routing 先を変更でき、変更が即座に反映される
- テスト警報送信機能により、変更後の routing を本番環境で事前確認できる
- 全政策変更が audit hash chain に記録されるため、誰がいつ変更したかを即座に確認できる
- ops 担当者の runbook との整合確認が変更フローに組み込まれており、runbook 更新依頼が自動送付される

## Trigger

担当者異動 / 夜間体制変更 / 新警報種別追加時（人事異動通知受信または体制変更決定通知をトリガーとする）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「夜間当直担当者の異動で routing 先を更新する必要がある」「新しい警報種別（設備 A 専用の振動警報）が追加され severity マッピングを設定する」「夏季休暇期間の限定的な routing 先変更」
- 頻度根拠: 製造業では月次で担当者の体制変動や季節的な警報体制変更が発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 業務管理者（主役） | シニア | 事務所 | Backstage プラグイン | escalation 政策変更・テスト警報送信 |
| ops 担当者 | シニア | 開発拠点 | ops ダッシュボード | runbook 担当者欄の更新 |
| 新担当者 | 現場スタッフ / 管理者 | 工場フロア / 事務所 | SPA / Mattermost | テスト警報受信確認 |

## 個人 KPI / 達成感

- 政策変更完了からテスト警報送信・受信確認までの所要時間（目標: 30 分以内）
- 担当者異動通知受領から政策変更完了までの所要時間（目標: 1 営業日以内）
- テスト警報の routing 確認率（変更ごと）100%
- ops 担当者への runbook 更新依頼送付率（変更ごと）100%

## 工数 / 関与人数 / コスト感

- 平常（単一担当者の routing 先変更）: 20〜40 分（変更 + テスト警報 + ops への依頼）
- 体制全体の更新（夜間体制変更など）: 1〜2 時間
- 失敗時（テスト警報が届かない）: +30 分〜1 時間（tier2 / ops 担当者と協働調査）
- 関与人数: 3〜4 名（業務管理者 + 新担当者 + ops 担当者 + 必要に応じて tier2 担当者）

## 前提

- 業務管理者が Backstage の警報 escalation 政策画面へのアクセス権を保有している
- 新担当者が SPA / Mattermost のアカウントを保有し、WebAuthn 登録済みである
- 変更対象の警報種別と severity マッピングが tierいつ で定義されている
- ops 担当者の runbook が最新状態に保たれている

## 流れ

1. 担当者異動通知（氏名 / 担当期間 / 担当時間帯）を受け取り、Backstage の警報 escalation 政策画面を開く
2. 変更対象の時間帯 / severity 区分 / 警報種別を選択する
3. routing 先を旧担当者から新担当者に変更する（Backstage の担当者ディレクトリから選択）
4. 新担当者の WebAuthn 登録状況が「登録済み」であることを確認する（P1 警報の step_up 要件）
5. diff preview で変更内容（routing 前後の担当者名・時間帯）を確認する
6. 変更理由欄に異動通知番号を記録し、保存ボタンを押下する
7. 「テスト警報送信」ボタンを押下し、新担当者に届くことを確認する
8. 新担当者から受信確認の返答を受け取る
9. ops 担当者に runbook 更新依頼を送付する（Backstage の runbook 整合依頼フォームを使用）
10. Mattermost `#alert-policy` に変更完了を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | 業務管理者 | 異動通知受領・Backstage 開く | メール: 「田中さんが夜間担当から異動。後任: 鈴木さん」 |
| T+10m | 業務管理者 | routing 先変更・WebAuthn 登録確認 | Backstage: 「鈴木さん: WebAuthn 登録済み」 |
| T+20m | 業務管理者 | diff preview 確認・異動番号記録・保存 | — |
| T+21m | tier2 | 政策反映・audit emit | Backstage: 「escalation 政策更新完了。audit ログ発行済み」 |
| T+22m | 業務管理者 | テスト警報送信ボタン押下 | — |
| T+23m | SPA / Mattermost | 新担当者（鈴木さん）にテスト警報届く | 端末: 「[テスト] P2 警報: 圧力センサ 異常」 |
| T+25m | 新担当者 | テスト警報受信確認・業務管理者に返答 | Mattermost: 「テスト警報受信しました」 |
| T+30m | 業務管理者 | ops 担当者への runbook 更新依頼送付 | Mattermost `#ops-runbook`: 「夜間担当 routing 変更完了。runbook 更新依頼」 |
| T+32m | 業務管理者 | `#alert-policy` 完了報告 | Mattermost `#alert-policy`: 「夜間担当 routing 変更完了（#HR-2024-088）」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報 escalation 政策変更が警報の routing 先・応答担当者に直接影響する |
| ライン稼働監視 | 高 | ライン異常警報の escalation 先が正しく設定されていないと緊急対応が遅延する |
| SCADA テレメトリ収集 | 中 | SCADA 異常警報の severity マッピングが escalation 政策で定義される |

## Backstage プラグイン操作 UI

- **警報 escalation 政策画面**: 時間帯（日中 / 夜間 / 休日）× severity（P1 / P2 / P3）× 警報種別の matrix 表示。各セルに担当者名と連絡先が表示される
- **担当者ディレクトリ選択**: 変更先担当者を組織ディレクトリから検索・選択する。WebAuthn 登録状況がアイコンで表示される
- **テスト警報送信パネル**: 警報種別と severity を選択して指定担当者にテスト警報を送信する。テスト警報には `[テスト]` プレフィックスが付与される
- **runbook 整合依頼フォーム**: ops 担当者へ runbook 更新依頼を送付する。変更内容のサマリが自動入力される

## 業務管理者の決定権限境界

- **実施可（単独で完結）**: routing 先担当者の変更 / テスト警報送信 / runbook 更新依頼送付
- **escalation 必要（tier2 担当者）**: 新規 severity レベルの追加 / 警報 routing ロジック（条件分岐）の変更
- **escalation 必要（ops 担当者）**: runbook の実際の更新作業 / 新担当者への escalation 手順説明

## 関連適合仕様

- [SLO 適合仕様](../../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
- ops 強制機構

## 期待結果 / 観測指標 / 受入条件

- 政策変更後のテスト警報が新担当者に 30 秒以内に届く
- 新担当者からの受信確認が得られている
- audit ログに変更者 / 異動番号 / 変更前後の担当者名が記録されている
- ops 担当者への runbook 更新依頼が送付されている
- 受入条件: 上記 4 点が E2E テストで全て pass

## 失敗時の挙動 / escalation

- **テスト警報が新担当者に届かない**: Backstage が「送信済み（未確認）」を表示し、5 分後にリマインダーを送付。5 分経過後も未確認の場合: escalation 先 tier2 担当者 Mattermost `#tier2-incident`（SLA: 30 分以内）
- **新担当者の WebAuthn 未登録（P1 警報の step_up 要件不満足）**: Backstage が「P1 警報の routing には WebAuthn 登録が必要です」と表示し、routing 先設定を保留する。新担当者に WebAuthn 登録を依頼してから保存する
- **ops 担当者の runbook 更新が 1 営業日以上未完了**: Mattermost `#alert-policy` に自動リマインダーが送付される

## 失敗パターン（3 例）

1. **テスト警報送信を省略して変更を完了とみなす**: 夜間の本番警報で新担当者に届かないことが発覚し、緊急対応が遅延する。テスト警報送信と受信確認を完了の必須ステップとして SOP に明記する
2. **WebAuthn 未登録担当者を P1 警報の routing 先に設定する**: P1 緊急事態で step_up 認証ができず、承認が遅延する。Backstage の保存時バリデーションで WebAuthn 未登録担当者の P1 設定を拒否する
3. **runbook 更新依頼を送付せずに変更を完了する**: ops 担当者の runbook が旧担当者のままで、実際の緊急対応時に手順書と実態が乖離する。Backstage の保存後に runbook 更新依頼フォームへの入力を必須ステップとして手順書に明記する

## 関連参照

- [業務管理者シナリオ INDEX](README.md)
- [03_緊急対応_設備停止承認](03_緊急対応_設備停止承認.md)
- `arch.tier2.tier2_index`
- `req.team.tier_engineer_requirement`
