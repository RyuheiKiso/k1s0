import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'router.dart';

/// タスク管理アプリケーションのルートウィジェット
/// Material Designテーマとgo_routerによるルーティングを設定する
class TaskApp extends ConsumerWidget {
  const TaskApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return MaterialApp.router(
      title: 'タスク管理',
      theme: ThemeData(
        colorSchemeSeed: Colors.blue,
        useMaterial3: true,
      ),
      /// go_routerの設定をMaterialAppに適用する
      routerConfig: router,
      debugShowCheckedModeBanner: false,
    );
  }
}
