import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

class _MockComponent extends Component {
  @override
  String get name => 'test-component';

  @override
  String get componentType => 'mock';

  ComponentStatus _status = ComponentStatus.uninitialized;

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
  Map<String, String> metadata() => {'version': '1.0.0'};
}

void main() {
  group('ComponentStatus', () {
    test('期待される全ての値を持つこと', () {
      expect(ComponentStatus.values.length, 5);
      expect(ComponentStatus.values, contains(ComponentStatus.uninitialized));
      expect(ComponentStatus.values, contains(ComponentStatus.ready));
      expect(ComponentStatus.values, contains(ComponentStatus.degraded));
      expect(ComponentStatus.values, contains(ComponentStatus.closed));
      expect(ComponentStatus.values, contains(ComponentStatus.error));
    });
  });

  group('Component', () {
    late _MockComponent component;

    setUp(() {
      component = _MockComponent();
    });

    test('name と componentType を持つこと', () {
      expect(component.name, 'test-component');
      expect(component.componentType, 'mock');
    });

    test('初期状態が uninitialized であること', () async {
      expect(await component.status(), ComponentStatus.uninitialized);
    });

    test('init 後に ready に遷移すること', () async {
      await component.init();
      expect(await component.status(), ComponentStatus.ready);
    });

    test('close 後に closed に遷移すること', () async {
      await component.init();
      await component.close();
      expect(await component.status(), ComponentStatus.closed);
    });

    test('メタデータを返すこと', () {
      expect(component.metadata(), {'version': '1.0.0'});
    });
  });
}
