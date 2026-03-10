import type { ConfigFieldSchema } from '../../types';

interface FloatFieldProps {
  schema: ConfigFieldSchema;
  value: number;
  onChange: (v: number) => void;
}

export function FloatField({ schema, value, onChange }: FloatFieldProps) {
  const hasRange = schema.min !== undefined && schema.max !== undefined;

  const handleChange = (raw: string) => {
    const parsed = Number.parseFloat(raw);
    if (!Number.isNaN(parsed)) {
      onChange(parsed);
    }
  };

  const error = getValidationError(schema, value);

  return (
    <div className="config-field config-field--float">
      <input
        type="number"
        step="any"
        value={value}
        onChange={(e) => handleChange(e.target.value)}
        min={schema.min}
        max={schema.max}
        aria-label={schema.label}
      />
      {hasRange && (
        <input
          type="range"
          step="any"
          value={value}
          onChange={(e) => handleChange(e.target.value)}
          min={schema.min}
          max={schema.max}
          aria-label={`${schema.label} slider`}
        />
      )}
      {schema.unit && <span className="config-field__unit">{schema.unit}</span>}
      {error && <span className="config-field__error" role="alert">{error}</span>}
    </div>
  );
}

function getValidationError(schema: ConfigFieldSchema, value: number): string | undefined {
  if (schema.min !== undefined && value < schema.min) {
    return `${schema.min} 以上の値を入力してください`;
  }
  if (schema.max !== undefined && value > schema.max) {
    return `${schema.max} 以下の値を入力してください`;
  }
  return undefined;
}
