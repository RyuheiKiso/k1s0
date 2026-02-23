import { useState } from 'react';
import type { ConfigFieldSchema } from '../../types';

interface ObjectFieldProps {
  schema: ConfigFieldSchema;
  value: object;
  onChange: (v: object) => void;
}

export function ObjectField({ schema, value, onChange }: ObjectFieldProps) {
  const [error, setError] = useState<string>();

  const handleChange = (raw: string) => {
    try {
      const parsed = JSON.parse(raw) as object;
      setError(undefined);
      onChange(parsed);
    } catch {
      setError('無効なJSONです');
    }
  };

  return (
    <div className="config-field config-field--object">
      <textarea
        value={JSON.stringify(value, null, 2)}
        onChange={(e) => handleChange(e.target.value)}
        aria-label={schema.label}
        rows={6}
      />
      {error && <span className="config-field__error" role="alert">{error}</span>}
    </div>
  );
}
