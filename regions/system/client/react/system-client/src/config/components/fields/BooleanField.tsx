import type { ConfigFieldSchema } from '../../types';

interface BooleanFieldProps {
  schema: ConfigFieldSchema;
  value: boolean;
  onChange: (v: boolean) => void;
}

export function BooleanField({ schema, value, onChange }: BooleanFieldProps) {
  return (
    <div className="config-field config-field--boolean">
      <label>
        <input
          type="checkbox"
          checked={value}
          onChange={(e) => onChange(e.target.checked)}
          aria-label={schema.label}
        />
        {schema.label}
      </label>
    </div>
  );
}
