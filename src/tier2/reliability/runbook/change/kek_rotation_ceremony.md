# KEK ローテーションセレモニー 変更管理手順書

<!-- docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §暗号鍵管理 -->
<!-- docs 参照: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md §KEK ローテーション -->
<!-- 緊急ローテーション手順: src/tier2/reliability/runbook/break_glass/emergency_key_rotation.md -->

## Classification

<!-- Change の種別と対象範囲の宣言 -->
- change_class: kek_rotation_ceremony
- rotation_cadence: 180 日ごとに実施する（定期ローテーション）
- change_type: scheduled (通常手順) / emergency (緊急手順は break_glass/ を参照)
- spec_reference: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md

## 前提条件と承認要件

<!-- セレモニー実施前に満たしておく必要がある条件 -->
1. 変更管理チケットが承認済みであること（Change Advisory Board の承認）
2. dual sign-off の参加者が 2 名以上確保されていること
   - 担当者 A: 実施責任者（操作を実行する）
   - 担当者 B: 承認者（操作内容を監査し cosign 署名を提供する）
3. Barman base backup が直近 24 時間以内に完了していること（ロールバック保証）
4. 全テナントのアクティビティが低い時間帯（メンテナンスウィンドウ）を選定すること
5. 影響テナントへのメンテナンス通知が送信済みであること（事前 72 時間以上）

## Phase 1: 事前確認 (Pre-Check)

<!-- セレモニー開始前の状態確認 -->
1. 現在の KEK バージョンと有効期限を確認する
   ```bash
   # 現在の KEK メタデータを取得する
   vault read transit/keys/k1s0-tier2-kek
   ```
2. OpenBao の Unseal 状態を確認する
   ```bash
   # OpenBao の状態を確認する（sealed でないこと）
   vault status
   ```
3. 全テナントのサービス稼働状態を確認する（メンテナンスモード移行前に正常稼働を確認）
4. 監視ダッシュボードで暗号化エラーレートが 0 であることを確認する
5. Mattermost `#k1s0-changes` にセレモニー開始を通知する

## Phase 2: メンテナンスモード移行 (Maintenance Window)

<!-- サービスをメンテナンスモードに移行する -->
1. tier2 API の新規リクエスト受付を停止する（graceful shutdown を開始する）
2. 進行中のトランザクションが完了するまで待機する（最大 5 分）
3. Outbox テーブルに未送信イベントが残っていないことを確認する
   ```sql
   -- Outbox の未送信件数を確認する（0 件であることを確認）
   SELECT count(*) FROM outbox_event WHERE status = 'PENDING';
   ```
4. メンテナンス開始時刻と開始時の audit_event chain_sequence を記録する

## Phase 3: KEK ローテーション実施 (Key Rotation)

<!-- OpenBao Transit で KEK をローテーションする -->
1. 担当者 A が OpenBao Transit で新 KEK バージョンを生成する
   ```bash
   # Transit key を rotate して新バージョンを生成する
   # この操作は担当者 A の認証情報で実行し audit に記録される
   vault write -f transit/keys/k1s0-tier2-kek/rotate
   ```
2. 担当者 B が新 KEK バージョンのフィンガープリントを独立して確認する
   ```bash
   # 新 KEK バージョンのメタデータを取得して確認する
   vault read transit/keys/k1s0-tier2-kek
   ```
3. 担当者 A と担当者 B がそれぞれ cosign で新 KEK バージョンに署名する（dual sign-off）
   ```bash
   # 担当者 A: 新 KEK バージョンに cosign 署名する
   echo "kek-version-$(vault read -field=latest_version transit/keys/k1s0-tier2-kek)" \
     | cosign sign-blob --key cosign_a.key --bundle kek_rotation_bundle_a.json -

   # 担当者 B: 独立して cosign 署名する
   echo "kek-version-$(vault read -field=latest_version transit/keys/k1s0-tier2-kek)" \
     | cosign sign-blob --key cosign_b.key --bundle kek_rotation_bundle_b.json -
   ```
4. 旧 KEK で暗号化されたデータを新 KEK で rewrap する
   ```bash
   # tier2 lifecycle-worker の rewrap コマンドで全暗号化データを移行する
   kubectl -n k1s0-tier2 run kek-rewrap --image=harbor.k1s0.internal/k1s0/lifecycle-worker:latest \
     --restart=Never -- /app/lifecycle-worker kek-rewrap --new-version <NEW_VERSION>
   ```
5. minimum_decryption_version を新バージョンに更新して旧バージョンの使用を禁止する

## Phase 4: データ整合性確認と復旧 (Verification & Recovery)

<!-- ローテーション後の確認とサービス再開 -->
1. rewrap 後のデータが正常に復号できることを確認する（サンプルデータで検証）
2. audit_event の hash chain が連続していることを確認する
3. tier2 API を再起動して新 KEK バージョンを読み込む
4. サービスをメンテナンスモードから解除して通常運用に戻す
5. 全テナントの暗号化・復号操作が正常であることを監視ダッシュボードで確認する

## Phase 5: 証跡保管と記録 (Evidence)

<!-- ローテーション完了後の証跡管理 -->
1. cosign bundle（kek_rotation_bundle_a.json / kek_rotation_bundle_b.json）を長期保管庫に転送する
2. Sigstore の rekor transparency log にエントリが記録されていることを確認する
3. 変更管理チケットにローテーション完了を記録し承認者に通知する
4. 次回ローテーション予定日（180 日後）を更新する
5. セレモニー参加者・実施日時・新旧 KEK バージョンを audit ログに追記する
