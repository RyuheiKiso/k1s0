import 'package:flutter_test/flutter_test.dart';
import 'package:system_client/system_client.dart';

void main() {
  group('ConfigValue.fromJson', () {
    test('文字列は StringConfigValue に変換される', () {
      final value = ConfigValue.fromJson('hello');
      expect(value, isA<StringConfigValue>());
      expect((value as StringConfigValue).value, equals('hello'));
    });

    test('整数は NumberConfigValue に変換される', () {
      final value = ConfigValue.fromJson(42);
      expect(value, isA<NumberConfigValue>());
      expect((value as NumberConfigValue).value, equals(42));
    });

    test('浮動小数点は NumberConfigValue に変換される', () {
      final value = ConfigValue.fromJson(3.14);
      expect(value, isA<NumberConfigValue>());
      expect((value as NumberConfigValue).value, closeTo(3.14, 0.001));
    });

    test('真偽値は BoolConfigValue に変換される', () {
      final value = ConfigValue.fromJson(true);
      expect(value, isA<BoolConfigValue>());
      expect((value as BoolConfigValue).value, isTrue);
    });

    test('リストは ListConfigValue に変換される', () {
      final value = ConfigValue.fromJson([1, 'a', true]);
      expect(value, isA<ListConfigValue>());
      final list = value as ListConfigValue;
      expect(list.values.length, equals(3));
    });

    test('マップは MapConfigValue に変換される', () {
      final value = ConfigValue.fromJson({'key': 'value', 'num': 10});
      expect(value, isA<MapConfigValue>());
      final map = value as MapConfigValue;
      expect(map.entries.length, equals(2));
    });

    test('null は空の StringConfigValue に変換される', () {
      final value = ConfigValue.fromJson(null);
      expect(value, isA<StringConfigValue>());
      expect((value as StringConfigValue).value, equals(''));
    });
  });

  group('ConfigValue.toJson', () {
    test('StringConfigValue は文字列を返す', () {
      expect(const StringConfigValue('hello').toJson(), equals('hello'));
    });

    test('NumberConfigValue は数値を返す', () {
      expect(const NumberConfigValue(42).toJson(), equals(42));
    });

    test('BoolConfigValue は真偽値を返す', () {
      expect(const BoolConfigValue(true).toJson(), isTrue);
    });

    test('ListConfigValue はリストを返す', () {
      const list = ListConfigValue([
        const StringConfigValue('a'),
        const NumberConfigValue(1),
      ]);
      expect(list.toJson(), equals(['a', 1]));
    });

    test('MapConfigValue はマップを返す', () {
      const map = MapConfigValue({
        'key': const StringConfigValue('val'),
      });
      expect(map.toJson(), equals({'key': 'val'}));
    });
  });

  group('ConfigValue 等値性', () {
    test('同じ文字列の StringConfigValue は等しい', () {
      expect(
        const StringConfigValue('a'),
        equals(const StringConfigValue('a')),
      );
    });

    test('異なる文字列の StringConfigValue は等しくない', () {
      expect(
        const StringConfigValue('a'),
        isNot(equals(const StringConfigValue('b'))),
      );
    });

    test('同じ値の ListConfigValue は等しい', () {
      expect(
        const ListConfigValue([StringConfigValue('x')]),
        equals(const ListConfigValue([StringConfigValue('x')])),
      );
    });

    test('同じエントリの MapConfigValue は等しい', () {
      expect(
        const MapConfigValue({'k': NumberConfigValue(1)}),
        equals(const MapConfigValue({'k': NumberConfigValue(1)})),
      );
    });
  });

  group('ConfigFieldType.fromString', () {
    test('string を変換する', () {
      expect(ConfigFieldType.fromString('string'), equals(ConfigFieldType.string));
    });

    test('integer を変換する', () {
      expect(ConfigFieldType.fromString('integer'), equals(ConfigFieldType.integer));
    });

    test('float を変換する', () {
      expect(ConfigFieldType.fromString('float'), equals(ConfigFieldType.float));
    });

    test('boolean を変換する', () {
      expect(ConfigFieldType.fromString('boolean'), equals(ConfigFieldType.boolean));
    });

    test('enum を enumType に変換する', () {
      expect(ConfigFieldType.fromString('enum'), equals(ConfigFieldType.enumType));
    });

    test('object を変換する', () {
      expect(ConfigFieldType.fromString('object'), equals(ConfigFieldType.object));
    });

    test('array を変換する', () {
      expect(ConfigFieldType.fromString('array'), equals(ConfigFieldType.array));
    });

    test('不明な文字列は string にフォールバックする', () {
      expect(ConfigFieldType.fromString('unknown'), equals(ConfigFieldType.string));
    });
  });

  group('ConfigFieldSchema.fromJson', () {
    test('基本フィールドをパースする', () {
      final schema = ConfigFieldSchema.fromJson({
        'key': 'timeout',
        'label': 'タイムアウト',
        'type': 'integer',
      });
      expect(schema.key, equals('timeout'));
      expect(schema.label, equals('タイムアウト'));
      expect(schema.type, equals(ConfigFieldType.integer));
    });

    test('default 値が設定される', () {
      final schema = ConfigFieldSchema.fromJson({
        'key': 'name',
        'label': '名前',
        'type': 'string',
        'default': 'デフォルト',
      });
      expect(schema.defaultValue, equals(const StringConfigValue('デフォルト')));
    });

    test('min/max が設定される', () {
      final schema = ConfigFieldSchema.fromJson({
        'key': 'port',
        'label': 'ポート',
        'type': 'integer',
        'min': 1024,
        'max': 65535,
      });
      expect(schema.min, equals(1024));
      expect(schema.max, equals(65535));
    });

    test('options が設定される', () {
      final schema = ConfigFieldSchema.fromJson({
        'key': 'level',
        'label': 'レベル',
        'type': 'enum',
        'options': ['low', 'medium', 'high'],
      });
      expect(schema.options, equals(['low', 'medium', 'high']));
    });
  });
}
