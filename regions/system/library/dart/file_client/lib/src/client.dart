import 'model.dart';

abstract class FileClient {
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  );
  Future<PresignedUrl> generateDownloadUrl(String path, Duration expiresIn);
  Future<void> delete(String path);
  Future<FileMetadata> getMetadata(String path);
  Future<List<FileMetadata>> list(String prefix);
  Future<void> copy(String src, String dst);
}

class InMemoryFileClient implements FileClient {
  final Map<String, FileMetadata> _files = {};

  List<FileMetadata> get storedFiles => List.unmodifiable(_files.values);

  @override
  Future<PresignedUrl> generateUploadUrl(
    String path,
    String contentType,
    Duration expiresIn,
  ) async {
    _files[path] = FileMetadata(
      path: path,
      sizeBytes: 0,
      contentType: contentType,
      etag: '',
      lastModified: DateTime.now(),
      tags: {},
    );
    return PresignedUrl(
      url: 'https://storage.example.com/upload/$path',
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
    if (!_files.containsKey(path)) {
      throw FileClientError('File not found: $path', 'NOT_FOUND');
    }
    return PresignedUrl(
      url: 'https://storage.example.com/download/$path',
      method: 'GET',
      expiresAt: DateTime.now().add(expiresIn),
      headers: {},
    );
  }

  @override
  Future<void> delete(String path) async {
    if (!_files.containsKey(path)) {
      throw FileClientError('File not found: $path', 'NOT_FOUND');
    }
    _files.remove(path);
  }

  @override
  Future<FileMetadata> getMetadata(String path) async {
    final meta = _files[path];
    if (meta == null) {
      throw FileClientError('File not found: $path', 'NOT_FOUND');
    }
    return meta;
  }

  @override
  Future<List<FileMetadata>> list(String prefix) async {
    return _files.values.where((f) => f.path.startsWith(prefix)).toList();
  }

  @override
  Future<void> copy(String src, String dst) async {
    final source = _files[src];
    if (source == null) {
      throw FileClientError('File not found: $src', 'NOT_FOUND');
    }
    _files[dst] = FileMetadata(
      path: dst,
      sizeBytes: source.sizeBytes,
      contentType: source.contentType,
      etag: source.etag,
      lastModified: source.lastModified,
      tags: Map.of(source.tags),
    );
  }
}
