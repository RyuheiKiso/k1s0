import type { ConfigFieldSchema } from '../../types';

interface EnumFieldProps {
  schema: ConfigFieldSchema;
  value: string;
  onChange: (v: string) => void;
}

export function EnumField({ schema, value, onChange }: EnumFieldProps) {
  return (
    <div className="config-field config-field--enum">
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        aria-label={schema.label}
      >
        {schema.options?.map((opt) => (
          <option key={opt} value={opt}>
            {opt}
          </option>
        ))}
      </select>
    </div>
  );
}
