import 'package:go_router/go_router.dart';

import '../screens/category_list_screen.dart';
import '../screens/item_list_screen.dart';
import '../screens/version_history_screen.dart';
import '../screens/tenant_extension_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートはカテゴリ一覧画面
  initialLocation: '/',
  routes: [
    /// カテゴリ一覧画面: マスタカテゴリの管理を行う
    GoRoute(
      path: '/',
      name: 'categories',
      builder: (context, state) => const CategoryListScreen(),
    ),

    /// アイテム一覧画面: 特定カテゴリに属するアイテムを管理する
    GoRoute(
      path: '/categories/:code/items',
      name: 'items',
      builder: (context, state) {
        final code = state.pathParameters['code']!;
        return ItemListScreen(categoryCode: code);
      },
    ),

    /// バージョン履歴画面: アイテムの変更履歴を閲覧する
    GoRoute(
      path: '/categories/:code/items/:item_code/versions',
      name: 'versions',
      builder: (context, state) {
        final code = state.pathParameters['code']!;
        final itemCode = state.pathParameters['item_code']!;
        return VersionHistoryScreen(
          categoryCode: code,
          itemCode: itemCode,
        );
      },
    ),

    /// テナント拡張画面: テナント固有のマスタカスタマイズを管理する
    GoRoute(
      path: '/tenants/:tenant_id/extensions',
      name: 'tenantExtensions',
      builder: (context, state) {
        final tenantId = state.pathParameters['tenant_id']!;
        return TenantExtensionScreen(tenantId: tenantId);
      },
    ),
  ],
);
