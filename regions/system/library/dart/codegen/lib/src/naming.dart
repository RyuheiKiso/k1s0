/// ケバブケース文字列をスネークケースに変換する。
/// 例: "auth-server" → "auth_server"
String toSnakeCase(String input) {
  return input.replaceAll('-', '_');
}

/// ケバブケース文字列をパスカルケースに変換する。
/// 例: "auth-server" → "AuthServer"
String toPascalCase(String input) {
  return input.split('-').map((segment) {
    if (segment.isEmpty) return '';
    return segment[0].toUpperCase() + segment.substring(1);
  }).join();
}

/// ケバブケース文字列をそのまま返す（正規化用）。
/// ハイフン以外の区切り文字（アンダースコア等）をハイフンに変換する。
/// 例: "auth_server" → "auth-server"
String toKebabCase(String input) {
  return input.replaceAll('_', '-').toLowerCase();
}

/// ケバブケース文字列をキャメルケースに変換する。
/// 例: "auth-server" → "authServer"
String toCamelCase(String input) {
  final pascal = toPascalCase(input);
  if (pascal.isEmpty) return '';
  return pascal[0].toLowerCase() + pascal.substring(1);
}
