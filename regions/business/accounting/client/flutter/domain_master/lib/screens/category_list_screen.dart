import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/domain_master.dart';
import '../providers/domain_master_provider.dart';

/// カテゴリ一覧画面
/// マスタカテゴリのCRUD操作と一覧表示を行うメイン画面
class CategoryListScreen extends ConsumerWidget {
  const CategoryListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// カテゴリ一覧の状態を監視する
    final categoriesAsync = ref.watch(categoryListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('マスタカテゴリ管理'),
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
                ref.read(categoryListProvider.notifier).load(),
          ),
        ],
      ),
      body: categoriesAsync.when(
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
                    ref.read(categoryListProvider.notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        /// データ取得成功時はカテゴリ一覧をリスト表示する
        data: (categories) {
          if (categories.isEmpty) {
            return const Center(child: Text('カテゴリがありません'));
          }
          return ListView.builder(
            itemCount: categories.length,
            itemBuilder: (context, index) {
              final category = categories[index];
              return _CategoryListTile(category: category);
            },
          );
        },
      ),
      /// カテゴリ追加用のFAB
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showCreateCategoryDialog(context, ref),
        tooltip: 'カテゴリ追加',
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

  /// カテゴリ作成ダイアログを表示する
  void _showCreateCategoryDialog(BuildContext context, WidgetRef ref) {
    final codeController = TextEditingController();
    final nameController = TextEditingController();
    final descController = TextEditingController();
    final sortController = TextEditingController(text: '0');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('カテゴリ作成'),
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
              final input = CreateCategoryInput(
                code: codeController.text.trim(),
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text) ?? 0,
              );
              ref.read(categoryListProvider.notifier).create(input);
              Navigator.of(context).pop();
            },
            child: const Text('作成'),
          ),
        ],
      ),
    );
  }
}

/// カテゴリ一覧の個別タイルウィジェット
/// カテゴリ情報の表示と編集・削除操作を提供する
class _CategoryListTile extends ConsumerWidget {
  final MasterCategory category;

  const _CategoryListTile({required this.category});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ListTile(
      /// カテゴリの表示名をタイトルに表示する
      title: Text(category.displayName),
      /// カテゴリコードと有効/無効状態をサブタイトルに表示する
      subtitle: Text(
        '${category.code} · ${category.isActive ? "有効" : "無効"}',
      ),
      /// 有効状態に応じてアイコンの色を変更する
      leading: Icon(
        Icons.folder,
        color: category.isActive ? Colors.indigo : Colors.grey,
      ),
      /// タップでアイテム一覧画面へ遷移する
      onTap: () => context.push('/categories/${category.code}/items'),
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

  /// カテゴリ編集ダイアログを表示する
  void _showEditDialog(BuildContext context, WidgetRef ref) {
    final nameController = TextEditingController(text: category.displayName);
    final descController = TextEditingController(text: category.description ?? '');
    final sortController =
        TextEditingController(text: category.sortOrder.toString());

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('カテゴリ編集: ${category.code}'),
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
              final input = UpdateCategoryInput(
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text),
              );
              ref
                  .read(categoryListProvider.notifier)
                  .update(category.code, input);
              Navigator.of(context).pop();
            },
            child: const Text('更新'),
          ),
        ],
      ),
    );
  }

  /// カテゴリ削除の確認ダイアログを表示する
  void _showDeleteConfirmation(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('カテゴリ削除'),
        content: Text('「${category.displayName}」を削除しますか？'),
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
                  .read(categoryListProvider.notifier)
                  .delete(category.code);
              Navigator.of(context).pop();
            },
            child: const Text('削除'),
          ),
        ],
      ),
    );
  }
}
