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
      child: const DomainMasterApp(),
    ),
  );
}

/// ドメインマスタアプリケーションのルートウィジェット
/// Material Designテーマとgo_routerによるルーティングを設定する
class DomainMasterApp extends ConsumerWidget {
  const DomainMasterApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'ドメインマスタ管理',
      theme: ThemeData(
        colorSchemeSeed: Colors.indigo,
        useMaterial3: true,
      ),
      /// go_routerの設定をMaterialAppに適用する
      routerConfig: router,
      debugShowCheckedModeBanner: false,
    );
  }
}
