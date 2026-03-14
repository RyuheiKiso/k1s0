import 'component.dart';
import 'errors.dart';
import 'secretstore.dart';

/// インメモリ実装の SecretStore（シークレットストア）コンポーネント。
/// シークレットをメモリ上の Map で管理する。
/// テスト・開発環境での利用を想定しており、外部の Vault 等を必要としない。
class InMemorySecretStore implements SecretStore {
  /// コンポーネントの識別名。
  @override
  final String name;

  /// コンポーネント種別を示す文字列。
  @override
  final String componentType = 'secretstore';

  /// コンポーネントの現在の状態（未初期化 / 準備完了 / クローズ済み）。
  ComponentStatus _status = ComponentStatus.uninitialized;

  /// キーと値のペアでシークレットを保持するインメモリストレージ。
  final _secrets = <String, String>{};

  /// コンストラクタ。name のデフォルト値は 'inmemory-secretstore'。
  InMemorySecretStore({this.name = 'inmemory-secretstore'});

  /// コンポーネントを初期化し、状態を ready に移行する。
  @override
  Future<void> init() async { _status = ComponentStatus.ready; }

  /// コンポーネントをクローズし、全シークレットを削除して状態を closed に移行する。
  @override
  Future<void> close() async { _secrets.clear(); _status = ComponentStatus.closed; }

  /// コンポーネントの現在の状態を返す。
  @override
  Future<ComponentStatus> status() async => _status;

  /// バックエンドが memory であることを示すメタデータを返す。
  @override
  Map<String, String> metadata() => {'backend': 'memory'};

  /// テスト用: シークレットを設定する。
  /// このメソッドで登録した値が getSecret / bulkGet で取得できる。
  void put(String key, String value) { _secrets[key] = value; }

  /// 指定キーに対応するシークレットを取得して返す。
  /// キーが存在しない場合は ComponentError をスローする。
  @override
  Future<SecretValue> getSecret(String key) async {
    final value = _secrets[key];
    // 指定キーのシークレットが存在しない場合はエラーをスローする。
    if (value == null) {
      throw ComponentError(
        component: name,
        operation: 'getSecret',
        message: 'secret "$key" not found',
      );
    }
    return SecretValue(key: key, value: value, metadata: {});
  }

  /// 指定された複数のキーに対するシークレットを一括取得する。
  /// いずれかのキーが存在しない場合は getSecret 内で ComponentError がスローされる。
  @override
  Future<Map<String, SecretValue>> bulkGet(List<String> keys) async {
    final result = <String, SecretValue>{};
    // 各キーについて順番に getSecret を呼び出して結果を蓄積する。
    for (final key in keys) {
      result[key] = await getSecret(key);
    }
    return result;
  }
}
