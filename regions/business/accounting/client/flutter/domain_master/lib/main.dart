import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app/router.dart';

/// アプリケーションのエントリポイント
/// ProviderScopeでRiverpodの状態管理を初期化する
void main() {
  runApp(
    const ProviderScope(
      child: DomainMasterApp(),
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
