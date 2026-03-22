import 'package:go_router/go_router.dart';

import '../screens/board_screen.dart';
import '../screens/wip_limit_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートはボードホーム（プロジェクト一覧はないのでデフォルトプロジェクト表示）
  initialLocation: '/boards/default',
  routes: [
    /// ボード表示画面: プロジェクトIDに対応するKanbanカラムを表示する
    GoRoute(
      path: '/boards/:projectId',
      name: 'board',
      builder: (context, state) {
        final projectId = state.pathParameters['projectId']!;
        return BoardScreen(projectId: projectId);
      },
    ),

    /// WIP制限編集画面: 指定カラムのWIP制限を編集する
    GoRoute(
      path: '/boards/:projectId/columns/:statusCode/wip-limit',
      name: 'wipLimit',
      builder: (context, state) {
        final projectId = state.pathParameters['projectId']!;
        final statusCode = state.pathParameters['statusCode']!;
        return WipLimitScreen(projectId: projectId, statusCode: statusCode);
      },
    ),
  ],
);
