import 'package:go_router/go_router.dart';

import '../screens/project_type_list_screen.dart';
import '../screens/project_type_detail_screen.dart';
import '../screens/status_definition_list_screen.dart';
import '../screens/tenant_extension_screen.dart';

/// アプリケーションのルーティング設定
/// go_routerを使用してSPA内のナビゲーションを管理する
final router = GoRouter(
  /// 初期ルートはプロジェクトタイプ一覧画面
  initialLocation: '/',
  routes: [
    /// プロジェクトタイプ一覧画面: プロジェクトタイプの管理を行う
    GoRoute(
      path: '/',
      name: 'projectTypes',
      builder: (context, state) => const ProjectTypeListScreen(),
    ),

    /// プロジェクトタイプ詳細・ステータス定義一覧画面
    GoRoute(
      path: '/project-types/:project_type_id/status-definitions',
      name: 'statusDefinitions',
      builder: (context, state) {
        final projectTypeId = state.pathParameters['project_type_id']!;
        return ProjectTypeDetailScreen(projectTypeId: projectTypeId);
      },
    ),

    /// ステータス定義バージョン履歴画面: ステータス定義の変更履歴を閲覧する
    GoRoute(
      path: '/status-definitions/:status_definition_id/versions',
      name: 'versions',
      builder: (context, state) {
        final statusDefinitionId =
            state.pathParameters['status_definition_id']!;
        return StatusDefinitionListScreen(
          statusDefinitionId: statusDefinitionId,
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
