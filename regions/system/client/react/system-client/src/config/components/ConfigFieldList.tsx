import type { ConfigCategorySchema, ConfigFieldValue } from '../types';
import { IntegerField } from './fields/IntegerField';
import { BooleanField } from './fields/BooleanField';
import { EnumField } from './fields/EnumField';
import { StringField } from './fields/StringField';
import { ObjectField } from './fields/ObjectField';
import { ArrayField } from './fields/ArrayField';

interface ConfigFieldListProps {
  category: ConfigCategorySchema & { fieldValues: Record<string, ConfigFieldValue> };
  onUpdate: (key: string, value: unknown) => void;
  onResetToDefault: (key: string) => void;
}

export function ConfigFieldList({ category, onUpdate, onResetToDefault }: ConfigFieldListProps) {
  return (
    <div className="config-field-list">
      {category.fields.map((field) => {
        const fieldValue = category.fieldValues[field.key];
        const value = fieldValue?.value;

        return (
          <div key={field.key} className="config-field-list__item">
            <div className="config-field-list__header">
              <label className="config-field-list__label">{field.label}</label>
              {field.description && (
                <span className="config-field-list__description">{field.description}</span>
              )}
              {fieldValue?.isDirty && (
                <button
                  type="button"
                  className="config-field-list__reset"
                  onClick={() => onResetToDefault(field.key)}
                >
                  デフォルトに戻す
                </button>
              )}
            </div>
            {renderField(field, value, (v: unknown) => onUpdate(field.key, v))}
            {fieldValue?.hasError && (
              <span className="config-field-list__error" role="alert">
                {fieldValue.hasError}
              </span>
            )}
          </div>
        );
      })}
    </div>
  );
}

function renderField(
  field: ConfigFieldListProps['category']['fields'][number],
  value: unknown,
  onChange: (v: unknown) => void,
) {
  switch (field.type) {
    case 'integer':
    case 'float':
      return (
        <IntegerField
          schema={field}
          value={(value as number) ?? 0}
          onChange={onChange}
        />
      );
    case 'boolean':
      return (
        <BooleanField
          schema={field}
          value={(value as boolean) ?? false}
          onChange={onChange}
        />
      );
    case 'enum':
      return (
        <EnumField
          schema={field}
          value={(value as string) ?? ''}
          onChange={onChange}
        />
      );
    case 'object':
      return (
        <ObjectField
          schema={field}
          value={(value as object) ?? {}}
          onChange={onChange}
        />
      );
    case 'array':
      return (
        <ArrayField
          schema={field}
          value={(value as string[]) ?? []}
          onChange={onChange}
        />
      );
    case 'string':
    default:
      return (
        <StringField
          schema={field}
          value={(value as string) ?? ''}
          onChange={onChange}
        />
      );
  }
}
