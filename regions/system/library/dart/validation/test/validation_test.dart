import 'package:test/test.dart';

import 'package:k1s0_validation/validation.dart';

void main() {
  group('validateEmail', () {
    test('有効なメールアドレスを受け入れること', () {
      expect(() => validateEmail('user@example.com'), returnsNormally);
    });

    test('ドットを含むメールアドレスを受け入れること', () {
      expect(() => validateEmail('first.last@example.co.jp'), returnsNormally);
    });

    test('@なしのメールアドレスを拒否すること', () {
      expect(() => validateEmail('invalid'), throwsA(isA<ValidationError>()));
    });

    test('ドメインなしのメールアドレスを拒否すること', () {
      expect(() => validateEmail('user@'), throwsA(isA<ValidationError>()));
    });
  });

  group('validateUuid', () {
    test('有効なUUID v4を受け入れること', () {
      expect(() => validateUuid('550e8400-e29b-41d4-a716-446655440000'), returnsNormally);
    });

    test('無効なUUIDを拒否すること', () {
      expect(() => validateUuid('not-a-uuid'), throwsA(isA<ValidationError>()));
    });

    test('UUID v1形式を拒否すること', () {
      expect(() => validateUuid('550e8400-e29b-11d4-a716-446655440000'), throwsA(isA<ValidationError>()));
    });
  });

  group('validateUrl', () {
    test('https URLを受け入れること', () {
      expect(() => validateUrl('https://example.com'), returnsNormally);
    });

    test('http URLを受け入れること', () {
      expect(() => validateUrl('http://example.com/path'), returnsNormally);
    });

    test('スキームなしのURLを拒否すること', () {
      expect(() => validateUrl('example.com'), throwsA(isA<ValidationError>()));
    });

    test('ftp URLを拒否すること', () {
      expect(() => validateUrl('ftp://example.com'), throwsA(isA<ValidationError>()));
    });
  });

  group('validateTenantId', () {
    test('有効なテナントIDを受け入れること', () {
      expect(() => validateTenantId('my-tenant-1'), returnsNormally);
    });

    test('短すぎるテナントIDを拒否すること', () {
      expect(() => validateTenantId('ab'), throwsA(isA<ValidationError>()));
    });

    test('大文字を含むテナントIDを拒否すること', () {
      expect(() => validateTenantId('MyTenant'), throwsA(isA<ValidationError>()));
    });

    test('特殊文字を含むテナントIDを拒否すること', () {
      expect(() => validateTenantId('my_tenant!'), throwsA(isA<ValidationError>()));
    });
  });

  group('ValidationError', () {
    test('フィールドとtoStringが正しいこと', () {
      const err = ValidationError('email', 'bad');
      expect(err.field, equals('email'));
      expect(err.message, equals('bad'));
      expect(err.toString(), contains('email'));
    });

    test('codeフィールドが設定されること', () {
      const err = ValidationError('email', 'bad', code: 'INVALID_EMAIL');
      expect(err.code, equals('INVALID_EMAIL'));
    });

    test('デフォルトのcodeがフィールド名から導出されること', () {
      const err = ValidationError('email', 'bad');
      expect(err.code, equals('INVALID_EMAIL'));
    });
  });

  group('validatePagination', () {
    test('有効なページネーションを受け入れること', () {
      expect(() => validatePagination(1, 10), returnsNormally);
      expect(() => validatePagination(1, 1), returnsNormally);
      expect(() => validatePagination(1, 100), returnsNormally);
      expect(() => validatePagination(999, 50), returnsNormally);
    });

    test('ページ番号が1未満の場合を拒否すること', () {
      expect(() => validatePagination(0, 10), throwsA(isA<ValidationError>()));
      expect(() => validatePagination(-1, 10), throwsA(isA<ValidationError>()));
    });

    test('1ページあたりの件数が範囲外の場合を拒否すること', () {
      expect(() => validatePagination(1, 0), throwsA(isA<ValidationError>()));
      expect(() => validatePagination(1, 101), throwsA(isA<ValidationError>()));
    });

    test('エラーに正しいコードが設定されること', () {
      try {
        validatePagination(0, 10);
      } on ValidationError catch (e) {
        expect(e.code, equals('INVALID_PAGE'));
      }
      try {
        validatePagination(1, 0);
      } on ValidationError catch (e) {
        expect(e.code, equals('INVALID_PER_PAGE'));
      }
    });
  });

  group('validateDateRange', () {
    test('有効な日付範囲を受け入れること', () {
      final start = DateTime(2024, 1, 1);
      final end = DateTime(2024, 12, 31);
      expect(() => validateDateRange(start, end), returnsNormally);
    });

    test('開始日と終了日が同じ場合を受け入れること', () {
      final dt = DateTime(2024, 6, 15);
      expect(() => validateDateRange(dt, dt), returnsNormally);
    });

    test('開始日が終了日より後の場合を拒否すること', () {
      final start = DateTime(2024, 12, 31);
      final end = DateTime(2024, 1, 1);
      expect(() => validateDateRange(start, end), throwsA(isA<ValidationError>()));
    });

    test('エラーに正しいコードが設定されること', () {
      try {
        validateDateRange(DateTime(2024, 12, 31), DateTime(2024, 1, 1));
      } on ValidationError catch (e) {
        expect(e.code, equals('INVALID_DATE_RANGE'));
      }
    });
  });

  group('ValidationErrors', () {
    test('空のコレクションにエラーがないこと', () {
      final errors = ValidationErrors();
      expect(errors.hasErrors(), isFalse);
      expect(errors.getErrors(), isEmpty);
    });

    test('エラーの追加が正しく動作すること', () {
      final errors = ValidationErrors();
      errors.add(const ValidationError('email', 'bad', code: 'INVALID_EMAIL'));
      errors.add(const ValidationError('page', 'bad', code: 'INVALID_PAGE'));

      expect(errors.hasErrors(), isTrue);
      expect(errors.getErrors(), hasLength(2));
      expect(errors.getErrors()[0].code, equals('INVALID_EMAIL'));
      expect(errors.getErrors()[1].code, equals('INVALID_PAGE'));
    });
  });

  // =========================================================================
  // validation_test_cases.json の全ケースを網羅するテスト（H-18 4言語統一）
  // =========================================================================

  group('JSON統一仕様: validateEmail', () {
    // JSON仕様の全有効メールアドレスを受け入れること
    test('有効ケース全件を受け入れること', () {
      expect(() => validateEmail('user@example.com'), returnsNormally);
      expect(() => validateEmail('user.name+tag@example.co.jp'), returnsNormally);
      expect(() => validateEmail('test123@sub.domain.org'), returnsNormally);
    });

    // JSON仕様の全無効メールアドレスを拒否すること
    test('無効ケース全件を拒否すること', () {
      // 空文字列
      expect(() => validateEmail(''), throwsA(isA<ValidationError>()));
      // @ なし
      expect(() => validateEmail('not-an-email'), throwsA(isA<ValidationError>()));
      // ドメインなし
      expect(() => validateEmail('user@'), throwsA(isA<ValidationError>()));
      // ローカルパートなし
      expect(() => validateEmail('@example.com'), throwsA(isA<ValidationError>()));
      // TLD が 1 文字
      expect(() => validateEmail('user@example.c'), throwsA(isA<ValidationError>()));
      // スペースを含む
      expect(() => validateEmail('user @example.com'), throwsA(isA<ValidationError>()));
    });
  });

  group('JSON統一仕様: validateUuid', () {
    // JSON仕様の全有効 UUID v4 を受け入れること
    test('有効ケース全件を受け入れること', () {
      expect(() => validateUuid('550e8400-e29b-41d4-a716-446655440000'), returnsNormally);
      expect(() => validateUuid('6ba7b810-9dad-41d5-80b4-00c04fd430c8'), returnsNormally);
      expect(() => validateUuid('f47ac10b-58cc-4372-a567-0e02b2c3d479'), returnsNormally);
    });

    // JSON仕様の全無効 UUID を拒否すること
    test('無効ケース全件を拒否すること', () {
      // 空文字列
      expect(() => validateUuid(''), throwsA(isA<ValidationError>()));
      // UUID 形式でない文字列
      expect(() => validateUuid('not-a-uuid'), throwsA(isA<ValidationError>()));
      // v1（バージョンビット 1）
      expect(() => validateUuid('550e8400-e29b-11d4-a716-446655440000'), throwsA(isA<ValidationError>()));
      // v2（バージョンビット 2）
      expect(() => validateUuid('550e8400-e29b-21d4-a716-446655440000'), throwsA(isA<ValidationError>()));
      // v3（バージョンビット 3）
      expect(() => validateUuid('550e8400-e29b-31d4-a716-446655440000'), throwsA(isA<ValidationError>()));
      // v5（バージョンビット 5）
      expect(() => validateUuid('550e8400-e29b-51d4-a716-446655440000'), throwsA(isA<ValidationError>()));
      // バリアントビット不正（c716 は許可されないバリアント）
      expect(() => validateUuid('550e8400-e29b-41d4-c716-446655440000'), throwsA(isA<ValidationError>()));
      // 不正な文字を含む
      expect(() => validateUuid('ZZZZZZZZ-ZZZZ-4ZZZ-8ZZZ-ZZZZZZZZZZZZ'), throwsA(isA<ValidationError>()));
    });
  });

  group('JSON統一仕様: validateTenantId', () {
    // JSON仕様の全有効テナントIDを受け入れること
    test('有効ケース全件を受け入れること', () {
      expect(() => validateTenantId('abc'), returnsNormally);
      expect(() => validateTenantId('my-tenant'), returnsNormally);
      expect(() => validateTenantId('tenant-123'), returnsNormally);
      expect(() => validateTenantId('a1b2c3d4'), returnsNormally);
    });

    // JSON仕様の全無効テナントIDを拒否すること
    test('無効ケース全件を拒否すること', () {
      // 空文字列
      expect(() => validateTenantId(''), throwsA(isA<ValidationError>()));
      // 1 文字
      expect(() => validateTenantId('a'), throwsA(isA<ValidationError>()));
      // 2 文字（最小長不足）
      expect(() => validateTenantId('ab'), throwsA(isA<ValidationError>()));
      // 先頭がハイフン
      expect(() => validateTenantId('-abc'), throwsA(isA<ValidationError>()));
      // 末尾がハイフン
      expect(() => validateTenantId('abc-'), throwsA(isA<ValidationError>()));
      // 大文字のみ
      expect(() => validateTenantId('ABC'), throwsA(isA<ValidationError>()));
      // アンダースコアを含む
      expect(() => validateTenantId('my_tenant'), throwsA(isA<ValidationError>()));
      // スペースを含む
      expect(() => validateTenantId('my tenant'), throwsA(isA<ValidationError>()));
    });
  });

  group('JSON統一仕様: validatePagination', () {
    // JSON仕様の全有効ページネーションを受け入れること
    test('有効ケース全件を受け入れること', () {
      expect(() => validatePagination(1, 1), returnsNormally);
      expect(() => validatePagination(1, 50), returnsNormally);
      expect(() => validatePagination(1, 100), returnsNormally);
      expect(() => validatePagination(10, 25), returnsNormally);
    });

    // JSON仕様の全無効ページネーションを拒否すること
    test('無効ケース全件を拒否すること', () {
      // page が 0
      expect(() => validatePagination(0, 10), throwsA(isA<ValidationError>()));
      // page が -1
      expect(() => validatePagination(-1, 10), throwsA(isA<ValidationError>()));
      // per_page が 0
      expect(() => validatePagination(1, 0), throwsA(isA<ValidationError>()));
      // per_page が 101（上限超過）
      expect(() => validatePagination(1, 101), throwsA(isA<ValidationError>()));
      // per_page が 200（大きく上限超過）
      expect(() => validatePagination(1, 200), throwsA(isA<ValidationError>()));
    });
  });
}
