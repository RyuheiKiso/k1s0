import 'package:go_router/go_router.dart';

import '../screens/payment_list_screen.dart';
import '../screens/payment_detail_screen.dart';
import '../screens/payment_form_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートは決済一覧画面
  initialLocation: '/',
  routes: [
    /// 決済一覧画面: 決済の検索・一覧表示を行う
    GoRoute(
      path: '/',
      name: 'payments',
      builder: (context, state) => const PaymentListScreen(),
    ),

    /// 決済詳細画面: 特定の決済情報の閲覧と操作を行う
    GoRoute(
      path: '/payments/:id',
      name: 'paymentDetail',
      builder: (context, state) {
        final id = state.pathParameters['id']!;
        return PaymentDetailScreen(paymentId: id);
      },
    ),

    /// 決済作成画面: 新規決済の開始を行う
    GoRoute(
      path: '/payments/new',
      name: 'paymentForm',
      builder: (context, state) => const PaymentFormScreen(),
    ),
  ],
);
