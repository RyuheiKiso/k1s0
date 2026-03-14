import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app/router.dart';
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
      child: const InventoryApp(),
    ),
  );
}

/// 在庫管理アプリケーションのルートウィジェット
/// Material Designテーマとgo_routerによるルーティングを設定する
class InventoryApp extends ConsumerWidget {
  const InventoryApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: '在庫管理',
      theme: ThemeData(
        colorSchemeSeed: Colors.teal,
        useMaterial3: true,
      ),
      /// go_routerの設定をMaterialAppに適用する
      routerConfig: router,
      debugShowCheckedModeBanner: false,
    );
  }
}
