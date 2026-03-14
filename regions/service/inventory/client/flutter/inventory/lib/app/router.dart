import 'package:go_router/go_router.dart';

import '../screens/inventory_list_screen.dart';
import '../screens/inventory_detail_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートは在庫一覧画面
  initialLocation: '/',
  routes: [
    /// 在庫一覧画面: 在庫アイテムの一覧表示と管理を行う
    GoRoute(
      path: '/',
      name: 'inventoryList',
      builder: (context, state) => const InventoryListScreen(),
    ),

    /// 在庫詳細画面: 個別の在庫アイテムの詳細表示と操作を行う
    GoRoute(
      path: '/inventory/:id',
      name: 'inventoryDetail',
      builder: (context, state) {
        final id = state.pathParameters['id']!;
        return InventoryDetailScreen(inventoryId: id);
      },
    ),
  ],
);
