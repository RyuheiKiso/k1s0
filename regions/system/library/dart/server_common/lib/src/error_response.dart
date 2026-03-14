import 'package:uuid/uuid.dart';

import 'error_code.dart';
import 'error_detail.dart';

/// UUID 生成用の定数インスタンス
const _uuid = Uuid();

/// エラーレスポンスの本体。
/// コード、メッセージ、リクエストID、詳細情報を保持する。
class ErrorBody {
  /// エラーコード
  final ErrorCode code;

  /// エラーメッセージ（人間可読）
  final String message;

  /// リクエスト追跡用 UUID
  String requestId;

  /// フィールドレベルのエラー詳細（オプション）
  final List<ErrorDetail> details;

  ErrorBody({
    required this.code,
    required this.message,
    required this.requestId,
    this.details = const [],
  });

  /// JSON マップに変換する。
  /// details が空の場合は出力に含めない。
  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{
      'code': code.value,
      'message': message,
      'request_id': requestId,
    };
    if (details.isNotEmpty) {
      json['details'] = details.map((d) => d.toJson()).toList();
    }
    return json;
  }
}

/// エラーレスポンスのエンベロープ。
/// `{ "error": ... }` 構造でエラー本体をラップする。
class ErrorResponse {
  /// エラー本体
  final ErrorBody error;

  ErrorResponse._(this.error);

  /// コードとメッセージからエラーレスポンスを生成する。
  /// リクエスト ID は UUID v4 で自動生成される。
  factory ErrorResponse(ErrorCode code, String message) {
    return ErrorResponse._(ErrorBody(
      code: code,
      message: message,
      requestId: _uuid.v4(),
    ));
  }

  /// コード、メッセージ、詳細情報からエラーレスポンスを生成する。
  factory ErrorResponse.withDetails(
    ErrorCode code,
    String message,
    List<ErrorDetail> details,
  ) {
    return ErrorResponse._(ErrorBody(
      code: code,
      message: message,
      requestId: _uuid.v4(),
      details: details,
    ));
  }

  /// リクエスト ID を上書きする。
  /// 相関 ID が既に利用可能な場合に使用する。
  ErrorResponse withRequestId(String requestId) {
    error.requestId = requestId;
    return this;
  }

  /// JSON マップに変換する。
  Map<String, dynamic> toJson() => {
        'error': error.toJson(),
      };
}
