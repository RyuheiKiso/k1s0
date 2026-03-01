import 'model.dart';
import 'client.dart';

/// MockFileClient は [FileClient] を実装したテスト用モッククラス。
///
/// 各メソッドはコールバック関数（`onXxx`）で動作をオーバーライドできる。
/// コールバックが未設定の場合はデフォルト実装（空応答）が使われる。
/// 呼び出し履歴は [calls] リストに記録される。
///
/// ```dart
/// final mock = MockFileClient();
///
/// mock.onGetMetadata = (path) async => FileMetadata(
///   path: path,
///   sizeBytes: 1024,
///   contentType: 'image/png',
///   etag: 'abc123',
///   lastModified: DateTime.now(),
///   tags: {},
/// );
///
/// final meta = await mock.getMetadata('uploads/image.png');
/// expect(meta.contentType, 'image/png');
/// expect(mock.calls, contains('getMetadata:uploads/image.png'));
/// ```
class MockFileClient implements FileClient {
  /// 記録された呼び出し一覧。書式: `'methodName:arg1,arg2,...'`
  final List<String> calls = [];

  // ---------------------------------------------------------------------------
  // コールバックプロパティ
  // ---------------------------------------------------------------------------

  Future<PresignedUrl> Function(String path, String contentType, Duration expiresIn)?
      onGenerateUploadUrl;

  Future<PresignedUrl> Function(String path, Duration expiresIn)?
      onGenerateDownloadUrl;

  Future<void> Function(String path)? onDelete;

  Future<FileMetadata> Function(String path)? onGetMetadata;

  Future<List<FileMetadata>> Function(String prefix)? onList;

  Future<void> Function(String src, String dst)? onCopy;

  // ---------------------------------------------------------------------------
  // FileClient 実装
  // ---------------------------------------------------------------------------

  @override
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  ) async {
    calls.add('generateUploadUrl:$path,$contentType');
    if (onGenerateUploadUrl != null) {
      return onGenerateUploadUrl!(path, contentType, expiresIn);
    }
    return PresignedUrl(
      url: 'https://mock.example.com/upload/$path',
      method: 'PUT',
      expiresAt: DateTime.now().add(expiresIn),
      headers: {},
    );
  }

  @override
  Future<PresignedUrl> generateDownloadUrl(
    String path,
    Duration expiresIn,
  ) async {
    calls.add('generateDownloadUrl:$path');
    if (onGenerateDownloadUrl != null) {
      return onGenerateDownloadUrl!(path, expiresIn);
    }
    return PresignedUrl(
      url: 'https://mock.example.com/download/$path',
      method: 'GET',
      expiresAt: DateTime.now().add(expiresIn),
      headers: {},
    );
  }

  @override
  Future<void> delete(String path) async {
    calls.add('delete:$path');
    if (onDelete != null) {
      return onDelete!(path);
    }
  }

  @override
  Future<FileMetadata> getMetadata(String path) async {
    calls.add('getMetadata:$path');
    if (onGetMetadata != null) {
      return onGetMetadata!(path);
    }
    return FileMetadata(
      path: path,
      sizeBytes: 0,
      contentType: 'application/octet-stream',
      etag: '',
      lastModified: DateTime.now(),
      tags: {},
    );
  }

  @override
  Future<List<FileMetadata>> list(String prefix) async {
    calls.add('list:$prefix');
    if (onList != null) {
      return onList!(prefix);
    }
    return [];
  }

  @override
  Future<void> copy(String src, String dst) async {
    calls.add('copy:$src,$dst');
    if (onCopy != null) {
      return onCopy!(src, dst);
    }
  }
}
