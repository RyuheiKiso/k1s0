import 'package:meta/meta.dart';

/// アップデートの種類。
/// checkForUpdate の結果として使用され、UIの表示方針を決定する。
enum UpdateType {
  /// アップデートなし。現在のバージョンが最新または最低バージョン以上。
  none,

  /// 任意のアップデート。新しいバージョンが存在するがスキップ可能。
  optional,

  /// 必須のアップデート。現在バージョンでは動作継続不可のため強制適用が必要。
  mandatory,
}

/// アプリのバージョン情報。
/// レジストリサーバーから取得した情報をアプリ向けに整形したモデル。
@immutable
class AppVersionInfo {
  /// 最新バージョン。サーバーで公開されている最新のバージョン番号。
  final String latestVersion;

  /// 最低限必要なバージョン。これを下回る場合は必須アップデートとなる。
  final String minimumVersion;

  /// リリースノート。最新バージョンの変更内容を説明するテキスト。
  final String? releaseNotes;

  /// 強制アップデートかどうか。true の場合は必ず mandatory として扱われる。
  final bool mandatory;

  /// ストアURL。iOS/Android の場合にアプリストアへのリンクが格納される。
  final String? storeUrl;

  /// リリース日時。最新バージョンがサーバーに公開された日時。
  final DateTime? publishedAt;

  /// 対象プラットフォーム。'ios', 'android', 'windows' などの文字列。
  final String? platform;

  /// 対象アーキテクチャ。'amd64', 'arm64' などの文字列。
  final String? arch;

  /// ファイルサイズ（バイト）。ダウンロード前のサイズ表示に使用する。
  final int? sizeBytes;

  /// SHA-256チェックサム。ダウンロード後のファイル整合性検証に使用する。
  final String? checksumSha256;

  /// ダウンロードURL。アーティファクトを取得するための署名付きURL。
  final String? downloadUrl;

  /// ダウンロードURLの有効期限（秒）。期限切れ前に再取得が必要。
  final int? expiresIn;

  /// [AppVersionInfo] を生成する。
  const AppVersionInfo({
    required this.latestVersion,
    required this.minimumVersion,
    this.releaseNotes,
    this.mandatory = false,
    this.storeUrl,
    this.publishedAt,
    this.platform,
    this.arch,
    this.sizeBytes,
    this.checksumSha256,
    this.downloadUrl,
    this.expiresIn,
  });

  /// 指定フィールドを上書きした新しいインスタンスを返す。
  /// 省略したフィールドは既存の値をそのまま引き継ぐ。
  AppVersionInfo copyWith({
    String? latestVersion,
    String? minimumVersion,
    String? releaseNotes,
    bool? mandatory,
    String? storeUrl,
    DateTime? publishedAt,
    String? platform,
    String? arch,
    int? sizeBytes,
    String? checksumSha256,
    String? downloadUrl,
    int? expiresIn,
  }) {
    return AppVersionInfo(
      latestVersion: latestVersion ?? this.latestVersion,
      minimumVersion: minimumVersion ?? this.minimumVersion,
      releaseNotes: releaseNotes ?? this.releaseNotes,
      mandatory: mandatory ?? this.mandatory,
      storeUrl: storeUrl ?? this.storeUrl,
      publishedAt: publishedAt ?? this.publishedAt,
      platform: platform ?? this.platform,
      arch: arch ?? this.arch,
      sizeBytes: sizeBytes ?? this.sizeBytes,
      checksumSha256: checksumSha256 ?? this.checksumSha256,
      downloadUrl: downloadUrl ?? this.downloadUrl,
      expiresIn: expiresIn ?? this.expiresIn,
    );
  }
}

/// ダウンロード成果物の情報。
/// getDownloadInfo で取得した署名付きURLとメタデータを保持する。
@immutable
class DownloadArtifactInfo {
  /// ダウンロードURL。署名付きの一時URLであり、有効期限がある。
  final String downloadUrl;

  /// ダウンロードURLの有効期限（秒）。この秒数が経過すると URLが無効になる。
  final int expiresIn;

  /// SHA-256チェックサム。ダウンロード後のファイル整合性検証に使用する。
  final String checksumSha256;

  /// ファイルサイズ（バイト）。ダウンロードのプログレス表示に使用する。
  final int? sizeBytes;

  /// [DownloadArtifactInfo] を生成する。
  const DownloadArtifactInfo({
    required this.downloadUrl,
    required this.expiresIn,
    required this.checksumSha256,
    this.sizeBytes,
  });
}

/// アップデート確認の結果。
/// checkForUpdate の戻り値として使用し、UIへのアップデート案内に必要な情報を提供する。
@immutable
class UpdateCheckResult {
  /// アップデートの種類。none / optional / mandatory のいずれか。
  final UpdateType type;

  /// 現在のバージョン。デバイスにインストールされているバージョン番号。
  final String currentVersion;

  /// サーバーから取得したバージョン情報。UIへの表示に使用する。
  final AppVersionInfo versionInfo;

  /// [UpdateCheckResult] を生成する。
  const UpdateCheckResult({
    required this.type,
    required this.currentVersion,
    required this.versionInfo,
  });

  /// アップデートが必要かどうか。optional または mandatory の場合に true を返す。
  bool get needsUpdate => type != UpdateType.none;

  /// 強制アップデートかどうか。mandatory の場合のみ true を返す。
  bool get isMandatory => type == UpdateType.mandatory;
}
