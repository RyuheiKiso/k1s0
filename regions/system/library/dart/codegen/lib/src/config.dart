import 'error.dart';

/// k1s0 アーキテクチャのティア（階層）を表す列挙型。
enum Tier {
  /// システム層
  system,

  /// ビジネス層
  business,

  /// サービス層
  service;

  /// 表示用の名前を返す。
  String get displayName {
    switch (this) {
      case Tier.system:
        return 'system';
      case Tier.business:
        return 'business';
      case Tier.service:
        return 'service';
    }
  }
}

/// API スタイルを表す列挙型。
enum ApiStyle {
  /// REST API のみ
  rest,

  /// gRPC のみ
  grpc,

  /// REST と gRPC の両方
  both;

  /// gRPC を含むかどうかを返す。
  bool get hasGrpc => this == ApiStyle.grpc || this == ApiStyle.both;

  /// REST を含むかどうかを返す。
  bool get hasRest => this == ApiStyle.rest || this == ApiStyle.both;
}

/// データベース種別を表す列挙型。
enum DatabaseType {
  /// PostgreSQL
  postgres,

  /// データベースなし
  none;

  /// データベースを使用するかどうかを返す。
  bool get hasDatabase => this != DatabaseType.none;
}

/// ケバブケースの正規表現パターン。
/// 小文字英数字とハイフンのみ許可し、先頭・末尾のハイフンと連続ハイフンを禁止する。
final _kebabCaseRegExp = RegExp(r'^[a-z0-9]+(-[a-z0-9]+)*$');

/// スキャフォールド生成の設定。
/// サーバー名、ティア、APIスタイル、データベース種別などを保持する。
class ScaffoldConfig {
  /// サーバー名（ケバブケース、例: "user-profile"）
  final String name;

  /// アーキテクチャティア
  final Tier tier;

  /// API スタイル
  final ApiStyle apiStyle;

  /// データベース種別
  final DatabaseType database;

  /// サーバーの説明
  final String description;

  /// gRPC サービスの .proto ファイルパス（オプション）
  final String? protoPath;

  /// クライアント SDK を同時に生成するかどうか
  final bool generateClient;

  const ScaffoldConfig({
    required this.name,
    required this.tier,
    required this.apiStyle,
    required this.database,
    required this.description,
    this.protoPath,
    this.generateClient = false,
  });

  /// 設定を検証する。
  /// 名前が空または不正なケバブケースの場合に ConfigError をスローする。
  void validate() {
    if (name.isEmpty) {
      throw const ConfigError('name must not be empty');
    }
    if (!_kebabCaseRegExp.hasMatch(name)) {
      throw const ConfigError(
        'name must be kebab-case (lowercase ascii, digits, hyphens)',
      );
    }
  }

  /// gRPC を含むかどうかの便利ゲッター。
  bool hasGrpc() => apiStyle.hasGrpc;

  /// REST を含むかどうかの便利ゲッター。
  bool hasRest() => apiStyle.hasRest;

  /// データベースを使用するかどうかの便利ゲッター。
  bool hasDatabase() => database.hasDatabase;
}
