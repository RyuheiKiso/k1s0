import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

// テスト用のシンプルなコンポーネント実装。
class _TestComponent extends Component {
  @override
  final String name;

  @override
  String get componentType => 'test';

  ComponentStatus _status = ComponentStatus.uninitialized;

  _TestComponent(this.name);

  @override
  Future<void> init() async {
    _status = ComponentStatus.ready;
  }

  @override
  Future<void> close() async {
    _status = ComponentStatus.closed;
  }

  @override
  Future<ComponentStatus> status() async => _status;

  @override
  Map<String, String> metadata() => {};
}

void main() {
  group('ComponentRegistry', () {
    late ComponentRegistry registry;

    setUp(() {
      registry = ComponentRegistry();
    });

    // コンポーネントを登録して名前で取得できることを確認する。
    test('コンポーネントを登録して取得できること', () {
      final c = _TestComponent('comp-1');
      registry.register(c);

      expect(registry.get('comp-1'), same(c));
    });

    // 同名のコンポーネントを重複登録すると例外がスローされることを確認する。
    test('同名コンポーネントの重複登録で例外がスローされること', () {
      registry.register(_TestComponent('dup'));
      expect(
        () => registry.register(_TestComponent('dup')),
        throwsA(isA<ComponentError>()),
      );
    });

    // 存在しない名前でコンポーネントを取得すると null が返ることを確認する。
    test('未登録のコンポーネントは null を返すこと', () {
      expect(registry.get('missing'), isNull);
    });

    // initAll が全コンポーネントを初期化することを確認する。
    test('initAll で全コンポーネントが初期化されること', () async {
      final a = _TestComponent('a');
      final b = _TestComponent('b');
      registry.register(a);
      registry.register(b);

      await registry.initAll();

      expect(await a.status(), ComponentStatus.ready);
      expect(await b.status(), ComponentStatus.ready);
    });

    // closeAll が全コンポーネントをクローズすることを確認する。
    test('closeAll で全コンポーネントがクローズされること', () async {
      final a = _TestComponent('a');
      registry.register(a);
      await a.init();

      await registry.closeAll();

      expect(await a.status(), ComponentStatus.closed);
    });

    // statusAll が全コンポーネントのステータスをマップで返すことを確認する。
    test('statusAll が全コンポーネントのステータスを返すこと', () async {
      registry.register(_TestComponent('a'));
      registry.register(_TestComponent('b'));
      await registry.initAll();

      final statuses = await registry.statusAll();
      expect(statuses['a'], ComponentStatus.ready);
      expect(statuses['b'], ComponentStatus.ready);
    });
  });
}
