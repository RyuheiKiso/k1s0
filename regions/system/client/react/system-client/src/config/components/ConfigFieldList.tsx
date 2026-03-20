import type { ConfigCategorySchema, ConfigFieldValue } from '../types';
import { IntegerField } from './fields/IntegerField';
import { FloatField } from './fields/FloatField';
import { BooleanField } from './fields/BooleanField';
import { EnumField } from './fields/EnumField';
import { StringField } from './fields/StringField';
import { ObjectField } from './fields/ObjectField';
import { ArrayField } from './fields/ArrayField';

// 数値型の型ガード: unknown な値が number であることを安全に検証する
function isNumber(value: unknown): value is number {
  return typeof value === 'number';
}

// 真偽値型の型ガード: unknown な値が boolean であることを安全に検証する
function isBoolean(value: unknown): value is boolean {
  return typeof value === 'boolean';
}

// 文字列型の型ガード: unknown な値が string であることを安全に検証する
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

// オブジェクト型の型ガード: unknown な値が非nullオブジェクトであることを安全に検証する
function isObject(value: unknown): value is object {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

// 文字列配列型の型ガード: unknown な値が string[] であることを安全に検証する
function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string');
}

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
    // 整数フィールド: 値が数値型であることを型ガードで検証し、不正な場合はデフォルト値0を使用
    case 'integer':
      return (
        <IntegerField
          schema={field}
          value={isNumber(value) ? value : 0}
          onChange={onChange}
        />
      );
    // 浮動小数点フィールド: 値が数値型であることを型ガードで検証し、不正な場合はデフォルト値0を使用
    case 'float':
      return (
        <FloatField
          schema={field}
          value={isNumber(value) ? value : 0}
          onChange={onChange}
        />
      );
    // 真偽値フィールド: 値がboolean型であることを型ガードで検証し、不正な場合はデフォルト値falseを使用
    case 'boolean':
      return (
        <BooleanField
          schema={field}
          value={isBoolean(value) ? value : false}
          onChange={onChange}
        />
      );
    // 列挙型フィールド: 値が文字列型であることを型ガードで検証し、不正な場合はデフォルト値空文字を使用
    case 'enum':
      return (
        <EnumField
          schema={field}
          value={isString(value) ? value : ''}
          onChange={onChange}
        />
      );
    // オブジェクトフィールド: 値がオブジェクト型であることを型ガードで検証し、不正な場合はデフォルト値空オブジェクトを使用
    case 'object':
      return (
        <ObjectField
          schema={field}
          value={isObject(value) ? value : {}}
          onChange={onChange}
        />
      );
    // 配列フィールド: 値が文字列配列型であることを型ガードで検証し、不正な場合はデフォルト値空配列を使用
    case 'array':
      return (
        <ArrayField
          schema={field}
          value={isStringArray(value) ? value : []}
          onChange={onChange}
        />
      );
    // 文字列フィールド(デフォルト): 値が文字列型であることを型ガードで検証し、不正な場合はデフォルト値空文字を使用
    case 'string':
    default:
      return (
        <StringField
          schema={field}
          value={isString(value) ? value : ''}
          onChange={onChange}
        />
      );
  }
}
