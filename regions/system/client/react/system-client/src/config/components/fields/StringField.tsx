import { useMemo } from 'react';
import type { ConfigFieldSchema } from '../../types';

interface StringFieldProps {
  schema: ConfigFieldSchema;
  value: string;
  onChange: (v: string) => void;
}

// FE-05 対応: ReDoS（Regular Expression Denial of Service）対策
// 長大なパターン文字列による CPU 過負荷を防ぐため、パターン長の上限を設ける
const MAX_PATTERN_LENGTH = 200;

export function StringField({ schema, value, onChange }: StringFieldProps) {
  // スキーマのパターンを安全に正規表現オブジェクトへ変換する
  // パターン長が上限を超える場合または無効な構文の場合は null を返し、バリデーションをスキップする
  const regex = useMemo(() => {
    if (!schema.pattern) return null;
    if (schema.pattern.length > MAX_PATTERN_LENGTH) {
      console.warn('[StringField] pattern が長すぎるため無視されました');
      return null;
    }
    try {
      return new RegExp(schema.pattern);
    } catch {
      return null;
    }
  }, [schema.pattern]);

  const error = regex && value && !regex.test(value)
    ? `パターン ${schema.pattern} に一致しません`
    : undefined;

  return (
    <div className="config-field config-field--string">
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        aria-label={schema.label}
      />
      {error && <span className="config-field__error" role="alert">{error}</span>}
    </div>
  );
}
