-- ============================================================
-- migration 0005_pgaudit_config.sql: pgaudit 詳細設定
-- オブジェクト監査・セッション監査の粒度を定義する
-- PostgreSQL 16 互換
-- ============================================================

-- search_path をアプリ専用スキーマに固定
SET search_path = k1s0, public;

-- ============================================================
-- pgaudit オブジェクト監査ロールの設定
-- 特定テーブルへのアクセスを監査対象として登録する
-- ============================================================

-- 監査対象オブジェクトを管理する専用ロールを作成
-- このロールに権限を付与することで pgaudit のオブジェクト監査が有効になる
DO $$
BEGIN
    -- audit_role が存在しない場合のみ作成（冪等性を保証）
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'k1s0_audit_role') THEN
        -- 監査専用ロールを作成（ログイン不可の純粋な権限管理ロール）
        CREATE ROLE k1s0_audit_role NOLOGIN;
    END IF;
END;
$$;

-- pgaudit オブジェクト監査の対象ロールとして k1s0_audit_role を指定
-- このロールが保持する権限に対するアクセスが監査される
ALTER ROLE k1s0app SET pgaudit.role = 'k1s0_audit_role';

-- ============================================================
-- 高リスクテーブルへのオブジェクト監査権限付与
-- audit_role への SELECT/UPDATE 権限付与により監査ログが生成される
-- ============================================================

-- domain_event テーブルへの全操作を audit_role 経由で監査
GRANT SELECT, INSERT ON k1s0.domain_event TO k1s0_audit_role;

-- state_store テーブルへの全操作を audit_role 経由で監査
GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.state_store TO k1s0_audit_role;

-- audit_event テーブルへのアクセスを audit_role 経由で監査
GRANT SELECT, INSERT ON k1s0.audit_event TO k1s0_audit_role;

-- outbox_message テーブルへの全操作を audit_role 経由で監査
GRANT SELECT, INSERT, UPDATE, DELETE ON k1s0.outbox_message TO k1s0_audit_role;

-- ============================================================
-- pgaudit セッション監査の詳細設定
-- GUC パラメータをアプリケーションレベルで上書き設定する
-- ============================================================

-- k1s0app ロールのセッション監査設定: write・ddl・role を対象に指定
ALTER ROLE k1s0app SET pgaudit.log = 'write,ddl,role';

-- k1s0app ロールのシステムカタログ監査を無効化（ノイズ削減）
ALTER ROLE k1s0app SET pgaudit.log_catalog = 'off';

-- k1s0app ロールの監査ログにリレーション情報を含める
ALTER ROLE k1s0app SET pgaudit.log_relation = 'on';

-- k1s0app ロールの監査ログにパラメータバインディングを記録
ALTER ROLE k1s0app SET pgaudit.log_parameter = 'on';

-- ============================================================
-- 監査設定の確認クエリ（CI スモークテスト用）
-- ============================================================

-- pgaudit 拡張が正常にインストールされているかを確認する
DO $$
DECLARE
    -- pgaudit の拡張バージョンを格納する変数
    v_pgaudit_version TEXT;
BEGIN
    -- pg_extension から pgaudit のバージョンを取得
    SELECT extversion
    INTO v_pgaudit_version
    FROM pg_extension
    WHERE extname = 'pgaudit';

    -- pgaudit が見つからない場合はエラーを発生させる
    IF v_pgaudit_version IS NULL THEN
        RAISE EXCEPTION 'pgaudit 拡張がインストールされていません';
    END IF;

    -- 検証成功メッセージを出力
    RAISE NOTICE 'pgaudit バージョン % が正常に設定されました', v_pgaudit_version;
END;
$$;
