/// 注文サービスのデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する

/// 注文ステータスの列挙型
/// 注文のライフサイクルにおける各状態を定義する
enum OrderStatus {
  /// 保留中: 注文が作成された直後の状態
  pending,
  /// 確認済み: 注文内容が確認された状態
  confirmed,
  /// 処理中: 注文が処理されている状態
  processing,
  /// 発送済み: 商品が発送された状態
  shipped,
  /// 配達完了: 商品が顧客に届いた状態
  delivered,
  /// キャンセル: 注文がキャンセルされた状態
  cancelled;

  /// 文字列からOrderStatusに変換する
  static OrderStatus fromString(String value) {
    return OrderStatus.values.firstWhere(
      (e) => e.name == value,
      orElse: () => OrderStatus.pending,
    );
  }

  /// ステータスの日本語表示名を返す
  String get displayName {
    return switch (this) {
      OrderStatus.pending => '保留中',
      OrderStatus.confirmed => '確認済み',
      OrderStatus.processing => '処理中',
      OrderStatus.shipped => '発送済み',
      OrderStatus.delivered => '配達完了',
      OrderStatus.cancelled => 'キャンセル',
    };
  }
}

/// 注文明細モデル
/// 注文に含まれる個別商品の情報を表す
class OrderItem {
  final String productId;
  final String productName;
  final int quantity;
  final double unitPrice;
  final double subtotal;

  const OrderItem({
    required this.productId,
    required this.productName,
    required this.quantity,
    required this.unitPrice,
    required this.subtotal,
  });

  /// JSONマップからOrderItemインスタンスを生成する
  factory OrderItem.fromJson(Map<String, dynamic> json) {
    return OrderItem(
      productId: json['product_id'] as String,
      productName: json['product_name'] as String,
      quantity: json['quantity'] as int,
      unitPrice: (json['unit_price'] as num).toDouble(),
      subtotal: (json['subtotal'] as num).toDouble(),
    );
  }

  /// OrderItemインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'product_id': productId,
      'product_name': productName,
      'quantity': quantity,
      'unit_price': unitPrice,
      'subtotal': subtotal,
    };
  }
}

/// 注文モデル
/// 顧客からの注文情報を表す
class Order {
  final String id;
  final String customerId;
  final OrderStatus status;
  final double totalAmount;
  final String currency;
  final List<OrderItem> items;
  final String? notes;
  final int version;
  final DateTime createdAt;
  final DateTime updatedAt;

  const Order({
    required this.id,
    required this.customerId,
    required this.status,
    required this.totalAmount,
    required this.currency,
    required this.items,
    this.notes,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからOrderインスタンスを生成する
  factory Order.fromJson(Map<String, dynamic> json) {
    return Order(
      id: json['id'] as String,
      customerId: json['customer_id'] as String,
      status: OrderStatus.fromString(json['status'] as String),
      totalAmount: (json['total_amount'] as num).toDouble(),
      currency: json['currency'] as String,
      items: (json['items'] as List<dynamic>)
          .map((item) => OrderItem.fromJson(item as Map<String, dynamic>))
          .toList(),
      notes: json['notes'] as String?,
      version: json['version'] as int,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// OrderインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'customer_id': customerId,
      'status': status.name,
      'total_amount': totalAmount,
      'currency': currency,
      'items': items.map((item) => item.toJson()).toList(),
      'notes': notes,
      'version': version,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// 注文作成時の入力モデル
/// 新規注文作成に必要なフィールドのみを定義する
class CreateOrderInput {
  final String customerId;
  final String currency;
  final List<CreateOrderItemInput> items;
  final String? notes;

  const CreateOrderInput({
    required this.customerId,
    required this.currency,
    required this.items,
    this.notes,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'customer_id': customerId,
      'currency': currency,
      'items': items.map((item) => item.toJson()).toList(),
      'notes': notes,
    };
  }
}

/// 注文明細作成時の入力モデル
/// 新規注文明細に必要なフィールドのみを定義する
class CreateOrderItemInput {
  final String productId;
  final String productName;
  final int quantity;
  final double unitPrice;

  const CreateOrderItemInput({
    required this.productId,
    required this.productName,
    required this.quantity,
    required this.unitPrice,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'product_id': productId,
      'product_name': productName,
      'quantity': quantity,
      'unit_price': unitPrice,
    };
  }
}

/// 注文ステータス更新時の入力モデル
/// 更新対象のステータスのみを定義する
class UpdateOrderStatusInput {
  final OrderStatus status;

  const UpdateOrderStatusInput({
    required this.status,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'status': status.name,
    };
  }
}
