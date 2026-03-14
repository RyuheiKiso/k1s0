import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/inventory.dart';
import '../providers/inventory_provider.dart';

/// 在庫一覧画面
/// 全在庫アイテムの一覧表示とステータス別の色分け表示を行うメイン画面
class InventoryListScreen extends ConsumerWidget {
  const InventoryListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    /// 在庫一覧の状態を監視する
    final inventoryAsync = ref.watch(inventoryListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('在庫一覧'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () =>
                ref.read(inventoryListProvider.notifier).load(),
          ),
        ],
      ),
      body: inventoryAsync.when(
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
                    ref.read(inventoryListProvider.notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        /// データ取得成功時は在庫一覧をリスト表示する
        data: (items) {
          if (items.isEmpty) {
            return const Center(child: Text('在庫データがありません'));
          }
          return ListView.builder(
            itemCount: items.length,
            itemBuilder: (context, index) {
              final item = items[index];
              return _InventoryListTile(item: item);
            },
          );
        },
      ),
    );
  }
}

/// 在庫一覧の個別タイルウィジェット
/// 在庫アイテムの情報とステータスに応じた色分けを表示する
class _InventoryListTile extends StatelessWidget {
  final InventoryItem item;

  const _InventoryListTile({required this.item});

  @override
  Widget build(BuildContext context) {
    return ListTile(
      /// 商品名をタイトルに表示する
      title: Text(item.productName),
      /// 倉庫名、利用可能数量、ステータスをサブタイトルに表示する
      subtitle: Text(
        '${item.warehouseName} · 数量: ${item.quantityAvailable} · ${item.status.displayName}',
      ),
      /// ステータスに応じたアイコン色を設定する
      leading: Icon(
        Icons.inventory_2,
        color: _statusColor(item.status),
      ),
      /// ステータスバッジを末尾に表示する
      trailing: _StatusBadge(status: item.status),
      /// タップで在庫詳細画面へ遷移する
      onTap: () => context.push('/inventory/${item.id}'),
    );
  }

  /// ステータスに応じた色を返す
  /// 在庫あり=緑、低在庫=オレンジ、在庫切れ=赤
  Color _statusColor(InventoryStatus status) {
    return switch (status) {
      InventoryStatus.inStock => Colors.green,
      InventoryStatus.lowStock => Colors.orange,
      InventoryStatus.outOfStock => Colors.red,
    };
  }
}

/// ステータスバッジウィジェット
/// 在庫ステータスを色付きのチップで視覚的に表示する
class _StatusBadge extends StatelessWidget {
  final InventoryStatus status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    /// ステータスに応じた背景色を設定する
    final color = switch (status) {
      InventoryStatus.inStock => Colors.green,
      InventoryStatus.lowStock => Colors.orange,
      InventoryStatus.outOfStock => Colors.red,
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: color),
      ),
      child: Text(
        status.displayName,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }
}
