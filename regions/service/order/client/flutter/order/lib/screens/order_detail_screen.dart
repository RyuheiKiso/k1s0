import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/order.dart';
import '../providers/order_provider.dart';

/// 注文詳細画面
/// 個別注文の詳細情報表示とステータス更新を行う画面
class OrderDetailScreen extends ConsumerStatefulWidget {
  /// 対象注文のID
  final String orderId;

  const OrderDetailScreen({super.key, required this.orderId});

  @override
  ConsumerState<OrderDetailScreen> createState() => _OrderDetailScreenState();
}

class _OrderDetailScreenState extends ConsumerState<OrderDetailScreen> {
  /// ステータス更新用の選択値
  OrderStatus? _selectedStatus;

  @override
  Widget build(BuildContext context) {
    /// 注文一覧の状態を監視し、対象注文を抽出する
    final ordersAsync = ref.watch(orderListProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text('注文詳細: ${widget.orderId}'),
        actions: [
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref.read(orderListProvider.notifier).load(),
          ),
        ],
      ),
      body: ordersAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('エラーが発生しました: $error'),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () => ref.read(orderListProvider.notifier).load(),
                child: const Text('再試行'),
              ),
            ],
          ),
        ),
        data: (orders) {
          /// 注文一覧から対象IDの注文を検索する
          final order = orders.where((o) => o.id == widget.orderId).firstOrNull;
          if (order == null) {
            return const Center(child: Text('注文が見つかりません'));
          }
          return _buildOrderDetail(context, order);
        },
      ),
    );
  }

  /// 注文詳細の全体レイアウトを構築する
  Widget _buildOrderDetail(BuildContext context, Order order) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          /// 注文基本情報カード
          _buildOrderInfoCard(context, order),
          const SizedBox(height: 16),

          /// ステータス更新カード
          _buildStatusUpdateCard(context, order),
          const SizedBox(height: 16),

          /// 注文明細一覧カード
          _buildItemsCard(context, order),
        ],
      ),
    );
  }

  /// 注文基本情報を表示するカードを構築する
  Widget _buildOrderInfoCard(BuildContext context, Order order) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '注文情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            _buildInfoRow('注文ID', order.id),
            _buildInfoRow('顧客ID', order.customerId),
            _buildInfoRow('ステータス', order.status.displayName),
            _buildInfoRow(
              '合計金額',
              '${order.totalAmount.toStringAsFixed(2)} ${order.currency}',
            ),
            _buildInfoRow('バージョン', order.version.toString()),
            if (order.notes != null && order.notes!.isNotEmpty)
              _buildInfoRow('備考', order.notes!),
            _buildInfoRow('作成日時', _formatDateTime(order.createdAt)),
            _buildInfoRow('更新日時', _formatDateTime(order.updatedAt)),
          ],
        ),
      ),
    );
  }

  /// ステータス更新操作を行うカードを構築する
  Widget _buildStatusUpdateCard(BuildContext context, Order order) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'ステータス更新',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                /// ステータス選択用のドロップダウン
                Expanded(
                  child: DropdownButtonFormField<OrderStatus>(
                    initialValue: _selectedStatus ?? order.status,
                    decoration: const InputDecoration(
                      labelText: '新しいステータス',
                      border: OutlineInputBorder(),
                    ),
                    items: OrderStatus.values.map((status) {
                      return DropdownMenuItem(
                        value: status,
                        child: Text(status.displayName),
                      );
                    }).toList(),
                    onChanged: (value) {
                      setState(() => _selectedStatus = value);
                    },
                  ),
                ),
                const SizedBox(width: 16),
                /// ステータス更新実行ボタン
                FilledButton(
                  onPressed: () => _updateStatus(order),
                  child: const Text('更新'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  /// 注文明細一覧を表示するカードを構築する
  Widget _buildItemsCard(BuildContext context, Order order) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '注文明細 (${order.items.length}件)',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            /// 明細データをDataTableで表示する
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: DataTable(
                columns: const [
                  DataColumn(label: Text('商品ID')),
                  DataColumn(label: Text('商品名')),
                  DataColumn(label: Text('数量'), numeric: true),
                  DataColumn(label: Text('単価'), numeric: true),
                  DataColumn(label: Text('小計'), numeric: true),
                ],
                rows: order.items.map((item) {
                  return DataRow(cells: [
                    DataCell(Text(item.productId)),
                    DataCell(Text(item.productName)),
                    DataCell(Text(item.quantity.toString())),
                    DataCell(Text(item.unitPrice.toStringAsFixed(2))),
                    DataCell(Text(item.subtotal.toStringAsFixed(2))),
                  ]);
                }).toList(),
              ),
            ),
            const Divider(),
            /// 合計金額を表示する
            Align(
              alignment: Alignment.centerRight,
              child: Text(
                '合計: ${order.totalAmount.toStringAsFixed(2)} ${order.currency}',
                style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  /// 情報表示用の行ウィジェットを構築する
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
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  /// 注文ステータスを更新する
  void _updateStatus(Order order) {
    final newStatus = _selectedStatus ?? order.status;
    if (newStatus == order.status) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('ステータスが変更されていません')),
      );
      return;
    }

    final input = UpdateOrderStatusInput(status: newStatus);
    ref.read(orderListProvider.notifier).updateStatus(order.id, input);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('ステータスを「${newStatus.displayName}」に更新しました')),
    );
  }

  /// DateTimeを読みやすい日本語形式にフォーマットする
  String _formatDateTime(DateTime dateTime) {
    return '${dateTime.year}/${dateTime.month.toString().padLeft(2, '0')}/'
        '${dateTime.day.toString().padLeft(2, '0')} '
        '${dateTime.hour.toString().padLeft(2, '0')}:'
        '${dateTime.minute.toString().padLeft(2, '0')}';
  }
}
