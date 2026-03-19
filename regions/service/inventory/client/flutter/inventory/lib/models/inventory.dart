/// 在庫管理のデータモデル定義
/// freezed/json_serializableを使用せず、手書きのfromJson/toJsonで実装する
library;

/// 在庫ステータスを表す列挙型
/// 在庫の状態に応じて表示色やアクションを切り替えるために使用する
enum InventoryStatus {
  /// 在庫あり: 十分な在庫がある状態
  inStock,
  /// 低在庫: 発注点を下回っている状態
  lowStock,
  /// 在庫切れ: 在庫数が0の状態
  outOfStock;

  /// 文字列からInventoryStatusに変換する
  static InventoryStatus fromString(String value) {
    return switch (value) {
      'in_stock' => InventoryStatus.inStock,
      'low_stock' => InventoryStatus.lowStock,
      'out_of_stock' => InventoryStatus.outOfStock,
      _ => InventoryStatus.inStock,
    };
  }

  /// InventoryStatusをAPI用の文字列に変換する
  String toJsonString() {
    return switch (this) {
      InventoryStatus.inStock => 'in_stock',
      InventoryStatus.lowStock => 'low_stock',
      InventoryStatus.outOfStock => 'out_of_stock',
    };
  }

  /// ステータスの日本語表示名を返す
  String get displayName {
    return switch (this) {
      InventoryStatus.inStock => '在庫あり',
      InventoryStatus.lowStock => '低在庫',
      InventoryStatus.outOfStock => '在庫切れ',
    };
  }
}

/// 在庫アイテムモデル
/// 倉庫内の商品在庫情報を表す
class InventoryItem {
  final String id;
  final String productId;
  final String productName;
  final String warehouseId;
  final String warehouseName;
  final int quantityAvailable;
  final int quantityReserved;
  final int reorderPoint;
  final InventoryStatus status;
  final int version;
  final DateTime createdAt;
  final DateTime updatedAt;

  const InventoryItem({
    required this.id,
    required this.productId,
    required this.productName,
    required this.warehouseId,
    required this.warehouseName,
    required this.quantityAvailable,
    required this.quantityReserved,
    required this.reorderPoint,
    required this.status,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
  });

  /// JSONマップからInventoryItemインスタンスを生成する
  factory InventoryItem.fromJson(Map<String, dynamic> json) {
    return InventoryItem(
      id: json['id'] as String,
      productId: json['product_id'] as String,
      productName: json['product_name'] as String,
      warehouseId: json['warehouse_id'] as String,
      warehouseName: json['warehouse_name'] as String,
      quantityAvailable: json['quantity_available'] as int,
      quantityReserved: json['quantity_reserved'] as int,
      reorderPoint: json['reorder_point'] as int,
      status: InventoryStatus.fromString(json['status'] as String),
      version: json['version'] as int,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  /// InventoryItemインスタンスをJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'product_id': productId,
      'product_name': productName,
      'warehouse_id': warehouseId,
      'warehouse_name': warehouseName,
      'quantity_available': quantityAvailable,
      'quantity_reserved': quantityReserved,
      'reorder_point': reorderPoint,
      'status': status.toJsonString(),
      'version': version,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }
}

/// 在庫操作（引当・引当解除）の入力モデル
/// 商品の在庫を引き当てまたは解除する際に使用する
class StockOperation {
  final String productId;
  final String warehouseId;
  final int quantity;

  const StockOperation({
    required this.productId,
    required this.warehouseId,
    required this.quantity,
  });

  /// 入力データをAPIリクエスト用のJSONマップに変換する
  Map<String, dynamic> toJson() {
    return {
      'product_id': productId,
      'warehouse_id': warehouseId,
      'quantity': quantity,
    };
  }
}

/// 在庫更新の入力モデル
/// 在庫数量や発注点を更新する際に使用する
class UpdateStockInput {
  final int? quantityAvailable;
  final int? reorderPoint;

  const UpdateStockInput({
    this.quantityAvailable,
    this.reorderPoint,
  });

  /// 更新データをAPIリクエスト用のJSONマップに変換する
  /// nullのフィールドは除外する
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{};
    if (quantityAvailable != null) json['quantity_available'] = quantityAvailable;
    if (reorderPoint != null) json['reorder_point'] = reorderPoint;
    return json;
  }
}
