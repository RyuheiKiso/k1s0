import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/order.dart';
import '../providers/order_provider.dart';

/// 注文一覧画面
/// 注文の検索・フィルタリング・一覧表示を行うメイン画面
class OrderListScreen extends ConsumerStatefulWidget {
  const OrderListScreen({super.key});

  @override
  ConsumerState<OrderListScreen> createState() => _OrderListScreenState();
}

class _OrderListScreenState extends ConsumerState<OrderListScreen> {
  /// 現在選択中のステータスフィルタ（nullの場合は全件表示）
  OrderStatus? _selectedStatus;

  @override
  Widget build(BuildContext context) {
    /// 注文一覧の状態を監視する
    final ordersAsync = ref.watch(orderListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('注文一覧'),
        actions: [
          /// 新規注文作成画面への遷移ボタン
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: '新規注文',
            onPressed: () => context.push('/orders/new'),
          ),
          /// 一覧を再読み込みするボタン
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: '更新',
            onPressed: () => ref.read(orderListProvider.notifier).load(
                  status: _selectedStatus,
                ),
          ),
        ],
      ),
      body: Column(
        children: [
          /// ステータスフィルタ用のチップを表示する
          _buildStatusFilterChips(),
          /// 注文一覧を表示する
          Expanded(
            child: ordersAsync.when(
              /// ローディング中はプログレスインジケーターを表示する
              loading: () =>
                  const Center(child: CircularProgressIndicator()),
              /// エラー時はエラーメッセージとリトライボタンを表示する
              error: (error, stack) => Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text('エラーが発生しました: $error'),
                    const SizedBox(height: 16),
                    ElevatedButton(
                      onPressed: () =>
                          ref.read(orderListProvider.notifier).load(
                                status: _selectedStatus,
                              ),
                      child: const Text('再試行'),
                    ),
                  ],
                ),
              ),
              /// データ取得成功時は注文一覧をリスト表示する
              data: (orders) {
                if (orders.isEmpty) {
                  return const Center(child: Text('注文がありません'));
                }
                return ListView.builder(
                  itemCount: orders.length,
                  itemBuilder: (context, index) {
                    final order = orders[index];
                    /// key に注文IDを指定してリスト再構築時の差分検出を最適化する
                    return _OrderListTile(
                      key: ValueKey(order.id),
                      order: order,
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  /// ステータスフィルタ用のチップ一覧を構築する
  /// 選択状態に応じてフィルタリングを実行する
  Widget _buildStatusFilterChips() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: [
            /// 「すべて」チップ: フィルタを解除する
            FilterChip(
              label: const Text('すべて'),
              selected: _selectedStatus == null,
              onSelected: (selected) {
                setState(() => _selectedStatus = null);
                ref.read(orderListProvider.notifier).load();
              },
            ),
            const SizedBox(width: 8),
            /// 各ステータスのフィルタチップを生成する
            ...OrderStatus.values.map((status) {
              return Padding(
                padding: const EdgeInsets.only(right: 8),
                child: FilterChip(
                  label: Text(status.displayName),
                  selected: _selectedStatus == status,
                  onSelected: (selected) {
                    setState(() {
                      _selectedStatus = selected ? status : null;
                    });
                    ref.read(orderListProvider.notifier).load(
                          status: selected ? status : null,
                        );
                  },
                ),
              );
            }),
          ],
        ),
      ),
    );
  }
}

/// 注文一覧の個別タイルウィジェット
/// 注文情報の概要表示とタップによる詳細遷移を提供する
class _OrderListTile extends StatelessWidget {
  final Order order;

  const _OrderListTile({super.key, required this.order});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: ListTile(
        /// 注文IDをタイトルに表示する
        title: Text('注文ID: ${order.id}'),
        /// 顧客ID・合計金額・作成日をサブタイトルに表示する
        subtitle: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('顧客ID: ${order.customerId}'),
            Text(
              '合計: ${order.totalAmount.toStringAsFixed(2)} ${order.currency} · '
              '${_formatDateTime(order.createdAt)}',
            ),
          ],
        ),
        /// ステータスバッジを右側に表示する
        trailing: _StatusBadge(status: order.status),
        /// タップで注文詳細画面へ遷移する
        onTap: () => context.push('/orders/${order.id}'),
      ),
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

/// ステータスバッジウィジェット
/// 注文ステータスに応じた色分けバッジを表示する
class _StatusBadge extends StatelessWidget {
  final OrderStatus status;

  const _StatusBadge({required this.status});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: BoxDecoration(
        color: _getStatusColor(status),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        status.displayName,
        style: const TextStyle(
          color: Colors.white,
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  /// ステータスに応じたバッジ背景色を返す
  Color _getStatusColor(OrderStatus status) {
    return switch (status) {
      OrderStatus.pending => Colors.orange,
      OrderStatus.confirmed => Colors.blue,
      OrderStatus.processing => Colors.purple,
      OrderStatus.shipped => Colors.teal,
      OrderStatus.delivered => Colors.green,
      OrderStatus.cancelled => Colors.red,
    };
  }
}
