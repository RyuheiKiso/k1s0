# 鍵緊急ローテーション Runbook (Break Glass)

<!-- docs 参照: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md §緊急鍵管理 -->
<!-- docs 参照: docs/04_詳細設計/02_強制機構/02_tier2強制機構.md §OpenBao Transit -->

## Classification

<!-- インシデントクラスと重大度の宣言 -->
- incident_class: emergency_key_rotation
- severity: P0 (page_immediate) — 鍵漏洩疑いまたは Cosign 証跡の破損は最高重大度
- spec_reference: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md

## 前提条件

<!-- このRunbook を実行する前に満たしておく必要がある条件 -->
1. break-glass アクセス権限を持つ担当者が 2 名以上確保されていること（dual sign-off 要件）
2. OpenBao の Unseal 手続きが完了していること（Unseal keys は物理的に分離保管）
3. Backstage emergency_override plugin にアクセスできること（または kubectl の直接アクセス権）
4. Audit ログの書き込みが正常であることを確認すること（audit chain が健全であること）

## Phase 1: 異常検知と判断 (Detection)

<!-- 鍵緊急ローテーションが必要と判断するためのチェックリスト -->
1. OpenBao の Prometheus アラート `VaultSealStatusCritical` または `TransitKeyLeakSuspected` を確認する
2. Sigstore の transparency log で KEK の cosign 証跡に異常がないか確認する
3. Harbor の admission policy reject log で不審な署名なしイメージのデプロイ試行を確認する
4. audit_event ClickHouse で直近 1 時間の `action=key_access` を集計し異常アクセスを特定する
   ```sql
   -- 直近 1 時間の鍵アクセス監査イベントを集計する
   SELECT actor_id, count(*) AS access_count, max(occurred_at) AS last_access
   FROM audit_event
   WHERE tenant_id = '<affected_tenant>'
     AND resource_type = 'kek'
     AND occurred_at > now() - INTERVAL 1 HOUR
   GROUP BY actor_id
   ORDER BY access_count DESC
   LIMIT 50
   ```
5. インシデント対応チャンネル（Mattermost `#k1s0-incident`）に発報し dual sign-off の承認者を招集する

## Phase 2: 封じ込め (Containment)

<!-- 被害拡大を防ぐための初動措置 -->
1. 影響を受けた KEK の使用を一時停止する（OpenBao policy で deny all に変更する）
   ```bash
   # OpenBao の KEK ポリシーを緊急 deny に変更する
   # NOTE: この操作は Audit ログに記録される
   vault policy write k1s0-kek-emergency - <<'EOF'
   # 緊急時: 全 path を deny にして鍵アクセスを遮断する
   path "transit/*" {
     capabilities = ["deny"]
   }
   EOF
   ```
2. 影響テナントへの新規認証要求を一時停止する（tier2 API の circuit breaker を trip させる）
3. 影響範囲（どのデータが当該 KEK で暗号化されているか）をインベントリで確認する
4. dual sign-off の承認者 2 名の立会いを確認してから次の Phase に進む

## Phase 3: 新 KEK の生成とローテーション (Key Rotation)

<!-- OpenBao Transit で新しい KEK を生成し古い KEK からの移行を実施する -->
1. OpenBao Transit で新しい KEK バージョンを生成する
   ```bash
   # Transit key の rotate（新バージョンを生成する）
   # 操作担当者 ID と承認者 ID を環境変数に設定すること
   vault write -f transit/keys/k1s0-tier2-kek/rotate
   ```
2. 新しい KEK バージョンを Sigstore で cosign 署名する
   ```bash
   # cosign で新 KEK のフィンガープリントに署名する（dual sign-off の物理証跡）
   cosign sign --key k1s0-cosign.key \
     "vault:transit/k1s0-tier2-kek:$(vault read -field=latest_version transit/keys/k1s0-tier2-kek)"
   ```
3. 旧 KEK バージョンで暗号化されたデータを新 KEK バージョンに再暗号化する
   ```bash
   # rewrap コマンドで旧バージョン暗号化データを新バージョンに移行する
   vault write transit/rewrap/k1s0-tier2-kek ciphertext="<old_ciphertext>"
   ```
4. minimum_decryption_version を更新して旧 KEK バージョンの復号を禁止する

## Phase 4: データ整合性確認 (Verification)

<!-- ローテーション後のデータ整合性を確認する -->
1. 全テナントのデータ復号テストを実行する（サンプルレコードの復号を確認する）
2. audit_event の hash chain が連続していることを確認する
3. Pact contract test を実行して tier2 API の動作に影響がないことを確認する
4. 監視ダッシュボードで暗号化エラーレートが 0 であることを確認する

## Phase 5: 復旧とサービス再開 (Recovery)

<!-- サービスを正常状態に戻す手順 -->
1. tier2 API の circuit breaker を reset してサービスを再開する
2. 影響テナントに対してサービス復旧通知を送信する（SLA 通知要件に従う）
3. OpenBao の通常ポリシーに戻す
4. 次の Phase に進む前に監視アラートが全て GREEN であることを確認する

## Phase 6: Postmortem と証跡保管

<!-- ローテーション完了後の事後処理 -->
1. インシデント対応記録を postmortem_template.md に従って作成する
2. cosign 署名済みの KEK ローテーション証跡を長期保管庫に転送する
3. dual sign-off の承認ログを audit_event に追記する
4. 次のローテーション予定日を kek_rotation_ceremony.md に記録する
