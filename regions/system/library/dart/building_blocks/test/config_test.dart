import 'package:test/test.dart';
import 'package:k1s0_building_blocks/building_blocks.dart';

void main() {
  group('ComponentConfig', () {
    test('全フィールドを指定して生成できること', () {
      const config = ComponentConfig(
        name: 'redis-store',
        type: 'statestore',
        version: '1.0',
        metadata: {'host': 'localhost', 'port': '6379'},
      );

      expect(config.name, 'redis-store');
      expect(config.type, 'statestore');
      expect(config.version, '1.0');
      expect(config.metadata, {'host': 'localhost', 'port': '6379'});
    });

    test('オプションフィールドなしで生成できること', () {
      const config = ComponentConfig(
        name: 'basic',
        type: 'binding',
      );

      expect(config.name, 'basic');
      expect(config.type, 'binding');
      expect(config.version, isNull);
      expect(config.metadata, isEmpty);
    });

    test('YAML マップから生成できること', () {
      final yaml = <dynamic, dynamic>{
        'name': 'kafka',
        'type': 'pubsub',
        'version': '2.0',
        'metadata': <dynamic, dynamic>{'broker': 'localhost:9092'},
      };

      final config = ComponentConfig.fromYaml(yaml);
      expect(config.name, 'kafka');
      expect(config.type, 'pubsub');
      expect(config.version, '2.0');
      expect(config.metadata, {'broker': 'localhost:9092'});
    });

    test('オプションフィールドなしの YAML マップから生成できること', () {
      final yaml = <dynamic, dynamic>{
        'name': 'simple',
        'type': 'secretstore',
      };

      final config = ComponentConfig.fromYaml(yaml);
      expect(config.name, 'simple');
      expect(config.type, 'secretstore');
      expect(config.version, isNull);
      expect(config.metadata, isEmpty);
    });
  });

  group('ComponentsConfig', () {
    test('複数コンポーネントを含む正常な YAML をパースできること', () {
      const yaml = '''
components:
  - name: redis-store
    type: statestore
    version: "1.0"
    metadata:
      host: localhost
      port: "6379"
  - name: kafka-pubsub
    type: pubsub
''';

      final config = ComponentsConfig.fromYaml(yaml);
      expect(config.components.length, 2);

      expect(config.components[0].name, 'redis-store');
      expect(config.components[0].type, 'statestore');
      expect(config.components[0].version, '1.0');
      expect(config.components[0].metadata, {'host': 'localhost', 'port': '6379'});

      expect(config.components[1].name, 'kafka-pubsub');
      expect(config.components[1].type, 'pubsub');
    });

    test('空のコンポーネント配列をパースできること', () {
      const yaml = '''
components: []
''';

      final config = ComponentsConfig.fromYaml(yaml);
      expect(config.components, isEmpty);
    });

    test('components フィールドが存在しない場合に ComponentError をスローすること', () {
      const yaml = '''
name: invalid
''';

      expect(
        () => ComponentsConfig.fromYaml(yaml),
        throwsA(isA<ComponentError>()),
      );
    });

    test('components がリストでない場合に ComponentError をスローすること', () {
      const yaml = '''
components: "not-a-list"
''';

      expect(
        () => ComponentsConfig.fromYaml(yaml),
        throwsA(isA<ComponentError>()),
      );
    });

    test('空の YAML の場合に ComponentError をスローすること', () {
      expect(
        () => ComponentsConfig.fromYaml(''),
        throwsA(isA<ComponentError>()),
      );
    });
  });
}
