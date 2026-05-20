# PITR Restore Drill 手順書

<!-- docs 参照: docs/03_概要設計/03_tier2設計方針/README.md §信頼性・復旧 -->
<!-- docs 参照: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md §RTO/RPO -->
<!-- argo workflow 参照: src/tier2/reliability/restore_drill_workflow.yaml -->

## Classification

<!-- Drill の種別と対象範囲の宣言 -->
- drill_class: pitr_restore_drill
- drill_cadence: 90 日ごとに実施する（restore_drill_workflow.yaml の drill_cadence_days と対応）
- target_rto: 4 時間以内（spec 要件: RTO ≤ 4h）
- target_rpo: 1 時間以内（spec 要件: RPO ≤ 1h）
- spec_reference: docs/04_詳細設計/01_適合仕様/10_テナント分離適合仕様.md

## 前提条件

<!-- このDrillを実行する前に満たしておく必要がある条件 -->
1. staging 環境が利用可能であること（本番データを staging に復元するため）
2. Barman の最新 base backup と WAL アーカイブが到達可能であること
3. drill 実施の 1 時間前に本番データのスナップショットが完了していること
4. drill 担当者が restore_drill_workflow.yaml の Argo Workflow 実行権限を持つこと
5. 監視ダッシュボードに drill_mode フラグを立て誤アラートを抑制すること

## Phase 1: Drill 準備 (Preparation)

<!-- Drill 開始前の確認と設定 -->
1. Argo Workflow の restore_drill_workflow.yaml をレビューして手順が最新であることを確認する
2. staging 環境の既存データをバックアップして restore 結果と比較できる状態にする
3. 復元対象のポイントインタイム（PITR target time）を決定する
   - 推奨: 現在時刻から 1 時間前の時刻を使用する（RPO 要件の境界値テスト）
4. drill 開始時刻と担当者を Mattermost `#k1s0-drills` に通知する
5. RTO 計測タイマーを開始する（drill 開始時刻を記録する）

## Phase 2: Barman からの復元 (Restore)

<!-- Barman WAL アーカイブを使用した PITR 復元手順 -->
1. Barman で指定ポイントインタイムの base backup を確認する
   ```bash
   # 指定時刻以前の最新 base backup を取得する
   barman list-backups k1s0-tier2-primary
   ```
2. staging PostgreSQL のデータディレクトリを停止してクリアする
   ```bash
   # staging の PostgreSQL を停止する（本番には影響しない）
   pg_ctlcluster 16 staging stop
   # データディレクトリをクリアする（PITR 復元前に必要）
   rm -rf /var/lib/postgresql/16/staging/*
   ```
3. Barman の `recover` コマンドで staging に復元する
   ```bash
   # PITR ターゲット時刻を指定して復元する（HLC ではなく PostgreSQL WAL の壁時計を使う）
   barman recover \
     --target-time "<PITR_TARGET_TIME>" \
     --remote-ssh-command "ssh postgres@staging" \
     k1s0-tier2-primary latest /var/lib/postgresql/16/staging
   ```
4. staging PostgreSQL を起動し recovery が完了するまで待機する

## Phase 3: データ整合性検証 (Verification)

<!-- 復元後のデータ整合性を確認する -->
1. 復元された PostgreSQL で全テナントの RLS が有効であることを確認する
   ```sql
   -- RLS が全テーブルで有効であることを確認する
   SELECT schemaname, tablename, rowsecurity
   FROM pg_tables
   WHERE schemaname = 'k1s0_tier2'
   ORDER BY tablename;
   ```
2. audit_event テーブルの hash chain が連続していることを確認する
   ```sql
   -- hash chain の連続性を確認する（欠番・重複がないこと）
   SELECT chain_sequence,
          lag(chain_sequence) OVER (PARTITION BY tenant_id ORDER BY chain_sequence) AS prev_seq,
          chain_sequence - lag(chain_sequence) OVER (PARTITION BY tenant_id ORDER BY chain_sequence) AS diff
   FROM audit_event
   WHERE tenant_id = '<test_tenant>'
   ORDER BY chain_sequence
   LIMIT 100;
   ```
3. テナント分離の境界を確認する（テナント A のデータがテナント B から見えないこと）
4. Pact contract test を staging 環境で実行して API 互換性を確認する
5. `tools/lock_yaml_generator/generate_audit_root_hash.py` で audit hash root を再計算して一致を確認する

## Phase 4: RTO/RPO 測定と記録 (Measurement)

<!-- Drill の結果を測定して記録する -->
1. RTO タイマーを停止し実際の復元所要時間を記録する
   - 目標: 4 時間以内
   - 実績: （記録する）
2. 復元されたデータの最新タイムスタンプと PITR ターゲット時刻の差を RPO として記録する
   - 目標: 1 時間以内
   - 実績: （記録する）
3. restore_drill_workflow.yaml の `record-metrics` ステップの出力と照合する
4. 目標未達成の場合は次の Drill までに改善計画を立案する

## Phase 5: Drill 結果の報告とクリーンアップ

<!-- Drill 完了後の後処理 -->
1. 監視ダッシュボードの drill_mode フラグを解除する
2. staging 環境のデータをリセットして通常の開発用途に戻す
3. Drill 結果レポートを作成し `#k1s0-drills` に投稿する
4. RTO/RPO が目標値を下回った場合は postmortem_template.md に従って報告書を作成する
5. 次回 Drill 予定日（90 日後）を calendar に登録する
