import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
// H-13 監査対応: 未認証ユーザーの画面アクセスをブロックするため認証ガードを使用する
// system_client の authGuardRedirect は未認証時に /login へリダイレクトする
// authProvider は Riverpod の NotifierProvider で認証状態を管理する
import 'package:system_client/system_client.dart';

import '../screens/project_type_list_screen.dart';
import '../screens/project_type_detail_screen.dart';
import '../screens/status_definition_list_screen.dart';
import '../screens/tenant_extension_screen.dart';

/// H-13 監査対応: GoRouter インスタンスを Riverpod の WidgetRef から生成するファクトリ関数。
/// GoRouter の redirect コールバック内で authProvider を参照するために ref が必要なため、
/// 静的な final 変数ではなく関数として定義する。
GoRouter createRouter(WidgetRef ref) => GoRouter(
  /// 初期ルートはプロジェクトタイプ一覧画面
  initialLocation: '/',
  /// H-13 監査対応: 未認証ユーザーをログインページへリダイレクトするガード関数を設定する
  /// authProvider から現在の認証状態を取得し、authGuardRedirect で /login への転送を判定する
  redirect: (context, state) {
    // Riverpod authProvider から認証状態を取得する（watch で再評価を保証）
    final authState = ref.read(authProvider);
    return authGuardRedirect(
      authState: authState,
      location: state.uri.toString(),
    );
  },
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
