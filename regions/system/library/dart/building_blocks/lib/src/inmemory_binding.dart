import 'dart:typed_data';

import 'binding.dart';
import 'component.dart';
import 'errors.dart';

/// OutputBinding への呼び出し記録を保持するデータクラス。
/// テスト検証時に invoke の引数を事後確認するために使用する。
class BindingInvocation {
  /// 呼び出されたオペレーション名。
  final String operation;

  /// 送信されたバイナリデータ。
  final Uint8List data;

  /// 呼び出し時に付与されたメタデータ。
  final Map<String, String> metadata;

  const BindingInvocation({
    required this.operation,
    required this.data,
    required this.metadata,
  });
}

/// インメモリ実装の InputBinding。
/// 外部システムの代わりにメモリ上のキューからデータを読み取る。
/// テスト・開発環境での利用を想定している。
class InMemoryInputBinding implements InputBinding {
  /// コンポーネントの識別名。
  @override
  final String name;

  /// コンポーネント種別を示す文字列。InputBinding であることを表す。
  @override
  final String componentType = 'binding.input';

  /// コンポーネントの現在の状態（未初期化 / 準備完了 / クローズ済み）。
  ComponentStatus _status = ComponentStatus.uninitialized;

  /// テスト用データを蓄積するインメモリキュー。
  final _queue = <BindingData>[];

  /// コンストラクタ。name のデフォルト値は 'inmemory-input-binding'。
  InMemoryInputBinding({this.name = 'inmemory-input-binding'});

  /// コンポーネントを初期化し、状態を ready に移行する。
  @override
  Future<void> init() async { _status = ComponentStatus.ready; }

  /// コンポーネントをクローズし、キューを空にして状態を closed に移行する。
  @override
  Future<void> close() async { _queue.clear(); _status = ComponentStatus.closed; }

  /// コンポーネントの現在の状態を返す。
  @override
  Future<ComponentStatus> status() async => _status;

  /// バックエンドが memory であること、データの方向が input であることを示すメタデータを返す。
  @override
  Map<String, String> metadata() => {'backend': 'memory', 'direction': 'input'};

  /// テスト用: データをキューに追加する。
  /// このメソッドでキューに積んだデータが read() で取り出される。
  void push(BindingData data) { _queue.add(data); }

  /// キューの先頭からデータを取り出して返す。
  /// キューが空の場合は ComponentError をスローする。
  @override
  Future<BindingData> read() async {
    if (_queue.isEmpty) {
      throw ComponentError(component: name, operation: 'read', message: 'queue is empty');
    }
    return _queue.removeAt(0);
  }
}

/// インメモリ実装の OutputBinding。
/// 外部システムへの送信の代わりに、呼び出し履歴をメモリ上に記録する。
/// テスト・開発環境での利用を想定しており、モックレスポンスやエラーを設定できる。
class InMemoryOutputBinding implements OutputBinding {
  /// コンポーネントの識別名。
  @override
  final String name;

  /// コンポーネント種別を示す文字列。OutputBinding であることを表す。
  @override
  final String componentType = 'binding.output';

  /// コンポーネントの現在の状態（未初期化 / 準備完了 / クローズ済み）。
  ComponentStatus _status = ComponentStatus.uninitialized;

  /// invoke が呼ばれるたびに追加される呼び出し履歴。
  final _invocations = <BindingInvocation>[];

  /// invoke 時に返すモックレスポンス。null の場合はデフォルト値を返す。
  BindingResponse? _mockResponse;

  /// invoke 時にスローするモックエラー。null の場合はエラーを発生させない。
  Object? _mockError;

  /// コンストラクタ。name のデフォルト値は 'inmemory-output-binding'。
  InMemoryOutputBinding({this.name = 'inmemory-output-binding'});

  /// コンポーネントを初期化し、状態を ready に移行する。
  @override
  Future<void> init() async { _status = ComponentStatus.ready; }

  /// コンポーネントをクローズし、呼び出し履歴を削除して状態を closed に移行する。
  @override
  Future<void> close() async { _invocations.clear(); _status = ComponentStatus.closed; }

  /// コンポーネントの現在の状態を返す。
  @override
  Future<ComponentStatus> status() async => _status;

  /// バックエンドが memory であること、データの方向が output であることを示すメタデータを返す。
  @override
  Map<String, String> metadata() => {'backend': 'memory', 'direction': 'output'};

  /// 最後に記録された呼び出しを返す。履歴が空の場合は null を返す。
  BindingInvocation? lastInvocation() =>
      _invocations.isEmpty ? null : _invocations.last;

  /// 記録された全呼び出し履歴を変更不可リストとして返す。
  List<BindingInvocation> allInvocations() => List.unmodifiable(_invocations);

  /// テスト用: invoke が返すモックレスポンスまたはモックエラーを設定する。
  /// error が設定されている場合、invoke はそのエラーをスローする。
  void setResponse({BindingResponse? response, Object? error}) {
    _mockResponse = response;
    _mockError = error;
  }

  /// テスト用: 呼び出し履歴とモック設定をすべてリセットする。
  void reset() {
    _invocations.clear();
    _mockResponse = null;
    _mockError = null;
  }

  /// 指定したオペレーションを実行し、呼び出し記録を内部リストに追加する。
  /// モックエラーが設定されている場合はそのエラーをスローする。
  /// モックレスポンスが設定されている場合はそれを返し、
  /// 未設定の場合は受け取ったデータとメタデータをそのまま返す。
  @override
  Future<BindingResponse> invoke(
    String operation,
    Uint8List data, {
    Map<String, String>? metadata,
  }) async {
    final meta = metadata ?? {};
    // 呼び出し履歴を記録する。
    _invocations.add(BindingInvocation(operation: operation, data: data, metadata: meta));
    // モックエラーが設定されている場合はスローする。
    if (_mockError != null) throw _mockError!;
    // モックレスポンスを返す。未設定の場合はデフォルトのレスポンスを返す。
    return _mockResponse ?? BindingResponse(data: data, metadata: meta);
  }
}
