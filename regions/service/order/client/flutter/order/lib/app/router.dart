import 'package:go_router/go_router.dart';

import '../screens/order_list_screen.dart';
import '../screens/order_detail_screen.dart';
import '../screens/order_form_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートは注文一覧画面
  initialLocation: '/',
  routes: [
    /// 注文一覧画面: 注文の検索・フィルタリング・一覧表示を行う
    GoRoute(
      path: '/',
      name: 'orders',
      builder: (context, state) => const OrderListScreen(),
    ),

    /// 注文詳細画面: 個別注文の詳細情報とステータス更新を行う
    GoRoute(
      path: '/orders/:id',
      name: 'orderDetail',
      builder: (context, state) {
        final id = state.pathParameters['id']!;
        return OrderDetailScreen(orderId: id);
      },
    ),

    /// 注文作成画面: 新規注文の入力フォームを表示する
    GoRoute(
      path: '/orders/new',
      name: 'newOrder',
      builder: (context, state) => const OrderFormScreen(),
    ),
  ],
);
