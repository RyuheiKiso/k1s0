-- 010 のロールバック: publish_failed 関連の SECURITY DEFINER 関数を削除する

DROP FUNCTION IF EXISTS eventstore.list_publish_failed_events(integer);
DROP FUNCTION IF EXISTS eventstore.count_publish_failed_all_tenants();
