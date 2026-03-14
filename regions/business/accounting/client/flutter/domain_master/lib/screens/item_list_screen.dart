import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/domain_master.dart';
import '../providers/domain_master_provider.dart';

/// アイテム一覧画面
/// カテゴリに属するマスタアイテムを階層構造で表示・管理する
class ItemListScreen extends ConsumerWidget {
  /// 対象カテゴリのコード
  final String categoryCode;

  const ItemListScreen({super.key, required this.categoryCode});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// アイテム一覧の状態を監視する
    final itemsAsync = ref.watch(itemListProvider(categoryCode));

    return Scaffold(
      appBar: AppBar(
        title: Text('アイテム一覧: $categoryCode'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () =>
                ref.read(itemListProvider(categoryCode).notifier).load(),
          ),
        ],
      ),
      body: itemsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () =>
                    ref.read(itemListProvider(categoryCode).notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        data: (items) {
          if (items.isEmpty) {
            return const Center(child: Text('アイテムがありません'));
          }

          /// 親アイテムのないルートレベルのアイテムを抽出する
          final rootItems =
              items.where((item) => item.parentItemId == null).toList();

          return ListView.builder(
            itemCount: rootItems.length,
            itemBuilder: (context, index) {
              return _ItemTreeTile(
                item: rootItems[index],
                allItems: items,
                categoryCode: categoryCode,
                depth: 0,
              );
            },
          );
        },
      ),
      /// アイテム追加用のFAB
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showCreateItemDialog(context, ref),
        tooltip: 'アイテム追加',
        child: const Icon(Icons.add),
      ),
    );
  }

  /// アイテム作成ダイアログを表示する
  void _showCreateItemDialog(BuildContext context, WidgetRef ref) {
    final codeController = TextEditingController();
    final nameController = TextEditingController();
    final descController = TextEditingController();
    final sortController = TextEditingController(text: '0');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('アイテム作成'),
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
              final input = CreateItemInput(
                code: codeController.text.trim(),
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text) ?? 0,
              );
              ref.read(itemListProvider(categoryCode).notifier).create(input);
              Navigator.of(context).pop();
            },
            child: const Text('作成'),
          ),
        ],
      ),
    );
  }
}

/// アイテムツリータイルウィジェット
/// 階層構造を持つアイテムをExpansionTileで再帰的に表示する
class _ItemTreeTile extends ConsumerWidget {
  final MasterItem item;
  final List<MasterItem> allItems;
  final String categoryCode;
  /// ツリーの深さ（インデント計算に使用）
  final int depth;

  const _ItemTreeTile({
    required this.item,
    required this.allItems,
    required this.categoryCode,
    required this.depth,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// 現在のアイテムを親とする子アイテムを抽出する
    final children =
        allItems.where((i) => i.parentItemId == item.id).toList();

    /// 子アイテムがある場合はExpansionTileで階層表示する
    if (children.isNotEmpty) {
      return ExpansionTile(
        leading: Icon(
          Icons.folder_open,
          color: item.isActive ? Colors.amber : Colors.grey,
        ),
        title: Text(item.displayName),
        subtitle: Text(
          '${item.code} · ${item.isActive ? "有効" : "無効"}',
        ),
        tilePadding: EdgeInsets.only(left: 16.0 + depth * 24.0, right: 16.0),
        trailing: _buildTrailingActions(context, ref),
        children: children
            .map((child) => _ItemTreeTile(
                  item: child,
                  allItems: allItems,
                  categoryCode: categoryCode,
                  depth: depth + 1,
                ))
            .toList(),
      );
    }

    /// 子アイテムがない場合はListTileで表示する
    return ListTile(
      contentPadding:
          EdgeInsets.only(left: 16.0 + depth * 24.0, right: 16.0),
      leading: Icon(
        Icons.description,
        color: item.isActive ? Colors.indigo : Colors.grey,
      ),
      title: Text(item.displayName),
      subtitle: Text(
        '${item.code} · ${item.isActive ? "有効" : "無効"}',
      ),
      trailing: _buildTrailingActions(context, ref),
    );
  }

  /// アイテムの操作メニューを構築する
  Widget _buildTrailingActions(BuildContext context, WidgetRef ref) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        /// バージョン履歴画面への遷移ボタン
        IconButton(
          icon: const Icon(Icons.history, size: 20),
          tooltip: 'バージョン履歴',
          onPressed: () => context.push(
            '/categories/$categoryCode/items/${item.code}/versions',
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
    );
  }

  /// アイテム編集ダイアログを表示する
  void _showEditDialog(BuildContext context, WidgetRef ref) {
    final nameController = TextEditingController(text: item.displayName);
    final descController =
        TextEditingController(text: item.description ?? '');
    final sortController =
        TextEditingController(text: item.sortOrder.toString());

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('アイテム編集: ${item.code}'),
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
              final input = UpdateItemInput(
                displayName: nameController.text.trim(),
                description: descController.text.trim().isNotEmpty
                    ? descController.text.trim()
                    : null,
                sortOrder: int.tryParse(sortController.text),
              );
              ref
                  .read(itemListProvider(categoryCode).notifier)
                  .update(item.code, input);
              Navigator.of(context).pop();
            },
            child: const Text('更新'),
          ),
        ],
      ),
    );
  }

  /// アイテム削除の確認ダイアログを表示する
  void _showDeleteConfirmation(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('アイテム削除'),
        content: Text('「${item.displayName}」を削除しますか？'),
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
                  .read(itemListProvider(categoryCode).notifier)
                  .delete(item.code);
              Navigator.of(context).pop();
            },
            child: const Text('削除'),
          ),
        ],
      ),
    );
  }
}
