import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app/app.dart';
import 'config/app_config.dart';
import 'config/config_provider.dart';

/// アプリケーションのエントリポイント
/// YAML設定ファイルを読み込んでからProviderScopeにoverrideして起動する
void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  /// 環境変数APP_ENVから実行環境を取得する（デフォルトはdevelopment）
  const env = String.fromEnvironment('APP_ENV', defaultValue: 'development');

  /// YAML設定ファイルを読み込む
  final config = await AppConfig.load(env);

  runApp(
    ProviderScope(
      overrides: [appConfigProvider.overrideWithValue(config)],
      child: const ProjectMasterApp(),
    ),
  );
}
