import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/status_definition.dart';
import '../providers/status_definition_provider.dart';

/// プロジェクトタイプ詳細画面（ステータス定義一覧）
/// プロジェクトタイプに属するステータス定義を表示・管理する
class ProjectTypeDetailScreen extends ConsumerWidget {
  /// 対象プロジェクトタイプのID
  final String projectTypeId;

  const ProjectTypeDetailScreen({super.key, required this.projectTypeId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// ステータス定義一覧の状態を監視する
    final statusDefsAsync = ref.watch(statusDefinitionListProvider(projectTypeId));

    return Scaffold(
      appBar: AppBar(
        title: Text('ステータス定義一覧: $projectTypeId'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () =>
                ref.read(statusDefinitionListProvider(projectTypeId).notifier).load(),
          ),
        ],
      ),
      body: statusDefsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    ref.read(statusDefinitionListProvider(projectTypeId).notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        data: (statusDefs) {
          if (statusDefs.isEmpty) {
            return const Center(child: Text('ステータス定義がありません'));
          }
          return ListView.builder(
            itemCount: statusDefs.length,
            itemBuilder: (context, index) {
              final def = statusDefs[index];
              return _StatusDefinitionListTile(
                statusDefinition: def,
                projectTypeId: projectTypeId,
              );
            },
          );
        },
      ),
      /// ステータス定義追加用のFAB
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showCreateStatusDefDialog(context, ref),
        tooltip: 'ステータス定義追加',
        child: const Icon(Icons.add),
      ),
    );
  }

  /// ステータス定義作成ダイアログを表示する
  void _showCreateStatusDefDialog(BuildContext context, WidgetRef ref) {
    final codeController = TextEditingController();
    final nameController = TextEditingController();
    final descController = TextEditingController();
    final colorController = TextEditingController();
    final sortController = TextEditingController(text: '0');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('ステータス定義作成'),
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
                controller: colorController,
                decoration: const InputDecoration(
                  labelText: '色（CSSカラー）',
                  hintText: '#3498db',
                ),
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
              final input = CreateStatusDefinitionInput(
                code: codeController.text.trim(),
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                color: colorController.text.trim().isNotEmpty
                    ? colorController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text) ?? 0,
              );
              ref
                  .read(statusDefinitionListProvider(projectTypeId).notifier)
                  .create(input);
              Navigator.of(context).pop();
            },
            child: const Text('作成'),
          ),
        ],
      ),
    );
  }
}

/// ステータス定義一覧の個別タイルウィジェット
/// ステータス定義情報の表示と編集・削除・履歴操作を提供する
class _StatusDefinitionListTile extends ConsumerWidget {
  final StatusDefinition statusDefinition;
  final String projectTypeId;

  const _StatusDefinitionListTile({
    required this.statusDefinition,
    required this.projectTypeId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ListTile(
      /// ステータス定義の表示名をタイトルに表示する
      title: Text(statusDefinition.displayName),
      /// コードと初期/終了フラグをサブタイトルに表示する
      subtitle: Text(
        '${statusDefinition.code}'
        '${statusDefinition.isInitial ? " · 初期" : ""}'
        '${statusDefinition.isTerminal ? " · 終了" : ""}',
      ),
      /// ステータス色をアイコンで表現する
      leading: Container(
        width: 40,
        height: 40,
        decoration: BoxDecoration(
          color: statusDefinition.color != null
              ? _parseColor(statusDefinition.color!)
              : Colors.grey.shade300,
          shape: BoxShape.circle,
        ),
        child: Center(
          child: Text(
            statusDefinition.code.substring(0, 1),
            style: const TextStyle(
              color: Colors.white,
              fontWeight: FontWeight.bold,
            ),
          ),
        ),
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          /// バージョン履歴画面への遷移ボタン
          IconButton(
            icon: const Icon(Icons.history, size: 20),
            tooltip: 'バージョン履歴',
            onPressed: () => context.push(
              '/status-definitions/${statusDefinition.id}/versions',
            ),
          ),
          PopupMenuButton<String>(
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
        ],
      ),
    );
  }

  /// CSSカラー文字列をFlutter Colorに変換する
  Color _parseColor(String cssColor) {
    try {
      if (cssColor.startsWith('#')) {
        final hex = cssColor.substring(1);
        if (hex.length == 6) {
          return Color(int.parse('FF$hex', radix: 16));
        }
      }
    } catch (_) {
      // パース失敗時はデフォルトカラーを返す
    }
    return Colors.blueGrey;
  }

  /// ステータス定義編集ダイアログを表示する
  void _showEditDialog(BuildContext context, WidgetRef ref) {
    final nameController =
        TextEditingController(text: statusDefinition.displayName);
    final descController =
        TextEditingController(text: statusDefinition.description ?? '');
    final sortController =
        TextEditingController(text: statusDefinition.sortOrder.toString());

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('ステータス定義編集: ${statusDefinition.code}'),
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
              final input = UpdateStatusDefinitionInput(
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text),
              );
              ref
                  .read(statusDefinitionListProvider(projectTypeId).notifier)
                  .update(statusDefinition.id, input);
              Navigator.of(context).pop();
            },
            child: const Text('更新'),
          ),
        ],
      ),
    );
  }

  /// ステータス定義削除の確認ダイアログを表示する
  void _showDeleteConfirmation(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('ステータス定義削除'),
        content: Text('「${statusDefinition.displayName}」を削除しますか？'),
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
                  .read(statusDefinitionListProvider(projectTypeId).notifier)
                  .delete(statusDefinition.id);
              Navigator.of(context).pop();
            },
            child: const Text('削除'),
          ),
        ],
      ),
    );
  }
}
