import 'dart:io';

import 'package:crypto/crypto.dart';

import 'error.dart';

/// ファイルのSHA-256チェックサムを検証するユーティリティ。
/// ダウンロードしたファイルの改ざん検知に使用する。
class ChecksumVerifier {
  /// ファイルのSHA-256チェックサムを検証する。一致する場合は `true` を返す。
  /// [filePath] の実際のチェックサムと [expectedChecksum] を比較する。
  /// 比較時は大文字小文字を区別しないよう小文字に正規化する。
  static Future<bool> verify(String filePath, String expectedChecksum) async {
    final actual = await calculate(filePath);
    return actual == expectedChecksum.toLowerCase();
  }

  /// ファイルのSHA-256チェックサムを検証し、不一致の場合は [ChecksumError] をスローする。
  /// ダウンロード後のファイル整合性チェックで使用することを想定している。
  static Future<void> verifyOrThrow(
    String filePath,
    String expectedChecksum,
  ) async {
    final verified = await verify(filePath, expectedChecksum);
    if (!verified) {
      throw const ChecksumError('Downloaded file checksum did not match.');
    }
  }

  /// ファイルのSHA-256チェックサムを計算して返す。
  /// ファイルをストリームとして読み込み、SHA-256ダイジェストを16進数文字列で返す。
  static Future<String> calculate(String filePath) async {
    final file = File(filePath);
    // ファイルをストリームとして読み込み、SHA-256ハッシュを計算する。
    final digest = await sha256.bind(file.openRead()).last;
    return digest.toString();
  }
}
