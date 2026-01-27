/// Form validation utilities
class K1s0Validators {
  K1s0Validators._();

  /// Required field validator
  static String? required(String? value, [String? fieldName]) {
    if (value == null || value.trim().isEmpty) {
      return fieldName != null ? '$fieldNameは必須です' : '必須項目です';
    }
    return null;
  }

  /// Email validator
  static String? email(String? value) {
    if (value == null || value.isEmpty) {
      return null; // Use required() for required validation
    }

    final emailRegex = RegExp(
      r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,253}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,253}[a-zA-Z0-9])?)*$",
    );

    if (!emailRegex.hasMatch(value)) {
      return '有効なメールアドレスを入力してください';
    }

    return null;
  }

  /// Phone number validator (Japanese format)
  static String? phone(String? value) {
    if (value == null || value.isEmpty) {
      return null;
    }

    // Remove common formatting characters
    final cleaned = value.replaceAll(RegExp(r'[\s\-()]'), '');

    // Japanese phone number patterns
    final phoneRegex = RegExp(r'^(0[0-9]{9,10}|\+81[0-9]{9,10})$');

    if (!phoneRegex.hasMatch(cleaned)) {
      return '有効な電話番号を入力してください';
    }

    return null;
  }

  /// Minimum length validator
  static String? Function(String?) minLength(int length) {
    return (String? value) {
      if (value == null || value.isEmpty) {
        return null;
      }

      if (value.length < length) {
        return '$length文字以上で入力してください';
      }

      return null;
    };
  }

  /// Maximum length validator
  static String? Function(String?) maxLength(int length) {
    return (String? value) {
      if (value == null || value.isEmpty) {
        return null;
      }

      if (value.length > length) {
        return '$length文字以下で入力してください';
      }

      return null;
    };
  }

  /// Numeric validator
  static String? numeric(String? value) {
    if (value == null || value.isEmpty) {
      return null;
    }

    if (double.tryParse(value) == null) {
      return '数値を入力してください';
    }

    return null;
  }

  /// Integer validator
  static String? integer(String? value) {
    if (value == null || value.isEmpty) {
      return null;
    }

    if (int.tryParse(value) == null) {
      return '整数を入力してください';
    }

    return null;
  }

  /// URL validator
  static String? url(String? value) {
    if (value == null || value.isEmpty) {
      return null;
    }

    final uri = Uri.tryParse(value);
    if (uri == null || !uri.hasScheme || (!uri.isScheme('http') && !uri.isScheme('https'))) {
      return '有効なURLを入力してください';
    }

    return null;
  }

  /// Pattern validator
  static String? Function(String?) pattern(RegExp regex, String message) {
    return (String? value) {
      if (value == null || value.isEmpty) {
        return null;
      }

      if (!regex.hasMatch(value)) {
        return message;
      }

      return null;
    };
  }

  /// Password strength validator
  static String? passwordStrength(String? value) {
    if (value == null || value.isEmpty) {
      return null;
    }

    if (value.length < 8) {
      return 'パスワードは8文字以上にしてください';
    }

    if (!value.contains(RegExp(r'[A-Z]'))) {
      return 'パスワードには大文字を含めてください';
    }

    if (!value.contains(RegExp(r'[a-z]'))) {
      return 'パスワードには小文字を含めてください';
    }

    if (!value.contains(RegExp(r'[0-9]'))) {
      return 'パスワードには数字を含めてください';
    }

    return null;
  }

  /// Match validator (e.g., for password confirmation)
  static String? Function(String?) match(String? other, String fieldName) {
    return (String? value) {
      if (value != other) {
        return '$fieldNameが一致しません';
      }

      return null;
    };
  }

  /// Combine multiple validators
  static String? Function(String?) combine(
    List<String? Function(String?)> validators,
  ) {
    return (String? value) {
      for (final validator in validators) {
        final error = validator(value);
        if (error != null) {
          return error;
        }
      }
      return null;
    };
  }
}
