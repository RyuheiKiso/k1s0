// ComponentRegistry はビルディングブロックコンポーネントの登録・管理を行うクラス。
// Map を用いてコンポーネントを名前で管理し、一括初期化・クローズ・ステータス取得をサポートする。

import 'component.dart';
import 'errors.dart';

class ComponentRegistry {
  /// 登録されたコンポーネントを名前をキーとして管理するマップ。
  final Map<String, Component> _components = {};

  /// コンポーネントをレジストリに登録する。同名が既に存在する場合は例外をスローする。
  void register(Component component) {
    if (_components.containsKey(component.name)) {
      throw ComponentError(
        component: component.name,
        operation: 'register',
        message: "コンポーネント '${component.name}' は既に登録されています",
      );
    }
    _components[component.name] = component;
  }

  /// 名前でコンポーネントを取得する。存在しない場合は null を返す。
  Component? get(String name) => _components[name];

  /// 登録済みの全コンポーネントを順次初期化する。
  /// いずれかの初期化に失敗した場合、その時点で例外をスローする。
  Future<void> initAll() async {
    for (final entry in _components.entries) {
      try {
        await entry.value.init();
      } catch (e) {
        throw ComponentError(
          component: entry.key,
          operation: 'initAll',
          message: "コンポーネント '${entry.key}' の初期化に失敗しました",
          cause: e,
        );
      }
    }
  }

  /// 登録済みの全コンポーネントを順次クローズする。
  /// いずれかのクローズに失敗した場合、その時点で例外をスローする。
  Future<void> closeAll() async {
    for (final entry in _components.entries) {
      try {
        await entry.value.close();
      } catch (e) {
        throw ComponentError(
          component: entry.key,
          operation: 'closeAll',
          message: "コンポーネント '${entry.key}' のクローズに失敗しました",
          cause: e,
        );
      }
    }
  }

  /// 登録済みの全コンポーネントのステータスをマップとして返す。
  Future<Map<String, ComponentStatus>> statusAll() async {
    final statuses = <String, ComponentStatus>{};
    for (final entry in _components.entries) {
      statuses[entry.key] = await entry.value.status();
    }
    return statuses;
  }
}
