/// K1s0 DataTable カラム定義
library;

import 'package:flutter/material.dart';

/// カラムの型種別
enum K1s0ColumnType {
  /// 文字列
  string,

  /// 数値
  number,

  /// 日付
  date,

  /// 日時
  dateTime,

  /// Boolean
  boolean,

  /// 単一選択
  singleSelect,
}

/// 選択肢オプション
class K1s0ValueOption {
  /// ラベル
  final String label;

  /// 値
  final dynamic value;

  const K1s0ValueOption({
    required this.label,
    required this.value,
  });
}

/// K1s0 カラム定義
class K1s0Column<T> {
  /// フィールド名
  final String field;

  /// ヘッダー名
  final String headerName;

  /// ソート可能
  final bool sortable;

  /// フィルタ可能
  final bool filterable;

  /// カラム型
  final K1s0ColumnType type;

  /// 固定幅
  final double? width;

  /// 可変幅の比率
  final double? flex;

  /// テキスト配置
  final TextAlign align;

  /// カスタムセルレンダラー
  final Widget Function(dynamic value, T row, int index)? renderCell;

  /// 選択肢（singleSelect 用）
  final List<K1s0ValueOption>? valueOptions;

  /// 日付フォーマット（date/dateTime 用）
  final String? dateFormat;

  /// 小数点以下桁数（number 用）
  final int? decimalPlaces;

  /// 3桁区切り（number 用）
  final bool thousandSeparator;

  /// プレフィックス（number 用）
  final String? prefix;

  /// サフィックス（number 用）
  final String? suffix;

  /// エクスポート対象
  final bool exportable;

  const K1s0Column({
    required this.field,
    required this.headerName,
    this.sortable = false,
    this.filterable = false,
    this.type = K1s0ColumnType.string,
    this.width,
    this.flex,
    this.align = TextAlign.left,
    this.renderCell,
    this.valueOptions,
    this.dateFormat,
    this.decimalPlaces,
    this.thousandSeparator = true,
    this.prefix,
    this.suffix,
    this.exportable = true,
  });

  /// 値を取得するヘルパー
  dynamic getValue(T row) {
    if (row is Map) {
      return row[field];
    }
    // 動的にフィールドを取得（リフレクションは使用不可のため、Map を推奨）
    return null;
  }

  /// 値をフォーマットする
  String formatValue(dynamic value) {
    if (value == null) return '';

    switch (type) {
      case K1s0ColumnType.date:
        if (value is DateTime) {
          return _formatDate(value, dateFormat ?? 'yyyy/MM/dd');
        }
        return value.toString();

      case K1s0ColumnType.dateTime:
        if (value is DateTime) {
          return _formatDate(value, dateFormat ?? 'yyyy/MM/dd HH:mm');
        }
        return value.toString();

      case K1s0ColumnType.number:
        if (value is num) {
          return _formatNumber(value);
        }
        return value.toString();

      case K1s0ColumnType.boolean:
        return value == true ? 'はい' : 'いいえ';

      case K1s0ColumnType.singleSelect:
        final option = valueOptions?.firstWhere(
          (opt) => opt.value == value,
          orElse: () => K1s0ValueOption(label: value.toString(), value: value),
        );
        return option?.label ?? value.toString();

      case K1s0ColumnType.string:
      default:
        return value.toString();
    }
  }

  String _formatDate(DateTime date, String format) {
    // 簡易実装（intl パッケージを使用する場合は DateFormat を使用）
    return format
        .replaceAll('yyyy', date.year.toString().padLeft(4, '0'))
        .replaceAll('MM', date.month.toString().padLeft(2, '0'))
        .replaceAll('dd', date.day.toString().padLeft(2, '0'))
        .replaceAll('HH', date.hour.toString().padLeft(2, '0'))
        .replaceAll('mm', date.minute.toString().padLeft(2, '0'))
        .replaceAll('ss', date.second.toString().padLeft(2, '0'));
  }

  String _formatNumber(num value) {
    String result;

    if (decimalPlaces != null) {
      result = value.toStringAsFixed(decimalPlaces!);
    } else {
      result = value.toString();
    }

    if (thousandSeparator) {
      final parts = result.split('.');
      parts[0] = parts[0].replaceAllMapped(
        RegExp(r'(\d{1,3})(?=(\d{3})+(?!\d))'),
        (match) => '${match[1]},',
      );
      result = parts.join('.');
    }

    return '${prefix ?? ''}$result${suffix ?? ''}';
  }
}
