// system_client の app_config を再エクスポートする（重複排除）
// 公開 barrel ファイル経由でエクスポートし、src/ 内部への直接参照を避ける
export 'package:system_client/system_client.dart' show AppConfig, ApiConfig;
