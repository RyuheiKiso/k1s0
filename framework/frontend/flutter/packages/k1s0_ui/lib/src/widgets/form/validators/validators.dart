/// K1s0 Form バリデーター
library;

/// バリデーター基底クラス
abstract class K1s0Validator {
  /// バリデーション実行
  String? validate(dynamic value);
}

/// 必須バリデーター
class RequiredValidator extends K1s0Validator {
  final String? message;

  RequiredValidator({this.message});

  @override
  String? validate(dynamic value) {
    if (value == null || value.toString().isEmpty) {
      return message ?? '必須項目です';
    }
    return null;
  }
}

/// メールアドレスバリデーター
class EmailValidator extends K1s0Validator {
  final String? message;

  EmailValidator({this.message});

  static final _emailRegex = RegExp(
    r'^[a-zA-Z0-9.!#$%&*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$',
  );

  @override
  String? validate(dynamic value) {
    if (value == null || value.toString().isEmpty) return null;
    if (!_emailRegex.hasMatch(value.toString())) {
      return message ?? '有効なメールアドレスを入力してください';
    }
    return null;
  }
}

/// 最小文字数バリデーター
class MinLengthValidator extends K1s0Validator {
  final int minLength;
  final String? message;

  MinLengthValidator(this.minLength, {this.message});

  @override
  String? validate(dynamic value) {
    if (value == null || value.toString().isEmpty) return null;
    if (value.toString().length < minLength) {
      return message ?? '$minLength文字以上で入力してください';
    }
    return null;
  }
}

/// 最大文字数バリデーター
class MaxLengthValidator extends K1s0Validator {
  final int maxLength;
  final String? message;

  MaxLengthValidator(this.maxLength, {this.message});

  @override
  String? validate(dynamic value) {
    if (value == null || value.toString().isEmpty) return null;
    if (value.toString().length > maxLength) {
      return message ?? '$maxLength文字以内で入力してください';
    }
    return null;
  }
}

/// 正規表現バリデーター
class PatternValidator extends K1s0Validator {
  final RegExp pattern;
  final String? message;

  PatternValidator(this.pattern, {this.message});

  @override
  String? validate(dynamic value) {
    if (value == null || value.toString().isEmpty) return null;
    if (!pattern.hasMatch(value.toString())) {
      return message ?? '形式が正しくありません';
    }
    return null;
  }
}

/// 範囲バリデーター
class RangeValidator extends K1s0Validator {
  final num? min;
  final num? max;
  final String? message;

  RangeValidator({this.min, this.max, this.message});

  @override
  String? validate(dynamic value) {
    if (value == null) return null;

    final numValue = num.tryParse(value.toString());
    if (numValue == null) return null;

    if (min != null && numValue < min!) {
      return message ?? '$min以上の値を入力してください';
    }
    if (max != null && numValue > max!) {
      return message ?? '$max以下の値を入力してください';
    }
    return null;
  }
}

/// 複合バリデーター
class CompositeValidator extends K1s0Validator {
  final List<K1s0Validator> validators;

  CompositeValidator(this.validators);

  @override
  String? validate(dynamic value) {
    for (final validator in validators) {
      final error = validator.validate(value);
      if (error != null) return error;
    }
    return null;
  }
}

/// バリデーター関数を作成
String? Function(dynamic) createValidator(List<K1s0Validator> validators) {
  final composite = CompositeValidator(validators);
  return composite.validate;
}
