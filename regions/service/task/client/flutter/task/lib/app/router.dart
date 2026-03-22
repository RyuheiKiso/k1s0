import 'package:go_router/go_router.dart';

import '../screens/task_list_screen.dart';
import '../screens/task_detail_screen.dart';
import '../screens/task_form_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートはタスク一覧画面
  initialLocation: '/',
  routes: [
    /// タスク一覧画面: タスクの検索・フィルタリング・一覧表示を行う
    GoRoute(
      path: '/',
      name: 'tasks',
      builder: (context, state) => const TaskListScreen(),
    ),

    /// タスク詳細画面: 個別タスクの詳細情報とステータス更新を行う
    GoRoute(
      path: '/tasks/:id',
      name: 'taskDetail',
      builder: (context, state) {
        final id = state.pathParameters['id']!;
        return TaskDetailScreen(taskId: id);
      },
    ),

    /// タスク作成画面: 新規タスクの入力フォームを表示する
    GoRoute(
      path: '/tasks/new',
      name: 'newTask',
      builder: (context, state) => const TaskFormScreen(),
    ),
  ],
);
