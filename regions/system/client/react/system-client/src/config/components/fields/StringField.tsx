import { useMemo } from 'react';
import type { ConfigFieldSchema } from '../../types';

interface StringFieldProps {
  schema: ConfigFieldSchema;
  value: string;
  onChange: (v: string) => void;
}

export function StringField({ schema, value, onChange }: StringFieldProps) {
  const regex = useMemo(
    () => (schema.pattern ? new RegExp(schema.pattern) : null),
    [schema.pattern],
  );

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
