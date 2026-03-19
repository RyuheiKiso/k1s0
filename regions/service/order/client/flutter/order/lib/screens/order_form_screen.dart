import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/order.dart';
import '../providers/order_provider.dart';

/// 注文作成画面
/// 新規注文の入力フォームを表示し、注文を作成する
class OrderFormScreen extends ConsumerStatefulWidget {
  const OrderFormScreen({super.key});

  @override
  ConsumerState<OrderFormScreen> createState() => _OrderFormScreenState();
}

class _OrderFormScreenState extends ConsumerState<OrderFormScreen> {
  /// フォームバリデーション用のキー
  final _formKey = GlobalKey<FormState>();

  /// 顧客ID入力用コントローラー
  final _customerIdController = TextEditingController();

  /// 通貨入力用コントローラー（デフォルト: JPY）
  final _currencyController = TextEditingController(text: 'JPY');

  /// 備考入力用コントローラー
  final _notesController = TextEditingController();

  /// 注文明細の動的リスト
  /// 各明細のコントローラーをマップで管理する
  final List<Map<String, TextEditingController>> _itemControllers = [];

  /// 各明細の小計を保持するリスト
  final List<double> _itemSubtotals = [];

  @override
  void initState() {
    super.initState();
    /// 初期状態で1つの空の明細行を追加する
    _addItem();
  }

  @override
  void dispose() {
    _customerIdController.dispose();
    _currencyController.dispose();
    _notesController.dispose();
    /// 全明細のコントローラーを破棄する
    for (final controllers in _itemControllers) {
      for (final c in controllers.values) {
        c.dispose();
      }
    }
    super.dispose();
  }

  /// 新しい明細行を追加する
  void _addItem() {
    setState(() {
      _itemControllers.add({
        'product_id': TextEditingController(),
        'product_name': TextEditingController(),
        'quantity': TextEditingController(text: '1'),
        'unit_price': TextEditingController(text: '0'),
      });
      _itemSubtotals.add(0);
    });
  }

  /// 指定インデックスの明細行を削除する
  /// 最低1行は残す
  void _removeItem(int index) {
    if (_itemControllers.length <= 1) return;
    setState(() {
      for (final c in _itemControllers[index].values) {
        c.dispose();
      }
      _itemControllers.removeAt(index);
      _itemSubtotals.removeAt(index);
    });
  }

  /// 指定インデックスの明細の小計を再計算する
  void _recalculateSubtotal(int index) {
    final quantity =
        int.tryParse(_itemControllers[index]['quantity']!.text) ?? 0;
    final unitPrice =
        double.tryParse(_itemControllers[index]['unit_price']!.text) ?? 0;
    setState(() {
      _itemSubtotals[index] = quantity * unitPrice;
    });
  }

  /// 全明細の合計金額を計算する
  double get _totalAmount {
    double total = 0;
    for (final subtotal in _itemSubtotals) {
      total += subtotal;
    }
    return total;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('新規注文'),
      ),
      body: Form(
        key: _formKey,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              /// 顧客情報セクション
              _buildCustomerSection(context),
              const SizedBox(height: 24),

              /// 注文明細セクション
              _buildItemsSection(context),
              const SizedBox(height: 24),

              /// 備考セクション
              _buildNotesSection(context),
              const SizedBox(height: 24),

              /// 合計金額と送信ボタン
              _buildFooterSection(context),
            ],
          ),
        ),
      ),
    );
  }

  /// 顧客情報入力セクションを構築する
  Widget _buildCustomerSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '顧客情報',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            /// 顧客ID入力フィールド
            TextFormField(
              controller: _customerIdController,
              decoration: const InputDecoration(
                labelText: '顧客ID',
                hintText: '顧客IDを入力してください',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                if (value == null || value.trim().isEmpty) {
                  return '顧客IDは必須です';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),
            /// 通貨入力フィールド
            TextFormField(
              controller: _currencyController,
              decoration: const InputDecoration(
                labelText: '通貨',
                hintText: '通貨コードを入力してください（例: JPY, USD）',
                border: OutlineInputBorder(),
              ),
              validator: (value) {
                if (value == null || value.trim().isEmpty) {
                  return '通貨は必須です';
                }
                return null;
              },
            ),
          ],
        ),
      ),
    );
  }

  /// 注文明細入力セクションを構築する
  Widget _buildItemsSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '注文明細',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                /// 明細行追加ボタン
                FilledButton.icon(
                  onPressed: _addItem,
                  icon: const Icon(Icons.add),
                  label: const Text('明細追加'),
                ),
              ],
            ),
            const SizedBox(height: 16),
            /// 各明細行のフォームを動的に生成する
            ...List.generate(_itemControllers.length, (index) {
              return _buildItemRow(context, index);
            }),
          ],
        ),
      ),
    );
  }

  /// 個別の明細行フォームを構築する
  Widget _buildItemRow(BuildContext context, int index) {
    final controllers = _itemControllers[index];

    return Container(
      margin: const EdgeInsets.only(bottom: 16),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.grey.shade300),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                '明細 #${index + 1}',
                style: const TextStyle(fontWeight: FontWeight.bold),
              ),
              /// 明細行削除ボタン（2行以上の場合のみ表示）
              if (_itemControllers.length > 1)
                IconButton(
                  icon: const Icon(Icons.delete, color: Colors.red),
                  tooltip: '明細削除',
                  onPressed: () => _removeItem(index),
                ),
            ],
          ),
          const SizedBox(height: 8),
          /// 商品IDと商品名を横並びで表示する
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  controller: controllers['product_id'],
                  decoration: const InputDecoration(
                    labelText: '商品ID',
                    border: OutlineInputBorder(),
                  ),
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return '商品IDは必須です';
                    }
                    return null;
                  },
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextFormField(
                  controller: controllers['product_name'],
                  decoration: const InputDecoration(
                    labelText: '商品名',
                    border: OutlineInputBorder(),
                  ),
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return '商品名は必須です';
                    }
                    return null;
                  },
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          /// 数量・単価・小計を横並びで表示する
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  controller: controllers['quantity'],
                  decoration: const InputDecoration(
                    labelText: '数量',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return '数量は必須です';
                    }
                    final qty = int.tryParse(value);
                    if (qty == null || qty <= 0) {
                      return '1以上の整数を入力してください';
                    }
                    return null;
                  },
                  onChanged: (_) => _recalculateSubtotal(index),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextFormField(
                  controller: controllers['unit_price'],
                  decoration: const InputDecoration(
                    labelText: '単価',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType:
                      const TextInputType.numberWithOptions(decimal: true),
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return '単価は必須です';
                    }
                    final price = double.tryParse(value);
                    if (price == null || price < 0) {
                      return '0以上の数値を入力してください';
                    }
                    return null;
                  },
                  onChanged: (_) => _recalculateSubtotal(index),
                ),
              ),
              const SizedBox(width: 8),
              /// 小計表示（自動計算・読み取り専用）
              SizedBox(
                width: 120,
                child: InputDecorator(
                  decoration: const InputDecoration(
                    labelText: '小計',
                    border: OutlineInputBorder(),
                  ),
                  child: Text(
                    _itemSubtotals[index].toStringAsFixed(2),
                    style: const TextStyle(fontWeight: FontWeight.bold),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  /// 備考入力セクションを構築する
  Widget _buildNotesSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '備考',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            /// 備考入力フィールド（複数行対応）
            TextFormField(
              controller: _notesController,
              decoration: const InputDecoration(
                labelText: '備考（任意）',
                hintText: '注文に関する備考を入力してください',
                border: OutlineInputBorder(),
              ),
              maxLines: 3,
            ),
          ],
        ),
      ),
    );
  }

  /// 合計金額と送信ボタンのフッターセクションを構築する
  Widget _buildFooterSection(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            /// 合計金額を大きく表示する
            Text(
              '合計: ${_totalAmount.toStringAsFixed(2)} ${_currencyController.text}',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            /// 注文作成ボタン
            FilledButton.icon(
              onPressed: _submitOrder,
              icon: const Icon(Icons.send),
              label: const Text('注文を作成'),
            ),
          ],
        ),
      ),
    );
  }

  /// 注文をサーバーに送信する
  /// フォームバリデーション後、注文データを構築してAPIに送信する
  void _submitOrder() {
    if (!_formKey.currentState!.validate()) return;

    /// 明細データを構築する
    final items = _itemControllers.map((controllers) {
      return CreateOrderItemInput(
        productId: controllers['product_id']!.text.trim(),
        productName: controllers['product_name']!.text.trim(),
        quantity: int.parse(controllers['quantity']!.text.trim()),
        unitPrice: double.parse(controllers['unit_price']!.text.trim()),
      );
    }).toList();

    /// 注文作成入力データを構築する
    final input = CreateOrderInput(
      customerId: _customerIdController.text.trim(),
      currency: _currencyController.text.trim(),
      items: items,
      notes: _notesController.text.trim().isNotEmpty
          ? _notesController.text.trim()
          : null,
    );

    /// 注文を作成してリストを更新する
    ref.read(orderListProvider.notifier).create(input);

    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('注文を作成しました')),
    );

    /// 注文一覧画面に戻る
    context.go('/');
  }
}
