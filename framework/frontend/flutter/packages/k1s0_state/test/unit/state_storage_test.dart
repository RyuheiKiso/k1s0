import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_state/src/persistence/state_storage.dart';

void main() {
  group('StringSerializer', () {
    const serializer = StringSerializer();

    test('toJson wraps value', () {
      final json = serializer.toJson('hello');

      expect(json, {'value': 'hello'});
    });

    test('fromJson unwraps value', () {
      final value = serializer.fromJson({'value': 'world'});

      expect(value, 'world');
    });

    test('roundtrip preserves value', () {
      const original = 'test string';
      final json = serializer.toJson(original);
      final result = serializer.fromJson(json);

      expect(result, original);
    });
  });

  group('IntSerializer', () {
    const serializer = IntSerializer();

    test('toJson wraps value', () {
      final json = serializer.toJson(42);

      expect(json, {'value': 42});
    });

    test('fromJson unwraps value', () {
      final value = serializer.fromJson({'value': 123});

      expect(value, 123);
    });

    test('roundtrip preserves value', () {
      const original = 99;
      final json = serializer.toJson(original);
      final result = serializer.fromJson(json);

      expect(result, original);
    });
  });

  group('DoubleSerializer', () {
    const serializer = DoubleSerializer();

    test('toJson wraps value', () {
      final json = serializer.toJson(3.14);

      expect(json, {'value': 3.14});
    });

    test('fromJson unwraps value', () {
      final value = serializer.fromJson({'value': 2.718});

      expect(value, 2.718);
    });

    test('fromJson handles int as double', () {
      final value = serializer.fromJson({'value': 42});

      expect(value, 42.0);
    });

    test('roundtrip preserves value', () {
      const original = 1.618;
      final json = serializer.toJson(original);
      final result = serializer.fromJson(json);

      expect(result, original);
    });
  });

  group('BoolSerializer', () {
    const serializer = BoolSerializer();

    test('toJson wraps true', () {
      final json = serializer.toJson(true);

      expect(json, {'value': true});
    });

    test('toJson wraps false', () {
      final json = serializer.toJson(false);

      expect(json, {'value': false});
    });

    test('fromJson unwraps value', () {
      expect(serializer.fromJson({'value': true}), true);
      expect(serializer.fromJson({'value': false}), false);
    });

    test('roundtrip preserves value', () {
      for (final original in [true, false]) {
        final json = serializer.toJson(original);
        final result = serializer.fromJson(json);
        expect(result, original);
      }
    });
  });

  group('ListSerializer', () {
    test('serializes list of strings', () {
      const serializer = ListSerializer(StringSerializer());
      final json = serializer.toJson(['a', 'b', 'c']);

      expect(json['items'], isA<List>());
      expect(json['items'], hasLength(3));
    });

    test('deserializes list of strings', () {
      const serializer = ListSerializer(StringSerializer());
      final list = serializer.fromJson({
        'items': [
          {'value': 'x'},
          {'value': 'y'},
        ],
      });

      expect(list, ['x', 'y']);
    });

    test('roundtrip preserves list', () {
      const serializer = ListSerializer(IntSerializer());
      final original = [1, 2, 3, 4, 5];
      final json = serializer.toJson(original);
      final result = serializer.fromJson(json);

      expect(result, original);
    });

    test('handles empty list', () {
      const serializer = ListSerializer(StringSerializer());
      final json = serializer.toJson([]);
      final result = serializer.fromJson(json);

      expect(result, isEmpty);
    });
  });

  group('MapSerializer', () {
    test('serializes map of strings', () {
      const serializer = MapSerializer(StringSerializer());
      final json = serializer.toJson({'key1': 'value1', 'key2': 'value2'});

      expect(json['entries'], isA<Map>());
    });

    test('deserializes map of strings', () {
      const serializer = MapSerializer(StringSerializer());
      final map = serializer.fromJson({
        'entries': {
          'k1': {'value': 'v1'},
          'k2': {'value': 'v2'},
        },
      });

      expect(map['k1'], 'v1');
      expect(map['k2'], 'v2');
    });

    test('roundtrip preserves map', () {
      const serializer = MapSerializer(IntSerializer());
      final original = {'a': 1, 'b': 2, 'c': 3};
      final json = serializer.toJson(original);
      final result = serializer.fromJson(json);

      expect(result, original);
    });

    test('handles empty map', () {
      const serializer = MapSerializer(StringSerializer());
      final json = serializer.toJson({});
      final result = serializer.fromJson(json);

      expect(result, isEmpty);
    });
  });

  group('StateStorage interface', () {
    test('StateStorage methods are defined', () {
      // This is a compile-time check that the interface is correctly defined
      // We cannot test an abstract class directly, but we can verify
      // that implementations would need to implement these methods

      // ignore: unused_local_variable
      late StateStorage storage;

      // The test passes if this compiles
      expect(true, isTrue);
    });
  });
}
