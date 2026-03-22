import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'router.dart';

/// ボード管理アプリケーションのルートウィジェット
/// Material Designテーマとgo_routerによるルーティングを設定する
class BoardApp extends ConsumerWidget {
  const BoardApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'ボード管理',
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
