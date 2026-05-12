---
id: plan.overview.anti_pattern_catalog
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.target_use_case
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 失敗シナリオ集 (anti-pattern catalog)

各パターンの構成: パターン名 / 状況説明 / 誤った行動 / なぜいけないか / 物理 enforce 機構 / 正しい対処

---

## 業務担当者 (現場オペレータ) アンチパターン

### AP-operator-01: IndexedDB への PII 平文書き込み
**状況**: オフライン業務記録機能を実装する際、フロントエンドエンジニアが IndexedDB に顧客識別子をそのまま書き込んだ。  
**誤った行動**: `db.put({ customerId: "CUST-001", inspectionResult: "NG" })` のように PII を暗号化せずに格納。  
**なぜいけないか**: デバイスを紛失・盗難された場合、PII が平文で漏洩する。GDPR/PIPA の「適切な技術的安全管理措置」義務違反となる。  
**物理 enforce 機構**: ESLint カスタムルール `no-plaintext-pii-indexeddb` が CI で拒否。  
**正しい対処**: → [03_tier3担当者シナリオ/04_オフライン業務記録] の暗号化ストレージ手順を参照。

### AP-operator-02: 未送信件数を確認せず帰宅
**状況**: 現場オペレータがネットワーク復旧を確認せず帰宅し、翌朝オフライン記録 47 件が未送信のまま発覚した。  
**誤った行動**: 帰宅前チェックリストをスキップして退出。  
**なぜいけないか**: 業務データの連続性が失われ、翌日の生産指示・品質判定に誤りが生じるリスクがある。  
**物理 enforce 機構**: Companion アプリが未送信件数ゼロになるまで「帰宅確認完了」ボタンを無効化する UI gate。  
**正しい対処**: → [07_業務担当者シナリオ/11_帰宅前_未送信件数確認] のチェックリスト手順を参照。

### AP-operator-03: オフライン時に別デバイスで再入力
**状況**: タブレットがオフラインになったため、手近にあったスマートフォンで同じデータを再入力した。  
**誤った行動**: 同一 tenant / 同一業務 ID に対して 2 つのデバイスから独立した record を作成。  
**なぜいけないか**: オンライン復旧時に BusinessConflict が発生し、どちらを正とするか自動解決できず手動解決コストが発生する。最悪の場合、二重計上となる。  
**物理 enforce 機構**: Outbox テーブルに idempotency_key (device_id + local_seq) を付与し、重複挿入は DB 側で unique constraint 違反として排除。  
**正しい対処**: → [07_業務担当者シナリオ/03_25h_オフライン業務記録] の conflict resolution 手順を参照。

### AP-operator-04: break_glass 発火を上長に黙認させる
**状況**: 夜間に設備停止が迫り、ops に連絡が取れないため現場担当者が break glass を独断で発火させたが、翌朝まで誰にも報告しなかった。  
**誤った行動**: break glass 行使後の即時通知義務を履行せず、翌朝の朝礼で初めて口頭報告。  
**なぜいけないか**: break glass は特権昇格操作であり、事後レビューなしでは悪用との区別ができない。post_review が翌日以降にずれると証跡が揮発するリスクがある。  
**物理 enforce 機構**: break glass 行使と同時に SIEM がアラートを発報し、ops / security の PagerDuty へ即時通知する admission webhook。  
**正しい対処**: → [07_業務担当者シナリオ/13_break_glass発火時の現場対応] の即時通知手順を参照。

### AP-operator-05: 警報を「よくある誤検知」として無視
**状況**: ライン稼働監視の警報が 1 週間に 5 回鳴ったため、担当者が「センサー誤検知」と判断して対応せずにいたところ、7 回目で実際の設備異常が発覚した。  
**誤った行動**: 警報に対して対応記録を作成せず、是正アクションなしに close。  
**なぜいけないか**: 警報 fatigue が蓄積し、本物の異常を見逃すリスクが増大する。また警報無視の記録がない場合、監査時に説明責任を果たせない。  
**物理 enforce 機構**: 警報は acknowledges + escalation 記録なしに 30 分以内に自動 escalate される Temporal workflow。  
**正しい対処**: → [07_業務担当者シナリオ/09_警報受信_アラート対応] の escalation 手順を参照。

---

## 業務管理者 アンチパターン

### AP-manager-01: Backstage step_up をスキップした緊急マスタ更新
**状況**: 製品マスタの誤りに気づいた管理者が、WebAuthn step_up の認証待ち時間を省略するため、通常の read 権限のまま直接 API を呼んでマスタを書き換えた。  
**誤った行動**: step_up が不要な読み取り専用 API エンドポイントを悪用してマスタを変更するために、フロントエンドをバイパス。  
**なぜいけないか**: step_up は書き込み操作への不正アクセス防止の最終防衛ラインであり、バイパスすると監査証跡が欠損する。また特権昇格記録がないため事後に正当性を証明できない。  
**物理 enforce 機構**: すべてのマスタ書き込み API は Authorization header の `acr` claim に `step_up` が存在しない場合に HTTP 403 を返す middleware enforcement。  
**正しい対処**: → [08_業務管理者シナリオ/01_マスタ更新_品目仕入先BOM] の step_up 経由フローを参照。

### AP-manager-02: DSAR を audit 以外のソースで対応
**状況**: DSAR 申請を受け取った管理者が、audit log を経由せず自身の記憶と手元のスプレッドシートで「保有 PII なし」と回答した。  
**誤った行動**: 正規の PII DSAR export 機能を使用せず、非公式ソースで回答書を作成。  
**なぜいけないか**: 見落とし・不正確な開示が発生した場合、法的責任を負う。また audit trail が残らないため、回答の正確性を事後に証明できない。  
**物理 enforce 機構**: DSAR 回答書発行は data 軸の DSAR export pipeline 経由でのみ生成可能な PDF テンプレートを強制し、手動スプレッドシートは電子押印できない構造とする。  
**正しい対処**: → [08_業務管理者シナリオ/02_監査ログ検索_PII_DSAR] のパイプライン手順を参照。

### AP-manager-03: DPA なしで partner に PII を提供
**状況**: 新規 partner との連携テストで早期検証を優先し、DPA 締結前にテストデータとして本番 PII をサンプル提供した。  
**誤った行動**: 法務 DPO の承認を受けずに partner の担当者に PII を含むデータを送付。  
**なぜいけないか**: GDPR 第 28 条違反となる。DPA がない状態での第三者提供は規制当局の制裁対象となる。  
**物理 enforce 機構**: partner IdP federation の設定フォームに法務 DPO 承認チケット番号の入力が必須となっており、未入力では federation config を保存できない。  
**正しい対処**: → [08_業務管理者シナリオ/04_partner連携設定_IdP_federation] の DPA 事前確認手順を参照。

### AP-manager-04: 業界規制自主点検を形式的に通過
**状況**: ISO 9001 自主点検チェックリストを、実際の手順書・ログを確認せずにチェックボックスを記入して提出した。  
**誤った行動**: 実地確認なしで全項目「適合」と記入し、外部監査前の社内承認を通過。  
**なぜいけないか**: 外部監査で不適合が発覚した場合、組織的な虚偽記録として扱われる可能性がある。また実態と乖離した記録が是正機会を失わせる。  
**物理 enforce 機構**: 自主点検フォームの各項目は関連 runbook の URL と実行ログのリンクを必須入力とし、未入力項目は承認フローに進めない構造。  
**正しい対処**: → [08_業務管理者シナリオ/10_業界規制_自主点検] の証跡付き点検手順を参照。

### AP-manager-05: tenant offboarding を archive なしで実行
**状況**: tenant の解約対応として、データ削除を依頼された管理者が archive プロセスをスキップして直接削除コマンドを実行した。  
**誤った行動**: `DELETE FROM tenants WHERE id = ?` を archive 状態への遷移なしに直接実行。  
**なぜいけないか**: 法定保存期間中のデータが消失し、後日の監査対応・訴訟対応が不可能となる。また crypto-shred が未完了の場合、PII 漏洩リスクが残る。  
**物理 enforce 機構**: tenant 削除 API は `status = archived` かつ `retention_end < now()` でない限り HTTP 409 を返す DB trigger + application layer double check。  
**正しい対処**: → [08_業務管理者シナリオ/07_テナント_onboarding_offboarding] の archive-first 手順を参照。

---

## 外部監査人 アンチパターン

### AP-auditor-01: 被監査組織の内部システムに直接アクセス
**状況**: 監査効率化のため、監査担当者が被監査組織の VPN 接続を借用して audit log を直接参照した。  
**誤った行動**: 被監査組織の IAM アカウントを使用してデータベースに直接接続し、audit log をダウンロード。  
**なぜいけないか**: 監査人が被監査対象のインフラに依存することで独立性が失われる。また監査人のアクセスログが被監査組織の audit trail に記録されないため証跡が汚染される。  
**物理 enforce 機構**: 監査人向け専用の read-only export API が存在し、それ以外のアクセス経路は Kyverno admission policy で拒否される。  
**正しい対処**: → [09_外部監査人シナリオ/01_audit_hash_chain独立検証] の独立アクセス経路を参照。

### AP-auditor-02: WORM 検証を口頭確認のみで済ませる
**状況**: 監査日程が逼迫していたため、Object Lock の設定を担当者の口頭説明で「確認済み」とした。  
**誤った行動**: `aws s3api get-object-lock-configuration` コマンドを自ら実行せずに口頭説明を証拠として記録。  
**なぜいけないか**: WORM の独立検証なしでは監査報告書の信頼性が失われる。また口頭のみの確認は改竄・虚偽説明のリスクを排除できない。  
**物理 enforce 機構**: 監査チェックリストシステムは各 WORM 検証項目に CLI 実行結果のスクリーンショットまたはログハッシュを必須添付とし、未添付の場合は「検証済み」マークを付与できない。  
**正しい対処**: → [09_外部監査人シナリオ/02_WORM_Object_Lock検証] の独立実行手順を参照。

### AP-auditor-03: provenance 検証を一部省略して合格判定
**状況**: SLSA L3 検証で 50 artifact 中 48 件が合格したため、残り 2 件を「時間内に処理できなかった」として未検証のまま全体合格と判定した。  
**誤った行動**: 未検証の 2 artifact を「合格推定」として報告書に記載。  
**なぜいけないか**: SLSA L3 は全 artifact の連鎖検証を要件とする。1 件でも未検証があれば L3 非適合であり、虚偽の準拠報告となる。  
**物理 enforce 機構**: provenance 検証スクリプトは全 artifact のハッシュリストに対して完全ループ実行し、1 件でも失敗・未完了があれば exit code 1 で終了する。  
**正しい対処**: → [09_外部監査人シナリオ/05_SLSA_L3_provenance独立検証] の全件検証手順を参照。

### AP-auditor-04: KEK shamir ceremony 観察で定足数未達成を黙認
**状況**: KEK shamir ceremony に必要な keyholder 3/5 名のうち 2 名が欠席したが、残り 3 名で儀式を続行した。監査人は「作業は完了した」と記録した。  
**誤った行動**: ceremony 手順書の「最低 3/5 keyholder 必須」条件が満たされているにもかかわらず、欠席者の代理入力があったことを見逃した。  
**なぜいけないか**: shamir secret sharing の定足数ルールが形骸化すると、KEK の不正再構成リスクが実現する。  
**物理 enforce 機構**: ceremony ソフトウェアは keyholder の biometric 認証 + YubiKey の両方がそろった場合のみ share を受け付ける。代理入力は技術的に不可能。  
**正しい対処**: → [09_外部監査人シナリオ/06_KEK_shamir_ceremony観察] の定足数確認手順を参照。

### AP-auditor-05: 監査報告書を被監査組織のシステムで保管
**状況**: 監査完了後、報告書ドラフトを被監査組織が提供した SharePoint に保存した。  
**誤った行動**: 被監査組織の管理下にあるストレージに最終報告書を配置し、WORM 保全を実施しなかった。  
**なぜいけないか**: 被監査組織が報告書を改竄・削除できる状態となり、独立性が失われる。  
**物理 enforce 機構**: 監査報告書は監査機関の WORM バケット + RFC3161 タイムスタンプで封印し、被監査組織への共有は read-only export リンクのみ許可。  
**正しい対処**: → [09_外部監査人シナリオ/10_監査報告書発行_証跡保全] の WORM 保全手順を参照。

---

## ops 担当者 アンチパターン

### AP-ops-01: break glass を事後報告なしで行使
**状況**: 深夜インシデント対応中、escalation 先が応答せず ops 担当者が単独で break glass を行使して本番 DB を直接操作した。翌朝の引き継ぎ時まで報告しなかった。  
**誤った行動**: PagerDuty への通知・Slack への即時投稿・incident ticket 作成を行わずに break glass を継続。  
**なぜいけないか**: break glass は特権操作であり、事後レビューなしでは内部不正との区別ができない。また翌番担当者が状況を把握できずに誤操作するリスクがある。  
**物理 enforce 機構**: break glass admission webhook が発火と同時に SIEM + Slack + PagerDuty へ自動通知し、5 分以内に incident ticket が自動生成される。  
**正しい対処**: → [10_ops担当者シナリオ/07_break_glass_execution] の即時通知手順を参照。

### AP-ops-02: postmortem 前に次リリースを開始
**状況**: インシデント収束から 24 時間後、postmortem が未完了のまま次スプリントのリリース PR を main にマージした。  
**誤った行動**: postmortem 承認 PR のマージを待たずに次リリースのデプロイをトリガー。  
**なぜいけないか**: インシデントの根本原因分析が未完了のまま次リリースが行われると、同一原因の再発が起きる可能性が高い。また postmortem の証跡が次インシデントと混在する。  
**物理 enforce 機構**: release pipeline に `postmortem_approved` ラベルのついた PR が closed になっていることを確認する CI gate が存在し、未完了の場合は deploy job が blocked。  
**正しい対処**: → [10_ops担当者シナリオ/03_postmortem_PR_merge_gate] の gate 手順を参照。

### AP-ops-03: toil 50% 超を「忙しい証拠」として誇示
**状況**: チームの toil 比率が 65% に達したにもかかわらず、「それだけ対応している」と SRE 50% ルールの是正に反対し、改善活動を先送りした。  
**誤った行動**: toil 削減提案を「手を動かす文化を壊す」として却下し、改善 PR を出さなかった。  
**なぜいけないか**: toil 50% 超は SRE 原則の重大違反であり、エンジニアの燃え尽きと信頼性低下を招く。また toil に埋没することで故障兆候の早期発見能力が低下する。  
**物理 enforce 機構**: toil 比率は observability dashboard の SLO として計測され、50% 超が 2 週間継続した場合に自動で engineering manager へエスカレーション alert を送信。  
**正しい対処**: → [10_ops担当者シナリオ/08_SRE_50pc_rule_toil削減] の改善計画手順を参照。

### AP-ops-04: chaos drill を本番環境のみで実施
**状況**: staging 環境の chaos drill が「本番との差異が大きい」という理由でスキップされ、chaos drill が本番環境のみで行われた。  
**誤った行動**: staging での chaos 訓練結果なしに本番 chaos drill を強行。  
**なぜいけないか**: staging での検証をスキップすることで、未知の障害パターンが本番に直撃するリスクがある。また staging での学習コストを無駄にする。  
**物理 enforce 機構**: chaos drill の本番実施チェックリストに「staging drill 完了レポート URL」の入力が必須とされており、未入力では本番ターゲット選択が unlocked されない。  
**正しい対処**: → [10_ops担当者シナリオ/12_chaos_drill_観察_signal評価] の staging-first 手順を参照。

### AP-ops-05: on-call handover を「変化なし」で口頭完了
**状況**: handover 担当者が「特に変わったことはない」と口頭で申し送りし、handover ドキュメントを作成しなかった。次番担当者がインシデント対応中に文脈を失い対応が遅延した。  
**誤った行動**: handover チケットの作成をスキップし、Slack メッセージ 1 行で引き継ぎを完了とした。  
**なぜいけないか**: handover ドキュメントがないと、インシデント発生時に前番の状況・対応履歴・未解決事項が伝わらない。MTTR が上昇し SLO 違反リスクが増大する。  
**物理 enforce 機構**: handover 完了は Backstage の runbook フォームへの提出が必須であり、未提出の場合は次番担当者の on-call duty が officially start されない (PagerDuty schedule が更新されない)。  
**正しい対処**: → [10_ops担当者シナリオ/01_on_call_rotation_handover] の handover チェックリストを参照。

---

## 法務DPO アンチパターン

### AP-legal-01: GDPR 72h を「営業日 72h」と解釈
**状況**: breach が金曜夜に発覚し、法務担当者が「72 営業時間 = 月曜開始で 3 営業日後」と解釈して DPA への通知を水曜に行った。  
**誤った行動**: GDPR 第 33 条の 72 時間を暦時間ではなく営業時間として計算。  
**なぜいけないか**: GDPR の 72 時間は暦時間 (calendar hours) である。土日を挟んでも通知義務は免除されない。遅延通知は制裁金の対象となる。  
**物理 enforce 機構**: breach 確認時刻から 72h のカウントダウンタイマーが自動起動し、60h / 66h / 70h に自動リマインダーが法務 DPO および経営層へ送信される。  
**正しい対処**: → [11_法務DPO担当者シナリオ/01_GDPR_72h_breach通知] の暦時間カウント手順を参照。

### AP-legal-02: crypto-shred を logical delete で代替
**状況**: right to be forgotten 申請を受けた際、KEK destroy の手続きを「複雑」として回避し、該当 PII を論理削除フラグで「削除済み」と処理した。  
**誤った行動**: `UPDATE users SET deleted = true WHERE id = ?` のみを実行し、暗号化データは storage に残存。  
**なぜいけないか**: 論理削除は GDPR/PIPA の「消去」要件を満たさない。暗号化データが storage に残る限り、KEK が存在する限り復元可能であり、消去の立証ができない。  
**物理 enforce 機構**: right to be forgotten の処理パイプラインは KEK destroy 完了の confirmation hash が返るまで「消去完了」ステータスに遷移しない FSM enforcement。  
**正しい対処**: → [11_法務DPO担当者シナリオ/04_right_to_be_forgotten_KEK_destroy承認] の crypto-shred 手順を参照。

### AP-legal-03: DPA なしで partner に PII を提供
**状況**: 新規 partner との API 連携を開始するにあたり、「技術検証フェーズ」として DPA 締結前に本番 PII を含むレスポンスを partner に返すテストを行った。  
**誤った行動**: 法務 DPO の承認なしで partner API key を発行し、PII 含む endpoint を開放。  
**なぜいけないか**: GDPR 第 28 条の処理委託者への要件を満たさない第三者提供となる。また DPA がない状態での提供は個人情報保護法の第三者提供制限にも違反する可能性がある。  
**物理 enforce 機構**: partner API key 発行フォームに DPA 締結済みチェックと法務 DPO 承認番号の入力が必須であり、未入力では key が発行されない (Backstage scaffolder による強制)。  
**正しい対処**: → [11_法務DPO担当者シナリオ/09_partner契約_DPA_締結] の締結前チェックを参照。

### AP-legal-04: 個人情報保護法の 30d を「業務日 30 日」と解釈
**状況**: 国内 breach を確認後、担当者が「業務日 30 日以内に報告すれば良い」と解釈し、土日・祝日を除いたカレンダーで 30 日目に個人情報保護委員会へ報告した。  
**誤った行動**: 個人情報保護法の速報義務 (30 日以内) を暦日ではなく業務日で計算。  
**なぜいけないか**: 個人情報保護法の改正により報告義務の期限は暦日 30 日とされている。業務日換算での遅延報告は義務違反となる。  
**物理 enforce 機構**: breach 確認から 30 暦日のカウントダウンが自動起動し、25d / 28d / 29d に法務 DPO と経営層へ自動リマインダーを送信。  
**正しい対処**: → [11_法務DPO担当者シナリオ/02_個人情報保護法30d報告] の暦日カウント手順を参照。

### AP-legal-05: OSS ライセンス評価を tier 確認のみで完了
**状況**: 新規 OSS 採用評価で tier1 担当者が「MIT ライセンスだから問題なし」と結論付け、法務 DPO のレビューを依頼せずに採用を確定した。  
**誤った行動**: ライセンス文書を tier で分類するのみで、specific patent clause・warranty disclaimer・trademark 条項を確認しなかった。  
**なぜいけないか**: MIT でも patent retaliation clause が含まれる variant が存在する。また商標条項によっては製品名での言及が制限される場合がある。法務確認なしの判断は法的リスクを残す。  
**物理 enforce 機構**: OSS 採用 PR の template に「法務 DPO 承認番号」フィールドが必須項目として含まれており、未入力では PR の merge gate が通過しない。  
**正しい対処**: → [11_法務DPO担当者シナリオ/05_OSSライセンス階層_新規採用承認] の法務評価フローを参照。

---

## tier1 担当者 アンチパターン

### AP-tier1-01: AGPL OSS を dev 依存として一時的に導入
**状況**: 開発ツールとして AGPL-3.0 の CLI を dev dependency に追加し、「開発環境専用だから問題ない」と判断して法務確認を省略した。  
**誤った行動**: `npm install --save-dev agpl-tool` を実行し、OSS 採用評価フローをスキップ。  
**なぜいけないか**: dev dependency が build artifact に含まれる場合、AGPL の感染リスクがある。また CI/CD 環境や Docker image に含まれた場合の解釈は判例が確立していない。  
**物理 enforce 機構**: CI での license-checker が AGPL を検出した場合に pipeline を failed とし、PR マージをブロックする Conftest ポリシー。  
**正しい対処**: → [01_tier1担当者シナリオ/01_新規OSS採用評価] の dev dependency 評価手順を参照。

### AP-tier1-02: L1+ 採用時に migration toolchain 計画なし
**状況**: L1+ 準拠のデータフォーマットを採用する決定をしたが、既存 L1 データの migration toolchain の設計を「後で考える」として採用を強行した。  
**誤った行動**: L1+ の spec 定義のみ完了させ、migration toolchain の設計書なしで implementation を開始。  
**なぜいけないか**: migration toolchain のない L1+ 採用は運用開始後に移行不能な技術的負債となる。また dry-run 未実施のまま本番適用した場合の rollback 手段がなくなる。  
**物理 enforce 機構**: L1+ 採用 ADR (Architecture Decision Record) の template に `migration_toolchain_design_doc` の URL が必須項目として存在し、未入力では ADR が approved ステータスに遷移しない。  
**正しい対処**: → [01_tier1担当者シナリオ/12_L1plus_migration_dryrun] の toolchain 設計手順を参照。

### AP-tier1-03: supply chain 障害時に SBOM なしで対応
**状況**: upstream の npm registry が 2 時間ダウンし、担当者が「影響なし」と判断したが、実際には 3 個の推移的依存が影響を受けていた。SBOM が最新でなかったため特定が遅延した。  
**誤った行動**: supply chain 障害発生時に最新 SBOM を参照せず、自身の記憶と package.json のみで影響範囲を判断。  
**なぜいけないか**: 推移的依存は package.json には記載されない。SBOM がない影響範囲調査は必然的に不完全となり、インシデント対応が遅延する。  
**物理 enforce 機構**: CI の毎ビルドで SBOM を自動生成・更新し、90 日間保存する pipeline job が強制的に実行される。  
**正しい対処**: → [01_tier1担当者シナリオ/09_supply_chain障害対応] の SBOM 参照手順を参照。

### AP-tier1-04: proto 破壊的変更を snapshot テストなしで push
**状況**: proto フィールドのリネームを「内部 API だから」として snapshot test をパスさせずに main ブランチに直接 push した。  
**誤った行動**: snapshot 違反検知 CI をローカルで skip して push。  
**なぜいけないか**: proto の破壊的変更は生成コードの binary incompatibility を引き起こし、rolling update 中に service が crash する。  
**物理 enforce 機構**: proto snapshot test は CI の必須ジョブとして設定され、スキップ不能。PR の merge には全 required check 合格が必須。  
**正しい対処**: → [01_tier1担当者シナリオ/08_公開API_snapshot違反対応] の互換性保持手順を参照。

### AP-tier1-05: SBOM 署名を省略して release
**状況**: release 急ぎで cosign による SBOM 署名ステップを「後で追加する」として省略し、unsigned SBOM を artifact として公開した。  
**誤った行動**: cosign sign コマンドをコメントアウトして release pipeline を通過。  
**なぜいけないか**: unsigned SBOM は改竄検知ができない。また SLSA L3 の要件を満たさず、供給チェーン攻撃の検知が不可能となる。  
**物理 enforce 機構**: release pipeline の最終ステップは cosign で署名された SBOM artifact の hash が Rekor transparency log に登録されていることを検証し、未登録の場合は artifact を公開しない gate。  
**正しい対処**: → [01_tier1担当者シナリオ/11_Library_release切り] の cosign 署名手順を参照。

---

## tier2 担当者 アンチパターン

### AP-tier2-01: テナント識別子を hardcode
**状況**: 特定テナント向けの緊急対応として、tenant_id を定数として hardcode したコードを main にマージした。  
**誤った行動**: `const TENANT = "tenant-A";` として context から tenant_id を注入せずに実装。  
**なぜいけないか**: マルチテナント構造が破壊され、他テナントのデータにアクセスするバグが混入するリスクがある。また hardcode の tenant_id は RLS を完全にバイパスする。  
**物理 enforce 機構**: Semgrep ルール `hardcoded-tenant-id` が hardcode 文字列パターンを検出し CI を failed とする。  
**正しい対処**: → [02_tier2担当者シナリオ/06_tenant_id強制注入追加] の context 注入手順を参照。

### AP-tier2-02: 業界固有概念を業界横断層に置く
**状況**: 製造業向け機能開発で、「検査ロット番号」を共通の domain entity として業界横断層に定義した。  
**誤った行動**: `InspectionLot` クラスを `common/domain` パッケージに配置。  
**なぜいけないか**: 業界横断層に業界固有概念が混入すると、他業界向け展開時にコンパイルエラーまたは意味的汚染が発生する。業界中立性 lint に違反する。  
**物理 enforce 機構**: Conftest ポリシーが `common/domain` パッケージ内の業界固有キーワード (configurable allowlist 外) を検出し CI を blocked。  
**正しい対処**: → [02_tier2担当者シナリオ/05_業界中立性違反検出_是正] の分離リファクタリング手順を参照。

### AP-tier2-03: atomic 三表を個別トランザクションで書く
**状況**: 実装コストを削減するため、Outbox / Inbox / aggregates の三表書き込みを単一トランザクションではなく順次 INSERT で実装した。  
**誤った行動**: `INSERT INTO aggregates` → `INSERT INTO outbox` → `INSERT INTO inbox` を別々の `BEGIN/COMMIT` で実行。  
**なぜいけないか**: 三表が単一トランザクションでない場合、障害時に部分書き込みが発生し exactly-once 保証が失われる。  
**物理 enforce 機構**: Outbox pattern 実装クラスの unit test がトランザクション境界を確認するテストを含み、CI で必須実行される。  
**正しい対処**: → [05_data担当者シナリオ/08_Outbox_atomic三表書込障害対応] の atomic 実装手順を参照。

### AP-tier2-04: Emergency Override を承認チェックなしで実行
**状況**: 生産停止の緊急事態で、Emergency Override の承認フローが「遅すぎる」として承認待ちをキャンセルし、直接コマンドを実行した。  
**誤った行動**: override コマンドを management plane に直接 POST し、approval workflow を迂回。  
**なぜいけないか**: Emergency Override は最高権限の操作であり、無承認実行は内部不正と区別できない。また承認証跡がない操作は監査時に説明できない。  
**物理 enforce 機構**: Emergency Override API は approval token が必須パラメータであり、token なしでは HTTP 403。approval token は 2 名の manager の署名が必要な JWT として発行される。  
**正しい対処**: → [02_tier2担当者シナリオ/12_Emergency_Override実行] の fast-track 承認フローを参照。

### AP-tier2-05: BoundedContext 分割を DB スキーマ変更前に完了宣言
**状況**: ドメイン分割の設計書を完成させ、コードリファクタリングも完了したが、DB スキーマはモノリシックのままで「段階的に移行する」として完了を宣言した。  
**誤った行動**: DB 分割なしで BoundedContext 分割を完了とし、次フェーズの依存タスクをアンブロック。  
**なぜいけないか**: アプリケーション層の BoundedContext 分割が DB レイヤで実現されていない場合、事実上のデータ依存が残存し、将来のスケーリングの障害となる。  
**物理 enforce 機構**: BoundedContext ADR の done criteria に「DB schema migration PR マージ済み」チェックが含まれており、未満の場合は ADR を shipped に遷移できない。  
**正しい対処**: → [02_tier2担当者シナリオ/02_ドメイン分割_BoundedContext] の DB 分割チェックリストを参照。

---

## tier3 担当者 アンチパターン

### AP-tier3-01: 業務ロジックを UI 層に書く
**状況**: 発注承認の金額上限チェックをフロントエンドの JavaScript のみで実装し、API 側には validation を設けなかった。  
**誤った行動**: `if (amount > limit) { showError(); return; }` をフロントエンドのみに実装。  
**なぜいけないか**: フロントエンドのバリデーションは DevTools で容易にバイパスできる。業務ロジックをサーバーサイドで強制しない限り不正操作を防げない。  
**物理 enforce 機構**: API gateway の contract test が「フロントエンドバイパス相当のリクエスト」を送信し、サーバーサイドで正しく拒否されることを検証する。  
**正しい対処**: → [03_tier3担当者シナリオ/01_新業務画面追加] の server-side validation 実装要件を参照。

### AP-tier3-02: a11y を後付けで追加
**状況**: 業務画面の実装が完了してから「a11y 対応が必要」とわかり、aria 属性を後付けで追加した結果、スクリーンリーダーの読み上げ順序が意味的に破綻した。  
**誤った行動**: HTML 構造を変えずに `aria-label` を機械的に追加。  
**なぜいけないか**: a11y は HTML 構造の設計段階から組み込む必要がある。後付け aria は誤ったセマンティクスを提供し、支援技術利用者の UX を悪化させる。  
**物理 enforce 機構**: axe-core による a11y lint が CI で必須実行され、WCAG 2.1 AA 違反が 1 件でもあれば build failed。  
**正しい対処**: → [03_tier3担当者シナリオ/08_a11y_i18n修正] の設計段階からの a11y 実装ガイドを参照。

### AP-tier3-03: 認証ロジックを UI に持ち込む
**状況**: WebAuthn step_up の認証判定ロジックをフロントエンドで実装し、step_up 済みフラグを localStorage に保存して API リクエスト時に送信した。  
**誤った行動**: `localStorage.setItem('stepup', 'true')` を条件分岐の根拠として使用。  
**なぜいけないか**: localStorage は JavaScript から自由に操作できるため、攻撃者が step_up フラグを偽装して特権操作を実行できる。  
**物理 enforce 機構**: サーバーサイドの authorization middleware が JWT の `acr` claim のみを step_up の根拠とし、任意の HTTP header / body フィールドを無視する enforcement。  
**正しい対処**: → [03_tier3担当者シナリオ/07_WebAuthn_step_up_federation] の JWT acr claim 検証手順を参照。

### AP-tier3-04: tier2 stub を bypass して直接 API 呼出し
**状況**: tier2 の generated stub の型が不便であるとして、stub を経由せずに fetch で直接 API エンドポイントを呼び出すコードを実装した。  
**誤った行動**: `fetch('/api/v1/orders', { method: 'POST', body: JSON.stringify(data) })` を stub 経由なしで記述。  
**なぜいけないか**: stub を bypass すると型安全性が失われ、API の破壊的変更が tier3 に気づかれずに混入する。また contract test のカバレッジが落ちる。  
**物理 enforce 機構**: ESLint rule `no-direct-api-fetch` が stub 経由なしの fetch 呼び出しを禁止し CI を failed とする。  
**正しい対処**: → [03_tier3担当者シナリオ/09_tier2_generated_stub経由refactor] のリファクタリング手順を参照。

### AP-tier3-05: pre_release smoke test をスキップしてリリース
**状況**: リリース日に時間が足りず、smoke test を「目視確認で代替」として省略し、本番リリース後に critical bug が発覚した。  
**誤った行動**: CI の smoke test job を skip フラグ付きで実行し、ジョブを強制 pass させた。  
**なぜいけないか**: smoke test は最低限の機能保証であり、省略した場合は既知のバグを本番に出荷することになる。  
**物理 enforce 機構**: release pipeline の smoke test job は skip 不能フラグが設定されており、job 失敗時は pipeline が stopped となり deploy job は実行されない。  
**正しい対処**: → [03_tier3担当者シナリオ/10_pre_release_smoke_test] の必須実行手順を参照。

---

## infra 担当者 アンチパターン

### AP-infra-01: Kyverno policy を prod 適用前に validation しない
**状況**: 新規 Kyverno policy を staging で検証せずに直接 prod クラスタに apply したところ、全 pod の admission が拒否されてサービスがダウンした。  
**誤った行動**: `kubectl apply -f policy.yaml --context=prod` を staging 検証なしで実行。  
**なぜいけないか**: Kyverno policy のバグはクラスタ全体のデプロイを停止させる。staging で検証しない場合、全テナントへの影響が同時に発生する。  
**物理 enforce 機構**: GitOps pipeline は prod クラスタへの policy 適用前に staging での kyverno-test + dry-run が必須 gate として存在し、未通過の場合は prod branch へのマージがブロックされる。  
**正しい対処**: → [04_infra担当者シナリオ/04_Kyverno_admission_policy追加] の staging-first 手順を参照。

### AP-infra-02: secret rotation を手動で「後でやる」と先送り
**状況**: secret rotation の cadence (90 日) を超過したが、「現在 incident 対応中」として rotation を先送りにした。cadence 超過が 6 ヶ月継続した。  
**誤った行動**: rotation cadence を手動 calendar で管理しており、超過を検知するアラートがなかった。  
**なぜいけないか**: secret の長期固定化は侵害リスクの長期化を意味する。90 日を超えた secret は compromise の発覚遅延が最大化された状態となる。  
**物理 enforce 機構**: secret rotation cadence は External Secrets Operator の annotation で管理され、期限超過の secret が存在する場合に Prometheus alert が発報し on-call に通知される。  
**正しい対処**: → [04_infra担当者シナリオ/08_secret_rotation] の自動 cadence 管理手順を参照。

### AP-infra-03: cluster upgrade を drain なしで node 入替
**状況**: 時間短縮のため、pod eviction の待機を省略して node を強制削除した。結果として stateful pod のデータが消失した。  
**誤った行動**: `kubectl delete node worker-01 --force` を drain 完了前に実行。  
**なぜいけないか**: drain なしの node 削除は running pod を即時 kill するため、stateful workload の data loss や job の中断が発生する。  
**物理 enforce 機構**: cluster upgrade Runbook は Temporal workflow として実装され、drain 完了イベントを受信するまで次の node 削除ステップに進まない state machine enforcement。  
**正しい対処**: → [04_infra担当者シナリオ/01_cluster_upgrade] の drain-first 手順を参照。

### AP-infra-04: chaos drill 後の circuit breaker を手動解除せず放置
**状況**: chaos drill でわざと circuit breaker を open にしたが、drill 終了後のリセット手順を実行せず、翌日の業務開始時も circuit が open のままだった。  
**誤った行動**: drill completion checklist の「circuit breaker reset」項目を記入せずに drill を完了とした。  
**なぜいけないか**: circuit breaker が open のまま残ると、翌日の本番業務で無関係な request が全て拒否され、偽のサービス障害が発生する。  
**物理 enforce 機構**: chaos drill の Temporal workflow が drill completion time から 60 分後に自動 rollback script を実行し、circuit breaker state を検証する。異常状態が残存する場合は on-call へアラート。  
**正しい対処**: → [04_infra担当者シナリオ/06_Chaos_drill実行] の drill 後クリーンアップ手順を参照。

### AP-infra-05: GitOps で prod と staging の config を共有
**状況**: 設定の DRY を優先するため、prod と staging が同一の Kyverno policy ファイルを参照する構成にした。staging での policy 変更が即座に prod に反映され障害が発生した。  
**誤った行動**: `kustomization.yaml` で prod / staging が共通の base policy を上書きなしで参照。  
**なぜいけないか**: prod と staging の policy 境界がなくなると、staging での実験的変更が prod に混入する。変更の段階的適用という GitOps の原則が破壊される。  
**物理 enforce 機構**: GitOps repo の directory 構造 lint (Conftest) が prod / staging の設定ファイルが分離されているかを確認し、共有参照を検出した場合に CI を failed とする。  
**正しい対処**: → [04_infra担当者シナリオ/05_GitOps配信_progressive_delivery] の environment 分離手順を参照。

---

## data 担当者 アンチパターン

### AP-data-01: rollback 不可 migration を本番適用
**状況**: NOT NULL 制約を追加する migration を rollback スクリプトなしで本番に適用したところ、アプリケーションのバグで constraint violation が多発し、migration を元に戻せなくなった。  
**誤った行動**: migration ファイルに down() メソッドを実装せず、up() のみで PR をマージ。  
**なぜいけないか**: rollback 不可の migration は障害発生時の recovery 手段を奪う。特に constraint 追加は既存データとの整合性によって runtime error を引き起こす可能性がある。  
**物理 enforce 機構**: migration lint (Flyway + custom script) が down() スクリプトの存在を確認し、未定義の場合は CI を failed とする。  
**正しい対処**: → [05_data担当者シナリオ/01_schema_migration] の reversible migration 実装要件を参照。

### AP-data-02: preserve_class を変更せずにデータを削除
**状況**: ディスク容量不足への緊急対応として、preserve_class = permanent のデータをクラス変更なしに DELETE した。  
**誤った行動**: `DELETE FROM records WHERE created_at < '2020-01-01'` を preservation_class 確認なしで実行。  
**なぜいけないか**: permanent class のデータには法定保存義務が設定されている可能性があり、無断削除は法令違反となる。また WORM バケットとの整合性が失われる。  
**物理 enforce 機構**: DELETE / UPDATE 文を発行する前に preservation_class を確認する DB trigger が存在し、permanent class への変更・削除は law firm approval token がない場合に ERROR を発生させる。  
**正しい対処**: → [05_data担当者シナリオ/02_preservation_class変更] のクラス変更手順を参照。

### AP-data-03: restore drill を本番データでのみ実施
**状況**: restore drill を「本番の実際のバックアップで検証したほうが確実」として、本番環境のデータを restore 先に使用した。  
**誤った行動**: 本番 DB を restore ターゲットとして指定し、既存データを上書きして drill を実施。  
**なぜいけないか**: restore drill は誤った場合に本番データを破壊する操作であり、staging / DR 環境でのみ実施すべき。本番データへの上書きは PII 漏洩・業務停止を引き起こす。  
**物理 enforce 機構**: restore drill の Temporal workflow は接続先 DB URL が prod パターンに一致する場合に workflow を中断し、ops / security へアラートを送信する guard step を含む。  
**正しい対処**: → [05_data担当者シナリオ/03_restore_drill] の DR 環境を使った drill 手順を参照。

### AP-data-04: DEK rotation 中に KEK rotation を並行実行
**状況**: セキュリティ強化の一環として、DEK rotation と KEK rotation を同時に開始した。途中で DEK が KEK で復号できなくなり、データが一時的にアクセス不能となった。  
**誤った行動**: `rotate_kek()` と `rotate_dek()` を並列 Temporal workflow として同時起動。  
**なぜいけないか**: DEK は KEK で暗号化されているため、KEK rotation 中に DEK rotation を実行すると復号失敗が発生する。この二つの操作は厳密に sequential であることが要件。  
**物理 enforce 機構**: KEK rotation workflow が active の間は DEK rotation workflow の start が mutex lock で blocked される。  
**正しい対処**: → [05_data担当者シナリオ/05_暗号化変更_KEK_DEK_rotation] のシーケンス手順を参照。

### AP-data-05: split_brain 状態で両ノードを primary として運用
**状況**: レプリケーション lag が発生し split_brain が疑われる状況で、ネットワーク担当者が「とりあえず両方 primary として動かしてデータ収集を続ける」と判断した。  
**誤った行動**: Patroni の failover をトリガーせず、両ノードが primary 状態のまま write を継続。  
**なぜいけないか**: dual primary による write は同一レコードへの競合 write を発生させ、後での merge が不可能なデータ divergence を生む。  
**物理 enforce 機構**: Patroni の DCS (etcd) が leader election を強制し、split_brain 検出時に旧 primary を自動 demote する。手動で dual primary を維持することは設計上不可能。  
**正しい対処**: → [05_data担当者シナリオ/06_replication_lag_split_brain対応] の Patroni failover 手順を参照。

---

## security 担当者 アンチパターン

### AP-security-01: 625 cell unreachable を理由なし省略
**状況**: threat model の 625 cell マトリクスのうち、攻撃困難と思われる 30 cell を「unreachable」として理由記載なしに skip した。  
**誤った行動**: threat model spreadsheet の 30 行を「N/A」と入力してレビューを完了とした。  
**なぜいけないか**: 理由なしの unreachable 判定は、将来の構成変更によって「unreachable」でなくなった際に見落とされる。また監査時に「見逃し」との区別ができない。  
**物理 enforce 機構**: threat model lint script が unreachable 判定されたすべての cell に `reason:` フィールドの記入を要求し、空欄がある場合は monthly review の merge をブロックする。  
**正しい対処**: → [06_security担当者シナリオ/01_threat_model_625cell月次レビュー] の unreachable 記入要件を参照。

### AP-security-02: KEK rotation 中に DEK rotation を並行実行
(data アンチパターン AP-data-04 の security 視点)

**状況**: security 担当者が quarterly KEK ceremony を実行中に、data 担当者が独立に DEK rotation を開始した。coordination なしに並行実行された。  
**誤った行動**: security / data 担当者間のコーディネーション手順を省略して独立に rotation を実行。  
**なぜいけないか**: KEK/DEK の rotation はシーケンシャルである必要があり、並行実行はデータ復号不能を引き起こす。  
**物理 enforce 機構**: KEK rotation workflow の active 状態は外部から参照可能なフラグとして公開され、DEK rotation の起動前チェックで確認される。  
**正しい対処**: → [06_security担当者シナリオ/06_secret_compromise_drill_KEK_shamir_ceremony] の coordination 手順を参照。

### AP-security-03: red team 結果を修正なしに「参考情報」として保管
**状況**: red team table-top で発覚した 5 件の脆弱性を「重要度低」として threat model に記録のみ行い、修正 ticket を作成しなかった。  
**誤った行動**: red team report を JIRA に attach するのみで remediation plan を作成しなかった。  
**なぜいけないか**: red team の発見事項は remediation plan なしでは改善につながらない。「参考情報」として放置すると次回 red team で同一の脆弱性が再発見される。  
**物理 enforce 機構**: red team report の提出フォームは `remediation_issue_url` フィールドが必須であり、未入力では report を closed にできない。  
**正しい対処**: → [06_security担当者シナリオ/03_red_team_table_top] の remediation plan 作成手順を参照。

### AP-security-04: CVE CRITICAL への対応を「次回スプリント」に延期
**状況**: CVE CRITICAL が通知されたが、スプリント計画を変更したくないとして「次回スプリントで対応」と判断し 14 日後に対応した。  
**誤った行動**: CVE CRITICAL を通常のバックログに追加して sprint planning まで放置。  
**なぜいけないか**: CVE CRITICAL は exploit が公開されると数時間でスキャンされる。14 日間は攻撃者に十分な機会を与える。SLA 違反にもなる。  
**物理 enforce 機構**: CVE CRITICAL 検出時に automated issue が作成され、72 時間以内に `fixed` または `mitigated` に遷移しない場合は on-call へ escalation alert が送信される。  
**正しい対処**: → [06_security担当者シナリオ/07_CVE_CRITICAL_incident対応] の緊急対応 SLA を参照。

### AP-security-05: cosign 検証エラーを「false positive」として admission を手動許可
**状況**: CI/CD pipeline で cosign 署名検証が失敗したが、「環境の問題」と判断して Kyverno の admission policy を一時的に disable にしてデプロイを継続した。  
**誤った行動**: `kubectl annotate policy cosign-verify disabled=true` を prod クラスタに適用してデプロイを通過させた。  
**なぜいけないか**: cosign 検証失敗は supply chain 攻撃の可能性を示すシグナルである。「false positive」と判断せずに原因究明が先決。policy の無効化は供給チェーン全体の保護を破壊する。  
**物理 enforce 機構**: cosign policy の disable は 2 名の security engineer の approval + incident ticket 番号が必要な変更であり、単独での disable は admission に拒否される RBAC 設定。  
**正しい対処**: → [06_security担当者シナリオ/08_cosign_supply_chain_admission拒否対応] の原因究明手順を参照。
