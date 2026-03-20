import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app_config.dart';

/// アプリケーション設定を提供するプロバイダー
/// main()でoverrideして実際の設定値を注入する
final appConfigProvider = Provider<AppConfig>((ref) {
  throw UnimplementedError('main() で override すること');
});
