import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../models/payment.dart';
import '../providers/payment_provider.dart';

/// 決済作成画面
/// 新規決済の開始フォームを表示する画面
class PaymentFormScreen extends ConsumerStatefulWidget {
  const PaymentFormScreen({super.key});

  @override
  ConsumerState<PaymentFormScreen> createState() => _PaymentFormScreenState();
}

class _PaymentFormScreenState extends ConsumerState<PaymentFormScreen> {
  /// フォームの検証キー
  final _formKey = GlobalKey<FormState>();

  /// 注文ID入力用コントローラー
  final _orderIdController = TextEditingController();

  /// 顧客ID入力用コントローラー
  final _customerIdController = TextEditingController();

  /// 金額入力用コントローラー
  final _amountController = TextEditingController();

  /// 通貨入力用コントローラー（デフォルト: JPY）
  final _currencyController = TextEditingController(text: 'JPY');

  /// 選択中の決済方法
  PaymentMethod _selectedMethod = PaymentMethod.creditCard;

  /// 送信中フラグ
  bool _isSubmitting = false;

  @override
  void dispose() {
    _orderIdController.dispose();
    _customerIdController.dispose();
    _amountController.dispose();
    _currencyController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('新規決済'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              /// 注文ID入力フィールド
              TextFormField(
                controller: _orderIdController,
                decoration: const InputDecoration(
                  labelText: '注文ID',
                  hintText: '注文IDを入力してください',
                  border: OutlineInputBorder(),
                ),
                validator: (value) {
                  if (value == null || value.trim().isEmpty) {
                    return '注文IDを入力してください';
                  }
                  return null;
                },
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
                    return '顧客IDを入力してください';
                  }
                  return null;
                },
              ),
              const SizedBox(height: 16),

              /// 金額入力フィールド
              TextFormField(
                controller: _amountController,
                decoration: const InputDecoration(
                  labelText: '金額',
                  hintText: '金額を入力してください',
                  border: OutlineInputBorder(),
                ),
                keyboardType: TextInputType.number,
                validator: (value) {
                  if (value == null || value.trim().isEmpty) {
                    return '金額を入力してください';
                  }
                  final amount = double.tryParse(value);
                  if (amount == null || amount <= 0) {
                    return '有効な金額を入力してください';
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
                  hintText: '通貨コードを入力してください（例: JPY）',
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 16),

              /// 決済方法ドロップダウン
              DropdownButtonFormField<PaymentMethod>(
                value: _selectedMethod,
                decoration: const InputDecoration(
                  labelText: '決済方法',
                  border: OutlineInputBorder(),
                ),
                /// 決済方法の選択肢を動的生成する
                items: PaymentMethod.values.map((method) {
                  return DropdownMenuItem(
                    value: method,
                    child: Text(method.label),
                  );
                }).toList(),
                onChanged: (value) {
                  if (value != null) {
                    setState(() => _selectedMethod = value);
                  }
                },
              ),
              const SizedBox(height: 24),

              /// 決済開始ボタン
              FilledButton.icon(
                onPressed: _isSubmitting ? null : _submitPayment,
                icon: _isSubmitting
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.white,
                        ),
                      )
                    : const Icon(Icons.payment),
                label: Text(_isSubmitting ? '処理中...' : '決済を開始'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  /// 決済開始処理を実行する
  /// フォームバリデーション後にAPIリクエストを送信する
  Future<void> _submitPayment() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() => _isSubmitting = true);

    try {
      final input = InitiatePaymentInput(
        orderId: _orderIdController.text.trim(),
        customerId: _customerIdController.text.trim(),
        amount: double.parse(_amountController.text.trim()),
        currency: _currencyController.text.trim().isNotEmpty
            ? _currencyController.text.trim()
            : null,
        paymentMethod: _selectedMethod,
      );

      await ref.read(paymentListProvider.notifier).initiate(input);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('決済を開始しました')),
        );
        /// 決済一覧画面に戻る
        context.go('/');
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('エラー: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isSubmitting = false);
      }
    }
  }
}
