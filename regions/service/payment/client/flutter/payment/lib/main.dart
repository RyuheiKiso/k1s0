import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app/router.dart';

/// アプリケーションのエントリポイント
/// ProviderScopeでRiverpodの状態管理を初期化する
void main() {
  runApp(
    const ProviderScope(
      child: PaymentApp(),
    ),
  );
}

/// 決済管理アプリケーションのルートウィジェット
/// Material Designテーマとgo_routerによるルーティングを設定する
class PaymentApp extends ConsumerWidget {
  const PaymentApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: '決済管理',
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
