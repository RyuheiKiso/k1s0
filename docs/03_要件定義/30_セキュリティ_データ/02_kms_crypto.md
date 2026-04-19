# SEC-KMS: 鍵管理と暗号要件

本ファイルは、k1s0 プラットフォームにおける**暗号鍵のライフサイクル（生成・保管・利用・ローテ・廃棄）と承認暗号アルゴリズム**を定義する。鍵管理は「暗号化しているか」よりも「鍵がどこでどう管理されているか」の方が監査で厳しく問われる領域であり、実装だけでなく運用手順が受け入れ基準の核になる。

k1s0 は tier1 自作領域の `crypto` モジュール（Rust）と OpenBao（オープンソース化された HashiCorp Vault フォーク）を組み合わせて鍵管理を担う。自作領域でハッシュチェーン・署名・暗号化を直接扱う以上、「承認されたアルゴリズム以外は採番拒否」「鍵は OpenBao Transit Engine 経由でのみ利用」「90 日ローテ」の 3 点を全要件の前提に置く。

本要件群は改正個人情報保護法 第 23 条（安全管理措置）、FISC 安対基準 準則 7.4（暗号鍵の管理）、電子帳簿保存法施行規則 第 3 条（真実性確保）の 3 系統の要求を横断的に満たす。受け入れ基準は**検知 / 防御 / 復旧**の 3 軸で構成する。

---

## 前提

- [`../../02_構想設計/03_技術選定/`](../../02_構想設計/03_技術選定/) OpenBao 採用
- [`../10_アーキテクチャ/01_tier1.md`](../10_アーキテクチャ/01_tier1.md) tier1 自作領域（crypto モジュール）
- [`../00_共通/00_glossary.md`](../00_共通/00_glossary.md) COM-GLO-004（自作領域）
- [`01_IAM.md`](./01_IAM.md) SEC-IAM-003（サービスアカウント鍵の管理）
- [`03_audit.md`](./03_audit.md) SEC-AUD-002（ハッシュチェーン署名鍵）

---

## 要件本体

### SEC-KMS-001: 鍵管理基盤として OpenBao を採用する

- 優先度: MUST（鍵の一元管理がないと監査応答で即座に不適合判定）
- Phase: Phase 1a
- 関連: `COM-CON-004`（ライセンス、OpenBao は MPL-2.0）、`SEC-IAM-003`

現状、JTC のプロジェクトでは鍵が Kubernetes Secret・環境変数・独自 DB テーブルに分散保管されるケースが多く、鍵の全数把握すら困難である。HashiCorp Vault の BSL ライセンス変更（2023 年）以降、商用利用で追加ライセンス費用の懸念もある。

本要件が満たされた世界では、全鍵は OpenBao（Vault の OSS フォーク、MPL-2.0）で一元管理され、API 経由でのみ取得・利用できる。Transit Secrets Engine による「鍵を持ち出さず暗号化を委譲する」運用で、アプリコードに鍵が平文で渡ることがない。

崩れた場合、鍵の所在不明・退職者による鍵持ち出し・鍵漏洩時の影響範囲特定困難などが発生し、FISC 安対基準 準則 7.4 に抵触する。

**受け入れ基準**

- 検知: `kubectl get secret --all-namespaces` で OpenBao 経由以外の長寿命 Secret を日次スキャン
- 防御: アプリから鍵への直接アクセスは OpenBao の Policy で拒否、Transit Engine API のみ許可
- 防御: OpenBao 自身は Shamir 秘密分散で Unseal、鍵管理者 3 名以上で閾値復号
- 復旧: OpenBao 障害時の緊急手順（break-glass Unseal）を Runbook 化、24 時間以内に RTO 達成

**検証方法**

- Phase 1a POC で鍵を OpenBao 経由でのみ取得する tier1 コードの疎通確認
- OpenBao Audit Log を `SEC-AUD-*` に連携

---

### SEC-KMS-002: 承認された暗号アルゴリズムのみを採用する

- 優先度: MUST（非承認アルゴリズムは改正個人情報保護法 ガイドライン「安全管理措置」解説で不適合）
- Phase: Phase 1a
- 関連: `SEC-AUD-002`、`SEC-SEC-001`

現状、開発者が「独自の暗号化」や古いアルゴリズム（MD5・SHA-1・3DES）を使ってしまう事故が残存し、NIST SP 800-131A でも非推奨化されたアルゴリズムが JTC システムで稼働している。

本要件が満たされた世界では、以下の承認アルゴリズムのみが tier1 `crypto` モジュールで公開され、それ以外のアルゴリズム呼び出しはコンパイルエラーになる。NIST FIPS 140-3 / CRYPTREC 電子政府推奨暗号リストの双方に整合する。

- 対称暗号: AES-256-GCM（AEAD、認証付き暗号）
- 公開鍵暗号: RSA-4096（鍵交換は RSA-OAEP）
- 署名: Ed25519（短い・高速・簡素）/ RSA-PSS SHA-256（互換用途）
- ハッシュ: SHA-256 / SHA-512（SHA-3 は将来要件）
- 鍵導出: HKDF-SHA-256 / Argon2id（パスワード）

崩れた場合、監査人が「脆弱な暗号が使われている」と指摘し、全データの再暗号化を要する事態となる（PostgreSQL 数 TB の再暗号化は数日〜数週間のダウンタイム）。

**受け入れ基準**

- 検知: `cargo deny` と SBOM スキャンで非承認暗号ライブラリの混入を検出
- 防御: tier1 `crypto` モジュールの公開 API は上記アルゴリズムに限定、それ以外は `pub(crate)` 以下
- 防御: 新規アルゴリズム追加は ADR 起票を必須とする
- 復旧: 脆弱化が発覚した場合のアルゴリズム切替手順（Transit Engine の key rewrap）を Runbook 化

**検証方法**

- Phase 1a で tier1 `crypto` の API 一覧を自動テストで検証し、許容外のアルゴリズムが `pub` でないことを保証

---

### SEC-KMS-003: 鍵ローテーション周期を 90 日以下に設定

- 優先度: MUST（FISC 安対基準 準則 7.4「暗号鍵は定期的に更新する」の具体化）
- Phase: Phase 1a
- 関連: `SEC-IAM-003`、`SEC-KMS-001`

現状、一度作った鍵が数年間同じまま使われる運用が大半で、鍵漏洩時の影響期間が線形に拡大する。

本要件が満たされた世界では、全鍵が 90 日以下で自動ローテされる。Transit Engine の key versioning により、旧バージョンの鍵で暗号化されたデータも透過的に復号可能（`min_decryption_version` で古いデータもサポート、`min_encryption_version` で新規書き込みは最新鍵に強制）。

崩れた場合、監査で「鍵ローテ運用がない」と指摘され、FISC 安対基準・ISMAP 登録審査で不適合判定を受ける。

**受け入れ基準**

- 検知: OpenBao の key version age を監視、90 日超えは Alert
- 防御: 自動ローテ cronjob が 89 日目に新バージョンを発行、`min_encryption_version` を更新
- 防御: ローテ履歴は `SEC-AUD-*` に記録、改ざん検知対象
- 復旧: ローテ失敗時の手動再実行手順を Runbook 化、24 時間以内に達成

**検証方法**

- 全鍵の最新 rotate_at タイムスタンプを Grafana で dashboard 化
- 四半期ごとにローテ実績を監査レポート化

---

### SEC-KMS-004: データ暗号化は Transit Secrets Engine 経由で行う

- 優先度: MUST（鍵を持ち出さない運用を構造的に強制）
- Phase: Phase 1a
- 関連: `SEC-KMS-001`、`SEC-DAT-*`

現状、暗号化するたびに鍵をアプリに取得させる「Envelope 暗号化」の自前実装では、鍵がメモリダンプで漏洩する経路が残る。

本要件が満たされた世界では、アプリは「暗号化してほしい平文」を OpenBao の Transit API (`/transit/encrypt/:name`) に渡し、結果の ciphertext（`vault:v1:...` 形式）を受け取る。鍵自体はアプリに一切渡らない。復号も同じく API 経由。

崩れた場合、アプリメモリからの鍵窃取経路が残り、侵害時の影響が鍵 lifetime 分拡大する。

**受け入れ基準**

- 検知: アプリが鍵 material を直接取得する API 呼び出しを監査ログで検知、アラート
- 防御: アプリに付与する OpenBao Policy は Transit `encrypt` / `decrypt` / `rewrap` のみ、`read` は拒否
- 防御: 大量データ暗号化は Transit の Datakey 発行（DEK）+ KEK で Envelope、ただし DEK の保管も暗号化状態
- 復旧: Transit エンドポイント障害時、メッセージキューでの暗号化要求のバッファリング（最大 15 分）

**検証方法**

- アプリから OpenBao への API 呼び出しトレースを Loki でフィルタし、想定外の endpoint が呼ばれていないか四半期レビュー

---

### SEC-KMS-005: HSM 連携を将来要件として用意する

- 優先度: SHOULD（Phase 2 の金融顧客向け。Phase 1 では OpenBao の AutoUnseal で代替）
- Phase: Phase 2
- 関連: `SEC-KMS-001`、`BIZ-ONB-*`

現状、FISC 安対基準 準則 7.4 の厳格解釈（金融機関向け）では、マスターキーの保護に HSM (Hardware Security Module) を要求するケースがある。ソフトウェア鍵管理のみでは金融顧客の稟議が通らない場合がある。

本要件が満たされた世界では、OpenBao の Auto Unseal を AWS KMS CloudHSM / 国産 Cloud HSM（さくら / IIJ）/ Thales Luna の PKCS#11 で実装でき、マスターキーが物理 HSM で保護される。FIPS 140-2 Level 3 準拠の監査応答が可能になる。

崩れた場合、金融顧客・公共セクター顧客の獲得が Phase 2 以降で困難になる。

**受け入れ基準**

- 検知: Phase 2 に向けて HSM 対応要否を顧客セグメント別に判定する仕組みを Phase 1b で整備
- 防御: OpenBao PKCS#11 Seal 設定の検証環境を Phase 1c に構築
- 防御: HSM 接続失敗時の縮退運転モード（SW Unseal にフォールバック）を禁止、完全停止を選択
- 復旧: HSM 障害時の現場対応手順を HSM ベンダと合意

**検証方法**

- Phase 2 着手前に POC で OpenBao + HSM の疎通確認

---

### SEC-KMS-006: 鍵廃棄（Crypto-shredding）の手順を定義する

- 優先度: MUST（GDPR 17 条「忘れられる権利」対応、および改正個人情報保護法 第 30 条の利用停止等請求）
- Phase: Phase 1b
- 関連: `SEC-PRV-004`、`SEC-BCK-*`

現状、削除要求に応えて物理的にデータ削除する実装は、バックアップ・レプリカ・ログに残るデータまで含めると困難を極める。

本要件が満たされた世界では、テナント別・利用者別に発行した DEK（Data Encryption Key）の廃棄で、バックアップ含む全データが実効的に復号不能になる（crypto-shredding）。改正個人情報保護法 第 30 条の利用停止等請求に対して 2 週間以内に実効的削除を完了できる。

崩れた場合、個人情報保護委員会への対応期限（通常 2 週間）を超過し、行政処分のリスクが発生する。

**受け入れ基準**

- 検知: 鍵廃棄イベントは `SEC-AUD-*` に記録、廃棄完了まで追跡
- 防御: 廃棄は Transit の `trim` + `delete_key`、最低 2 名の承認必須
- 防御: 誤削除防止のため `deletion_allowed: true` のフラグ付与から 24 時間の cooldown
- 復旧: 誤廃棄検知時は 24 時間以内なら delete flag の取り消しで復旧可能

**検証方法**

- 疑似データで crypto-shredding を実施、バックアップから復号不能となることを Phase 1b POC で検証

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| SEC-KMS-001 | OpenBao 鍵管理基盤 | MUST | 1a |
| SEC-KMS-002 | 承認暗号アルゴリズム | MUST | 1a |
| SEC-KMS-003 | 90 日ローテ | MUST | 1a |
| SEC-KMS-004 | Transit Engine 経由暗号化 | MUST | 1a |
| SEC-KMS-005 | HSM 将来要件 | SHOULD | 2 |
| SEC-KMS-006 | Crypto-shredding | MUST | 1b |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 5 | SEC-KMS-001, 002, 003, 004, 006 |
| SHOULD | 1 | SEC-KMS-005 |

### Phase 達成度

| Phase | 必達件数 | 未達影響 |
|---|---|---|
| 1a | 4 | FISC 安対基準 準則 7.4 違反、稟議通過不可 |
| 1b | 1 | 改正個人情報保護法 第 30 条対応不可 |
| 2 | 1 | 金融・公共顧客向けの HSM 要件 |
