/// コード生成の結果を保持するクラス。
/// 生成されたファイルとスキップされたファイルのリストを管理する。
class GenerateResult {
  /// 生成されたファイルパスのリスト
  final List<String> created;

  /// スキップされたファイルパスのリスト（既に存在するなど）
  final List<String> skipped;

  const GenerateResult({
    required this.created,
    required this.skipped,
  });

  /// 生成されたファイル数を返す。
  int get createdCount => created.length;

  /// スキップされたファイル数を返す。
  int get skippedCount => skipped.length;

  @override
  String toString() =>
      'GenerateResult(created: $createdCount, skipped: $skippedCount)';
}
