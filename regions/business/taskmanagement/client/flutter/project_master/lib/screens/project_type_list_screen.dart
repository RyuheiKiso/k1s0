import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/project_type.dart';
import '../providers/project_type_provider.dart';

/// プロジェクトタイプ一覧画面
/// プロジェクトタイプのCRUD操作と一覧表示を行うメイン画面
class ProjectTypeListScreen extends ConsumerWidget {
  const ProjectTypeListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// プロジェクトタイプ一覧の状態を監視する
    final projectTypesAsync = ref.watch(projectTypeListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('プロジェクトタイプ管理'),
        actions: [
          /// テナント拡張画面への遷移ボタン
          IconButton(
            icon: const Icon(Icons.business),
            tooltip: 'テナント拡張',
            onPressed: () => _showTenantIdDialog(context),
          ),
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () =>
                ref.read(projectTypeListProvider.notifier).load(),
          ),
        ],
      ),
      body: projectTypesAsync.when(
        /// ローディング中はプログレスインジケーターを表示する
        loading: () => const Center(child: CircularProgressIndicator()),
        /// エラー時はエラーメッセージとリトライボタンを表示する
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    ref.read(projectTypeListProvider.notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        /// データ取得成功時はプロジェクトタイプ一覧をリスト表示する
        data: (projectTypes) {
          if (projectTypes.isEmpty) {
            return const Center(child: Text('プロジェクトタイプがありません'));
          }
          return ListView.builder(
            itemCount: projectTypes.length,
            itemBuilder: (context, index) {
              final projectType = projectTypes[index];
              return _ProjectTypeListTile(projectType: projectType);
            },
          );
        },
      ),
      /// プロジェクトタイプ追加用のFAB
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showCreateProjectTypeDialog(context, ref),
        tooltip: 'プロジェクトタイプ追加',
        child: const Icon(Icons.add),
      ),
    );
  }

  /// テナントID入力ダイアログを表示し、テナント拡張画面へ遷移する
  void _showTenantIdDialog(BuildContext context) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('テナントID入力'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(
            labelText: 'テナントID',
            hintText: 'テナントIDを入力してください',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            onPressed: () {
              final tenantId = controller.text.trim();
              if (tenantId.isNotEmpty) {
                Navigator.of(context).pop();
                context.push('/tenants/$tenantId/extensions');
              }
            },
            child: const Text('開く'),
          ),
        ],
      ),
    );
  }

  /// プロジェクトタイプ作成ダイアログを表示する
  void _showCreateProjectTypeDialog(BuildContext context, WidgetRef ref) {
    final codeController = TextEditingController();
    final nameController = TextEditingController();
    final descController = TextEditingController();
    final sortController = TextEditingController(text: '0');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('プロジェクトタイプ作成'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: codeController,
                decoration: const InputDecoration(labelText: 'コード'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: nameController,
                decoration: const InputDecoration(labelText: '表示名'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: descController,
                decoration: const InputDecoration(labelText: '説明（任意）'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: sortController,
                decoration: const InputDecoration(labelText: '表示順'),
                keyboardType: TextInputType.number,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            onPressed: () {
              final input = CreateProjectTypeInput(
                code: codeController.text.trim(),
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text) ?? 0,
              );
              ref.read(projectTypeListProvider.notifier).create(input);
              Navigator.of(context).pop();
            },
            child: const Text('作成'),
          ),
        ],
      ),
    );
  }
}

/// プロジェクトタイプ一覧の個別タイルウィジェット
/// プロジェクトタイプ情報の表示と編集・削除操作を提供する
class _ProjectTypeListTile extends ConsumerWidget {
  final ProjectType projectType;

  const _ProjectTypeListTile({required this.projectType});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ListTile(
      /// プロジェクトタイプの表示名をタイトルに表示する
      title: Text(projectType.displayName),
      /// プロジェクトタイプコードと有効/無効状態をサブタイトルに表示する
      subtitle: Text(
        '${projectType.code} · ${projectType.isActive ? "有効" : "無効"}',
      ),
      /// 有効状態に応じてアイコンの色を変更する
      leading: Icon(
        Icons.category,
        color: projectType.isActive ? Colors.indigo : Colors.grey,
      ),
      /// タップでステータス定義一覧画面へ遷移する
      onTap: () =>
          context.push('/project-types/${projectType.id}/status-definitions'),
      trailing: PopupMenuButton<String>(
        onSelected: (value) {
          switch (value) {
            case 'edit':
              _showEditDialog(context, ref);
              break;
            case 'delete':
              _showDeleteConfirmation(context, ref);
              break;
          }
        },
        itemBuilder: (context) => [
          const PopupMenuItem(value: 'edit', child: Text('編集')),
          const PopupMenuItem(value: 'delete', child: Text('削除')),
        ],
      ),
    );
  }

  /// プロジェクトタイプ編集ダイアログを表示する
  void _showEditDialog(BuildContext context, WidgetRef ref) {
    final nameController =
        TextEditingController(text: projectType.displayName);
    final descController =
        TextEditingController(text: projectType.description ?? '');
    final sortController =
        TextEditingController(text: projectType.sortOrder.toString());

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('プロジェクトタイプ編集: ${projectType.code}'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: nameController,
                decoration: const InputDecoration(labelText: '表示名'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: descController,
                decoration: const InputDecoration(labelText: '説明'),
              ),
              const SizedBox(height: 8),
              TextField(
                controller: sortController,
                decoration: const InputDecoration(labelText: '表示順'),
                keyboardType: TextInputType.number,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            onPressed: () {
              final input = UpdateProjectTypeInput(
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text),
              );
              ref
                  .read(projectTypeListProvider.notifier)
                  .update(projectType.id, input);
              Navigator.of(context).pop();
            },
            child: const Text('更新'),
          ),
        ],
      ),
    );
  }

  /// プロジェクトタイプ削除の確認ダイアログを表示する
  void _showDeleteConfirmation(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('プロジェクトタイプ削除'),
        content: Text('「${projectType.displayName}」を削除しますか？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          FilledButton(
            style: FilledButton.styleFrom(
              backgroundColor: Colors.red,
            ),
            onPressed: () {
              ref
                  .read(projectTypeListProvider.notifier)
                  .delete(projectType.id);
              Navigator.of(context).pop();
            },
            child: const Text('削除'),
          ),
        ],
      ),
    );
  }
}
