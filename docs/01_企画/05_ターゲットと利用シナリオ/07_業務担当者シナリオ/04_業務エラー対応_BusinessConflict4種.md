---
id: plan.overview.scenario_business_operator_business_conflict
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier3.tier3_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D]
  proof_classes: []
---

# 業務エラー対応_BusinessConflict4種

## 一文方針

stale_write / lost_update / supersede / concurrent_edit の 4 subtype それぞれの UI 分岐 (field-level rebase / 3-way merge / silent toast / presence indicator) を現場担当者目線で踏む。

> 午後 2 時、検査担当者が測定値を入力して「送信」をタップした直後、画面に黄色いバナーが現れた。「このレコードは他のユーザーによって更新されています (stale_write)。現在の値と比較して、送信しますか？」。担当者は表示された差分を見て、自分の測定値が正しいことを確認し「上書き送信」をタップする。10 分後、別の担当者が同じロットを同時に編集していたことを示す presence indicator が画面に表示され、concurrent_edit の調整が必要になった。

## ペルソナ要約

主役: 業務担当者 (現場スタッフ級)、目的: BusinessConflict の 4 パターンを適切に判断・操作して業務データの整合性を守る

## 現状業務での痛み

- 複数担当者が同じ記録を操作しているかどうかわからず、後から上書きされて自分の入力が消える
- 「送信エラー」とだけ表示され、なぜエラーになったのか・どう対処すればよいのかが不明
- 競合解消のために担当者同士が口頭で調整する必要があり、作業が中断する
- エラーを恐れて入力を後回しにする習慣が生まれ、記録の遅延が常態化する

## k1s0 でこう変わる

- BusinessConflict の 4 subtype それぞれに適した UI (rebase / merge / toast / presence) が表示されるため、担当者が適切な対処を選択できる
- 競合の原因 (誰がいつ更新したか) が画面上で明示され、口頭調整が不要になる
- silent toast で自動解消される軽微な競合 (supersede) は担当者の手を煩わせない
- presence indicator で同時編集者を事前に把握でき、競合の発生を予防できる

## Trigger

業務エラー (BusinessConflict) が tier3 SPA に表示された時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 週次〜日次 (多人数作業 / 輻輳時間帯に集中)
- 典型きっかけ: 複数担当者が同一ロットの検査結果を操作する時間帯 / 朝礼後の一斉入力時
- 頻度根拠: 製造現場では複数担当者が同一ロットを扱う機会が頻繁にあり、特に交代時間帯や一斉作業時に競合が集中して発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 業務担当者A (主役) | 現場スタッフ | ライン検査台 | SPA 検査入力フォーム | BusinessConflict 確認・解消操作 |
| 業務担当者B | 現場スタッフ | ライン検査台 | SPA 検査入力フォーム | concurrent_edit 時の同時編集者 |
| 品質管理担当 | 品質管理スタッフ | 品質管理室 | 品質 DB ダッシュボード | 解消後の記録確認 |

## 個人 KPI / 達成感

- BusinessConflict 解消所要時間 (目標: 各 subtype で 2 分以内)
- 競合による入力放棄件数ゼロ (全件を適切に解消して送信完了)
- escalation 不要で自己解決できた件数の割合
- 解消後に品質 DB に正しい値が記録されていることの確認

## 工数 / 関与人数 / コスト感

- stale_write (field-level rebase): 1〜2 分
- lost_update (3-way merge): 2〜3 分
- supersede (silent toast、自動解消): 0 分 (担当者操作不要)
- concurrent_edit (presence indicator + 調整): 3〜5 分
- 失敗時 (自己解消できず escalation): 10〜15 分
- 関与人数: 1〜3 名

## 前提

- SPA が BusinessConflict の 4 subtype を識別して適切な UI を表示できる状態
- tier2 API が競合検出・ETag / Last-Modified 管理を実装済み
- presence indicator が有効で、同時編集者のアバター・名前が表示される
- 業務担当者が「競合が発生することがある」という基本理解を研修済みである

## 流れ

### 共通フロー

1. SPA の入力フォームに測定値を入力し「送信」をタップする
2. tier2 API が競合を検出し、BusinessConflict サブタイプを含むエラーレスポンスを返す
3. SPA が subtype に応じた UI を表示する

### stale_write (古い値での上書き試みを検出)

4. 黄色バナー「このレコードは他のユーザーによって更新されています」と field-level の差分 (自分の値 vs 現在の server 値) を表示
5. 担当者が差分を確認し「上書き送信」または「server の値を採用」を選択
6. 選択した値で送信完了、トーストで確認

### lost_update (更新消失の危険を検出)

4. オレンジバナー「あなたが見ていた値は古い値です」と 3-way merge 画面 (元の値 / 自分の変更 / 他者の変更) を表示
5. 担当者がフィールドごとにどちらの値を採用するかを選択
6. merge 結果を確認して「この内容で送信」をタップし完了

### supersede (後から来た更新が前を無効化)

4. SPA が自動的に新しい値を採用し「最新の値に更新しました」silent toast を 3 秒表示
5. 担当者の操作不要 (トーストを確認するだけ)

### concurrent_edit (同時編集中を検出)

4. presence indicator に「担当者B が同じレコードを編集中」アバターが表示される
5. 担当者A が編集を一時停止するか、担当者B の完了を待つか選択する
6. 担当者B の送信完了後、担当者A の画面が最新値に更新される
7. 担当者A が残りのフィールドを入力して送信完了

## Timeline

| T+ | actor | action | Mattermost/端末通知例 |
|----|-------|--------|---------------------|
| T+0s | 業務担当者A | 送信ボタンタップ | — |
| T+1s | tier2 | BusinessConflict 検出・subtype 判定 | — |
| T+1s | SPA | subtype 別 UI 表示 | 端末: 「競合が検出されました (stale_write)」 |
| T+30s | 業務担当者A | 差分確認・解消操作 | — |
| T+60s | 業務担当者A | 解消選択・再送信 | — |
| T+61s | tier2 | 受付・品質 DB 記録 | — |
| T+62s | SPA | 「記録しました」トースト | 端末: 「競合を解消して記録しました」 |
| T+65s | 品質管理担当 | ダッシュボードで最新値確認 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 品質検査結果配信 | 高 | 複数担当者による同一ロット検査結果の競合解消が主要シナリオ |
| 在庫最新値 | 中 | 在庫数量の concurrent_edit が発生しやすい |
| 受注 sub | 中 | 受注数量変更時の stale_write が発生するケース |

## 現場の物理環境

- 騒音: ライン検査台周辺 (60〜80 dB)
- 照度: 検査台上 500〜1000 lux
- 防塵: 指先露出型手袋でのタッチ操作
- 立位時間: 検査工程中は立位継続
- 心理的プレッシャー: ライン停止を避けるため競合解消を素早く行う必要がある

## 使用デバイス・端末

- 主端末: iPad / Windows tablet
- SPA アクセス: Safari / Chrome または PWA
- presence indicator: SPA 画面上のアバター表示

## 業務手順書 (SOP) との対応

| SOP 条項 | k1s0 対応 |
|----------|----------|
| ISO 9001 §8.6 製品・サービスのリリース | 競合解消後の確定値のみが品質証跡として記録される |
| 社内品質規程 §5.5 記録の修正手順 | field-level rebase / 3-way merge が公式の修正手順として承認される |
| 社内品質規程 §5.6 記録の同時更新防止 | presence indicator による同時編集の事前通知が同時更新防止策となる |

## 関連適合仕様 / 関連 OSS

- クライアント状態適合仕様 (BusinessConflict 4 subtype の UI 分岐)
- スキーマ進化適合仕様 (フィールド追加時の競合動作定義)
- Bidi 適合仕様 (presence indicator のリアルタイム同期)
- React (SPA フロントエンド)
- ETag / Last-Modified (競合検出メカニズム)

## 期待結果 / 観測指標 / 受入条件

- 4 subtype それぞれで適切な UI が表示され、担当者が解消操作を完了できる
- stale_write / lost_update の解消後に正しい値が品質 DB に記録される
- supersede が silent toast で自動解消され、担当者の操作を要求しない
- concurrent_edit で presence indicator が 1 秒以内に表示される
- 受入条件: 4 subtype の E2E テストシナリオが全て pass

## 失敗時の挙動 / escalation

- 解消操作後の再送信も失敗した場合: 「解消できませんでした。担当者に連絡してください」メッセージと Mattermost での品質管理担当への通知
- concurrent_edit で相手が長時間 (5 分以上) 編集中のまま: 「担当者B は 5 分以上編集中です。強制的に自分の変更を送信しますか？」確認ダイアログを表示
- escalation: 同一レコードで 3 回以上競合が繰り返される場合はライン管理者に Mattermost でアラートを送信

## 失敗パターン (anti-pattern)

1. **差分を確認せずに「上書き送信」を即座にタップする**: 他の担当者の正当な変更を消してしまう可能性がある。SPA は差分が存在する場合、「上書き送信」ボタンを 3 秒間非活性にして差分表示を読む時間を確保する
2. **concurrent_edit の presence indicator を無視して同時入力を続ける**: 入力完了後に競合が多発し、解消コストが増大する。SPA は presence indicator が表示されている間、入力フィールドに「同時編集中」ウォーターマークを表示して注意を促す
3. **3-way merge 画面で全フィールドを「自分の変更を採用」で統一する**: lost_update の場合は他者の変更が正当な可能性がある。SPA は全フィールドを一方に統一した場合「全フィールドを自分の変更で上書きしようとしています。確認しますか？」の二重確認ダイアログを表示する

## 関連参照

- `docs/01_企画/05_ターゲットと利用シナリオ/07_業務担当者シナリオ/02_検査結果入力_オンライン.md`
- `docs/01_企画/05_ターゲットと利用シナリオ/07_業務担当者シナリオ/03_25h_オフライン業務記録.md`
- `arch.tier3.tier3_index`
