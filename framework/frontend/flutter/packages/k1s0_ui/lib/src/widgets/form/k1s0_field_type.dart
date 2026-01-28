/// K1s0 Form フィールドタイプ
library;

/// フォームフィールドの種類
enum K1s0FieldType {
  /// テキスト入力
  text,

  /// メールアドレス入力
  email,

  /// パスワード入力
  password,

  /// 数値入力
  number,

  /// テキストエリア
  textarea,

  /// ドロップダウン選択
  select,

  /// ラジオボタン
  radio,

  /// チェックボックス
  checkbox,

  /// スイッチ
  switchField,

  /// 日付選択
  date,

  /// 日時選択
  dateTime,

  /// 時刻選択
  time,

  /// スライダー
  slider,

  /// 評価（星）
  rating,
}
