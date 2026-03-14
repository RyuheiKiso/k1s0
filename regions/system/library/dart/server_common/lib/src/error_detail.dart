/// エラー詳細情報。
/// フィールドレベルのバリデーションエラー等の追加コンテキストを提供する。
/// REST-API設計.md D-007 仕様に準拠する。
class ErrorDetail {
  /// エラーが発生したフィールド名
  final String field;

  /// エラーの理由（機械可読）
  final String reason;

  /// エラーメッセージ（人間可読）
  final String message;

  const ErrorDetail({
    required this.field,
    required this.reason,
    required this.message,
  });

  /// JSON マップに変換する。
  Map<String, dynamic> toJson() => {
        'field': field,
        'reason': reason,
        'message': message,
      };
}
