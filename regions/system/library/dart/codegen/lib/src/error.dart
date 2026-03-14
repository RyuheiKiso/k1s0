/// コード生成エラーの基底クラス。
/// メッセージとエラーコードを保持する。
class CodegenError implements Exception {
  /// エラーメッセージ
  final String message;

  /// エラーコード（分類用）
  final String code;

  const CodegenError(this.message, {this.code = 'CODEGEN_ERROR'});

  @override
  String toString() => 'CodegenError($code): $message';
}

/// 設定バリデーションエラー。
/// ScaffoldConfig の検証失敗時にスローされる。
class ConfigError extends CodegenError {
  const ConfigError(super.message) : super(code: 'CONFIG_ERROR');
}

/// テンプレート処理エラー。
/// テンプレートのレンダリング失敗時にスローされる。
class TemplateError extends CodegenError {
  const TemplateError(super.message) : super(code: 'TEMPLATE_ERROR');
}

/// 入出力エラー。
/// ファイル読み書き失敗時にスローされる。
class IoError extends CodegenError {
  const IoError(super.message) : super(code: 'IO_ERROR');
}
