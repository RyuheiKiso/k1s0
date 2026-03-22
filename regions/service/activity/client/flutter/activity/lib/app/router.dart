import 'package:go_router/go_router.dart';

import '../screens/activity_list_screen.dart';
import '../screens/activity_detail_screen.dart';
import '../screens/activity_form_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートはアクティビティ一覧画面
  initialLocation: '/',
  routes: [
    /// アクティビティ一覧画面: アクティビティの検索・フィルタリング・一覧表示を行う
    GoRoute(
      path: '/',
      name: 'activities',
      builder: (context, state) => const ActivityListScreen(),
    ),

    /// アクティビティ詳細画面: 個別アクティビティの詳細情報と承認フロー操作を行う
    GoRoute(
      path: '/activities/:id',
      name: 'activityDetail',
      builder: (context, state) {
        final id = state.pathParameters['id']!;
        return ActivityDetailScreen(activityId: id);
      },
    ),

    /// アクティビティ作成画面: 新規アクティビティの入力フォームを表示する
    GoRoute(
      path: '/activities/new',
      name: 'newActivity',
      builder: (context, state) => const ActivityFormScreen(),
    ),
  ],
);
