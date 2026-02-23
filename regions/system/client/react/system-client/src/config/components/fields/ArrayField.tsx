import type { ConfigFieldSchema } from '../../types';

interface ArrayFieldProps {
  schema: ConfigFieldSchema;
  value: string[];
  onChange: (v: string[]) => void;
}

export function ArrayField({ schema, value, onChange }: ArrayFieldProps) {
  const handleChange = (raw: string) => {
    const items = raw.split(',').map((s) => s.trim()).filter(Boolean);
    onChange(items);
  };

  return (
    <div className="config-field config-field--array">
      <input
        type="text"
        value={value.join(', ')}
        onChange={(e) => handleChange(e.target.value)}
        aria-label={schema.label}
        placeholder="カンマ区切りで入力"
      />
    </div>
  );
}
