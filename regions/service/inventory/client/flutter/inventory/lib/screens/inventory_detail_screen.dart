import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/inventory.dart';
import '../providers/inventory_provider.dart';

/// 在庫詳細画面
/// 個別の在庫アイテムの詳細表示と在庫操作（引当・引当解除・更新）を行う
class InventoryDetailScreen extends ConsumerStatefulWidget {
  /// 対象の在庫アイテムID
  final String inventoryId;

  const InventoryDetailScreen({super.key, required this.inventoryId});

  @override
  ConsumerState<InventoryDetailScreen> createState() =>
      _InventoryDetailScreenState();
}

class _InventoryDetailScreenState extends ConsumerState<InventoryDetailScreen> {
  @override
  Widget build(BuildContext context) {
    /// 在庫一覧から対象アイテムを検索する
    final inventoryAsync = ref.watch(inventoryListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('在庫詳細'),
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
        /// データ取得成功時は在庫詳細を表示する
        data: (items) {
          /// 一覧から対象IDのアイテムを検索する
          final item = items.where((i) => i.id == widget.inventoryId).firstOrNull;
          if (item == null) {
            return const Center(child: Text('在庫データが見つかりません'));
          }
          return _DetailBody(item: item);
        },
      ),
    );
  }
}

/// 在庫詳細の本体ウィジェット
/// アイテム情報の表示と操作ボタンを配置する
class _DetailBody extends ConsumerWidget {
  final InventoryItem item;

  const _DetailBody({required this.item});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          /// ステータスカード: 現在の在庫ステータスを色付きで表示する
          _buildStatusCard(context),
          const SizedBox(height: 16),

          /// 基本情報カード: 商品と倉庫の情報を表示する
          _buildInfoCard(context),
          const SizedBox(height: 16),

          /// 数量情報カード: 在庫数量の詳細を表示する
          _buildQuantityCard(context),
          const SizedBox(height: 24),

          /// 操作ボタン群: 引当・引当解除・在庫更新の操作を提供する
          _buildActionButtons(context, ref),
        ],
      ),
    );
  }

  /// ステータスカードを構築する
  /// 在庫ステータスに応じた色とアイコンを表示する
  Widget _buildStatusCard(BuildContext context) {
    final color = _statusColor(item.status);
    return Card(
      color: color.withOpacity(0.1),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(Icons.circle, color: color, size: 16),
            const SizedBox(width: 8),
            Text(
              item.status.displayName,
              style: TextStyle(
                color: color,
                fontSize: 18,
                fontWeight: FontWeight.bold,
              ),
            ),
            const Spacer(),
            Text(
              'バージョン: ${item.version}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }

  /// 基本情報カードを構築する
  /// 商品ID、商品名、倉庫ID、倉庫名を表示する
  Widget _buildInfoCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '基本情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const Divider(),
            _buildInfoRow('商品ID', item.productId),
            _buildInfoRow('商品名', item.productName),
            _buildInfoRow('倉庫ID', item.warehouseId),
            _buildInfoRow('倉庫名', item.warehouseName),
            _buildInfoRow('作成日時', _formatDateTime(item.createdAt)),
            _buildInfoRow('更新日時', _formatDateTime(item.updatedAt)),
          ],
        ),
      ),
    );
  }

  /// 数量情報カードを構築する
  /// 利用可能数量、引当数量、発注点を表示する
  Widget _buildQuantityCard(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '数量情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const Divider(),
            _buildInfoRow('利用可能数量', '${item.quantityAvailable}'),
            _buildInfoRow('引当数量', '${item.quantityReserved}'),
            _buildInfoRow('発注点', '${item.reorderPoint}'),
          ],
        ),
      ),
    );
  }

  /// 操作ボタン群を構築する
  /// 引当・引当解除・在庫更新の3つの操作ボタンを横並びで表示する
  Widget _buildActionButtons(BuildContext context, WidgetRef ref) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        /// 在庫引当ボタン
        FilledButton.icon(
          onPressed: () => _showReserveDialog(context, ref),
          icon: const Icon(Icons.lock),
          label: const Text('在庫引当'),
        ),
        /// 引当解除ボタン
        FilledButton.tonalIcon(
          onPressed: () => _showReleaseDialog(context, ref),
          icon: const Icon(Icons.lock_open),
          label: const Text('引当解除'),
        ),
        /// 在庫更新ボタン
        OutlinedButton.icon(
          onPressed: () => _showUpdateDialog(context, ref),
          icon: const Icon(Icons.edit),
          label: const Text('在庫更新'),
        ),
      ],
    );
  }

  /// 情報行ウィジェットを構築する
  /// ラベルと値を横並びで表示する共通コンポーネント
  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: const TextStyle(
                fontWeight: FontWeight.bold,
                color: Colors.grey,
              ),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  /// 在庫引当ダイアログを表示する
  /// 引当数量を入力してサーバーに送信する
  void _showReserveDialog(BuildContext context, WidgetRef ref) {
    final quantityController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('在庫引当'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('商品: ${item.productName}'),
              Text('倉庫: ${item.warehouseName}'),
              Text('利用可能数量: ${item.quantityAvailable}'),
              const SizedBox(height: 16),
              TextField(
                controller: quantityController,
                decoration: const InputDecoration(labelText: '引当数量'),
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
              final quantity = int.tryParse(quantityController.text);
              if (quantity != null && quantity > 0) {
                final operation = StockOperation(
                  productId: item.productId,
                  warehouseId: item.warehouseId,
                  quantity: quantity,
                );
                ref.read(inventoryListProvider.notifier).reserveStock(operation);
                Navigator.of(context).pop();
              }
            },
            child: const Text('引当実行'),
          ),
        ],
      ),
    );
  }

  /// 引当解除ダイアログを表示する
  /// 解除数量を入力してサーバーに送信する
  void _showReleaseDialog(BuildContext context, WidgetRef ref) {
    final quantityController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('引当解除'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('商品: ${item.productName}'),
              Text('倉庫: ${item.warehouseName}'),
              Text('引当数量: ${item.quantityReserved}'),
              const SizedBox(height: 16),
              TextField(
                controller: quantityController,
                decoration: const InputDecoration(labelText: '解除数量'),
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
              final quantity = int.tryParse(quantityController.text);
              if (quantity != null && quantity > 0) {
                final operation = StockOperation(
                  productId: item.productId,
                  warehouseId: item.warehouseId,
                  quantity: quantity,
                );
                ref.read(inventoryListProvider.notifier).releaseStock(operation);
                Navigator.of(context).pop();
              }
            },
            child: const Text('解除実行'),
          ),
        ],
      ),
    );
  }

  /// 在庫更新ダイアログを表示する
  /// 利用可能数量と発注点を入力してサーバーに送信する
  void _showUpdateDialog(BuildContext context, WidgetRef ref) {
    final quantityController = TextEditingController(
      text: item.quantityAvailable.toString(),
    );
    final reorderController = TextEditingController(
      text: item.reorderPoint.toString(),
    );

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('在庫更新'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('商品: ${item.productName}'),
              const SizedBox(height: 16),
              TextField(
                controller: quantityController,
                decoration: const InputDecoration(labelText: '利用可能数量'),
                keyboardType: TextInputType.number,
              ),
              const SizedBox(height: 8),
              TextField(
                controller: reorderController,
                decoration: const InputDecoration(labelText: '発注点'),
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
              final input = UpdateStockInput(
                quantityAvailable: int.tryParse(quantityController.text),
                reorderPoint: int.tryParse(reorderController.text),
              );
              ref
                  .read(inventoryListProvider.notifier)
                  .updateStock(item.id, input);
              Navigator.of(context).pop();
            },
            child: const Text('更新'),
          ),
        ],
      ),
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

  /// DateTimeをフォーマットされた文字列に変換する
  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}/${dateTime.month.toString().padLeft(2, '0')}/${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
