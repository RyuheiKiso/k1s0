import 'package:test/test.dart';
import 'package:building_blocks/building_blocks.dart';

void main() {
  group('ComponentConfig', () => {
    test('should create with all fields', () {
      final config = ComponentConfig(
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

    test('should create without optional fields', () {
      final config = ComponentConfig(
        name: 'basic',
        type: 'binding',
      );

      expect(config.name, 'basic');
      expect(config.type, 'binding');
      expect(config.version, isNull);
      expect(config.metadata, isEmpty);
    });

    test('should create from YAML map', () {
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

    test('should create from YAML map without optional fields', () {
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

  group('ComponentsConfig', () => {
    test('should parse valid YAML with multiple components', () {
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

    test('should parse empty components array', () {
      const yaml = '''
components: []
''';

      final config = ComponentsConfig.fromYaml(yaml);
      expect(config.components, isEmpty);
    });

    test('should throw ComponentError when components field is missing', () {
      const yaml = '''
name: invalid
''';

      expect(
        () => ComponentsConfig.fromYaml(yaml),
        throwsA(isA<ComponentError>()),
      );
    });

    test('should throw ComponentError when components is not a list', () {
      const yaml = '''
components: "not-a-list"
''';

      expect(
        () => ComponentsConfig.fromYaml(yaml),
        throwsA(isA<ComponentError>()),
      );
    });

    test('should throw ComponentError for empty YAML', () {
      expect(
        () => ComponentsConfig.fromYaml(''),
        throwsA(isA<ComponentError>()),
      );
    });
  });
}
