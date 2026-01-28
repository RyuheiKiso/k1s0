import 'package:flutter_test/flutter_test.dart';
import 'package:k1s0_realtime/src/sse/sse_parser.dart';

void main() {
  group('SSEParser', () {
    late SSEParser parser;

    setUp(() {
      parser = SSEParser();
    });

    test('基本的なイベントをパースする', () {
      expect(parser.parseLine('data: hello'), isNull);

      final event = parser.parseLine('');
      expect(event, isNotNull);
      expect(event!.eventType, equals('message'));
      expect(event.data, equals('hello'));
    });

    test('イベントタイプ付きイベントをパースする', () {
      parser.parseLine('event: custom');
      parser.parseLine('data: payload');

      final event = parser.parseLine('');
      expect(event, isNotNull);
      expect(event!.eventType, equals('custom'));
      expect(event.data, equals('payload'));
    });

    test('複数行データをパースする', () {
      parser.parseLine('data: line1');
      parser.parseLine('data: line2');
      parser.parseLine('data: line3');

      final event = parser.parseLine('');
      expect(event, isNotNull);
      expect(event!.data, equals('line1\nline2\nline3'));
    });

    test('イベント ID をパースする', () {
      parser.parseLine('id: 42');
      parser.parseLine('data: test');

      final event = parser.parseLine('');
      expect(event, isNotNull);
      expect(event!.id, equals('42'));
    });

    test('retry をパースする', () {
      parser.parseLine('retry: 3000');
      parser.parseLine('data: test');

      final event = parser.parseLine('');
      expect(event, isNotNull);
      expect(event!.retry, equals(3000));
    });

    test('コメント行を無視する', () {
      expect(parser.parseLine(': this is a comment'), isNull);
      parser.parseLine('data: test');

      final event = parser.parseLine('');
      expect(event, isNotNull);
      expect(event!.data, equals('test'));
    });

    test('データなしの空行はイベントを生成しない', () {
      final event = parser.parseLine('');
      expect(event, isNull);
    });

    test('コロン後のスペースを除去する', () {
      parser.parseLine('data: hello world');
      final event = parser.parseLine('');
      expect(event!.data, equals('hello world'));
    });

    test('コロンなしのフィールドを処理する', () {
      parser.parseLine('data');
      final event = parser.parseLine('');
      expect(event!.data, equals(''));
    });
  });
}
