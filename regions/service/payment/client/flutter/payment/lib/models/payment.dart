/// 決済のデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する
library;

/// 決済ステータスの列挙型
/// 決済のライフサイクル状態を表す
enum PaymentStatus {
  /// 決済開始待ち
  pending,
  /// 処理中
  processing,
  /// 完了
  completed,
  /// 失敗
  failed,
  /// 返金済み
  refunded;

  /// 文字列からPaymentStatusに変換する
  static PaymentStatus fromString(String value) {
    return PaymentStatus.values.firstWhere(
      (e) => e.name == value,
      orElse: () => PaymentStatus.pending,
    );
  }

  /// 日本語の表示ラベルを返す
  String get label {
    return switch (this) {
      PaymentStatus.pending => '保留中',
      PaymentStatus.processing => '処理中',
      PaymentStatus.completed => '完了',
      PaymentStatus.failed => '失敗',
      PaymentStatus.refunded => '返金済み',
    };
  }
}

/// 決済方法の列挙型
/// 利用可能な決済手段を表す
enum PaymentMethod {
  /// クレジットカード
  creditCard('credit_card'),
  /// 銀行振込
  bankTransfer('bank_transfer'),
  /// コンビニ払い
  convenienceStore('convenience_store'),
  /// 電子ウォレット
  eWallet('e_wallet');

  /// APIで使用するスネークケースの値
  final String value;

  const PaymentMethod(this.value);

  /// 文字列からPaymentMethodに変換する
  static PaymentMethod fromString(String value) {
    return PaymentMethod.values.firstWhere(
      (e) => e.value == value,
      orElse: () => PaymentMethod.creditCard,
    );
  }

  /// 日本語の表示ラベルを返す
  String get label {
    return switch (this) {
      PaymentMethod.creditCard => 'クレジットカード',
      PaymentMethod.bankTransfer => '銀行振込',
      PaymentMethod.convenienceStore => 'コンビニ払い',
      PaymentMethod.eWallet => '電子ウォレット',
    };
  }
}

/// 決済モデル
/// 決済処理の情報を表す
class Payment {
  final String id;
  final String orderId;
  final String customerId;
  final double amount;
  final String currency;
  final PaymentStatus status;
  final PaymentMethod paymentMethod;
  final String? transactionId;
  final String? failureReason;
  final double? refundAmount;
  final DateTime createdAt;
  final DateTime updatedAt;

  const Payment({
    required this.id,
    required this.orderId,
    required this.customerId,
    required this.amount,
    required this.currency,
    required this.status,
    required this.paymentMethod,
    this.transactionId,
    this.failureReason,
    this.refundAmount,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからPaymentインスタンスを生成する
  factory Payment.fromJson(Map<String, dynamic> json) {
    return Payment(
      id: json['id'] as String,
      orderId: json['order_id'] as String,
      customerId: json['customer_id'] as String,
      amount: (json['amount'] as num).toDouble(),
      currency: json['currency'] as String,
      status: PaymentStatus.fromString(json['status'] as String),
      paymentMethod: PaymentMethod.fromString(json['payment_method'] as String),
      transactionId: json['transaction_id'] as String?,
      failureReason: json['failure_reason'] as String?,
      refundAmount: json['refund_amount'] != null
          ? (json['refund_amount'] as num).toDouble()
          : null,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// PaymentインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'order_id': orderId,
      'customer_id': customerId,
      'amount': amount,
      'currency': currency,
      'status': status.name,
      'payment_method': paymentMethod.value,
      'transaction_id': transactionId,
      'failure_reason': failureReason,
      'refund_amount': refundAmount,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// 決済開始時の入力モデル
/// 新規決済を開始するために必要なフィールドのみを定義する
class InitiatePaymentInput {
  final String orderId;
  final String customerId;
  final double amount;
  final String? currency;
  final PaymentMethod paymentMethod;

  const InitiatePaymentInput({
    required this.orderId,
    required this.customerId,
    required this.amount,
    this.currency,
    required this.paymentMethod,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{
      'order_id': orderId,
      'customer_id': customerId,
      'amount': amount,
      'payment_method': paymentMethod.value,
    };
    if (currency != null) {
      json['currency'] = currency;
    }
    return json;
  }
}
