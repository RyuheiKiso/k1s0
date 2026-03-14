import 'dart:typed_data';

import 'component.dart';
import 'errors.dart';
import 'statestore.dart';

/// ETag 生成用のグローバルカウンター。
/// set が呼ばれるたびにインクリメントし、単調増加する ETag を生成する。
int _etagCounter = 0;

/// StateStore の内部エントリ。値と ETag をペアで保持する。
/// ETag により楽観的並行制御（Optimistic Concurrency Control）を実現する。
class _Entry {
  /// 保存されたバイナリ値。
  final Uint8List value;

  /// エントリのバージョンを示す ETag 文字列。
  final String etag;

  _Entry(this.value, this.etag);
}

/// インメモリ実装の StateStore（状態ストア）コンポーネント。
/// 状態をメモリ上の Map で管理し、ETag を用いた楽観的並行制御をサポートする。
/// テスト・開発環境での利用を想定しており、外部の Redis 等を必要としない。
class InMemoryStateStore implements StateStore {
  /// コンポーネントの識別名。
  @override
  final String name;

  /// コンポーネント種別を示す文字列。
  @override
  final String componentType = 'statestore';

  /// コンポーネントの現在の状態（未初期化 / 準備完了 / クローズ済み）。
  ComponentStatus _status = ComponentStatus.uninitialized;

  /// キーと内部エントリのペアで状態を保持するインメモリストレージ。
  final _entries = <String, _Entry>{};

  /// コンストラクタ。name のデフォルト値は 'inmemory-statestore'。
  InMemoryStateStore({this.name = 'inmemory-statestore'});

  /// コンポーネントを初期化し、状態を ready に移行する。
  @override
  Future<void> init() async { _status = ComponentStatus.ready; }

  /// コンポーネントをクローズし、全エントリを削除して状態を closed に移行する。
  @override
  Future<void> close() async { _entries.clear(); _status = ComponentStatus.closed; }

  /// コンポーネントの現在の状態を返す。
  @override
  Future<ComponentStatus> status() async => _status;

  /// バックエンドが memory であることを示すメタデータを返す。
  @override
  Map<String, String> metadata() => {'backend': 'memory'};

  /// 指定キーに対応する状態エントリを取得する。
  /// エントリが存在しない場合は null を返す。
  @override
  Future<StateEntry?> get(String key) async {
    final e = _entries[key];
    if (e == null) return null;
    return StateEntry(key: key, value: e.value, etag: e.etag);
  }

  /// 指定キーに値を保存し、新しい ETag を返す。
  /// etag が指定された場合は楽観的並行制御を行い、不一致時は ETagMismatchError をスローする。
  @override
  Future<String> set(String key, Uint8List value, {String? etag}) async {
    final existing = _entries[key];
    if (etag != null) {
      // etag が指定されているのにエントリが存在しない場合は不一致エラーをスローする。
      if (existing == null) throw ETagMismatchError(key: key, expected: etag, actual: '');
      // 既存の etag と指定された etag が一致しない場合は不一致エラーをスローする。
      if (existing.etag != etag) throw ETagMismatchError(key: key, expected: etag, actual: existing.etag);
    }
    // カウンターをインクリメントして新しい ETag を生成し、エントリを上書き保存する。
    final newETag = '${++_etagCounter}';
    _entries[key] = _Entry(value, newETag);
    return newETag;
  }

  /// 指定キーのエントリを削除する。
  /// エントリが存在しない場合は何もしない。
  /// etag が指定された場合は楽観的並行制御を行い、不一致時は ETagMismatchError をスローする。
  @override
  Future<void> delete(String key, {String? etag}) async {
    final existing = _entries[key];
    // エントリが存在しない場合は処理をスキップする。
    if (existing == null) return;
    // etag が指定されていて不一致の場合はエラーをスローする。
    if (etag != null && existing.etag != etag) {
      throw ETagMismatchError(key: key, expected: etag, actual: existing.etag);
    }
    _entries.remove(key);
  }

  /// 指定された複数のキーに対する状態エントリを一括取得する。
  /// 存在しないキーはスキップされ、結果リストに含まれない。
  @override
  Future<List<StateEntry>> bulkGet(List<String> keys) async {
    final results = <StateEntry>[];
    // 各キーを順番に get して、値が存在するものだけ結果に追加する。
    for (final key in keys) {
      final entry = await get(key);
      if (entry != null) results.add(entry);
    }
    return results;
  }

  /// 複数のエントリを一括保存し、各エントリの新しい ETag リストを返す。
  /// ETag の順序は入力エントリの順序に対応する。
  @override
  Future<List<String>> bulkSet(List<StateEntry> entries) async {
    final etags = <String>[];
    // 各エントリを順番に set して生成された ETag を蓄積する。
    for (final entry in entries) {
      etags.add(await set(entry.key, entry.value));
    }
    return etags;
  }
}
