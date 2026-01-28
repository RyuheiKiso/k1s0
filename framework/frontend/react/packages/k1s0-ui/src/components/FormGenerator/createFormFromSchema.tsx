/**
 * createFormFromSchema - Zod スキーマから MUI フォームを生成
 */

import React, { useEffect } from 'react';
import { useForm, UseFormReturn, DefaultValues, Path } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import type {
  FormGeneratorOptions,
  GeneratedFormProps,
  ParsedFieldInfo,
  MuiFieldComponent,
} from './types.js';
import { parseSchema, extractDefaultValues } from './utils/schemaParser.js';
import { resolveFieldComponent } from './utils/fieldTypeResolver.js';
import { FormContainer } from './components/FormContainer.js';
import { FormGrid } from './components/FormGrid.js';
import { FormActions } from './components/FormActions.js';
import { MuiTextField } from './fields/MuiTextField.js';
import { MuiSelect } from './fields/MuiSelect.js';
import { MuiRadioGroup } from './fields/MuiRadioGroup.js';
import { MuiCheckbox } from './fields/MuiCheckbox.js';
import { MuiSwitch } from './fields/MuiSwitch.js';
import { MuiDatePicker, MuiDateTimePicker, MuiTimePicker } from './fields/MuiDatePicker.js';
import { MuiSlider } from './fields/MuiSlider.js';
import { MuiRating } from './fields/MuiRating.js';
import { MuiAutocomplete } from './fields/MuiAutocomplete.js';
import { useConditionalField } from './hooks/useConditionalField.js';

/**
 * フィールドコンポーネントをレンダリング
 */
function renderFieldComponent<T extends z.infer<z.ZodObject<z.ZodRawShape>>>(
  component: MuiFieldComponent,
  props: {
    name: Path<T>;
    label?: string;
    placeholder?: string;
    helperText?: string;
    parsedInfo: ParsedFieldInfo;
    form: UseFormReturn<T>;
    variant?: 'outlined' | 'filled' | 'standard';
    size?: 'small' | 'medium';
    fullWidth?: boolean;
    disabled?: boolean;
    readOnly?: boolean;
    config?: FormGeneratorOptions<z.ZodObject<z.ZodRawShape>>['fieldConfig'][string];
  }
): React.ReactNode {
  const { config, ...fieldProps } = props;

  // カスタムレンダーがあればそれを使用
  if (config?.render) {
    return config.render({
      name: fieldProps.name,
      label: fieldProps.label,
      helperText: fieldProps.helperText,
      error: fieldProps.form.formState.errors[fieldProps.name]?.message as string | undefined,
      disabled: fieldProps.disabled,
      readOnly: fieldProps.readOnly,
      form: fieldProps.form,
    });
  }

  const commonProps = { ...fieldProps, config };

  switch (component) {
    case 'TextField':
      return <MuiTextField {...commonProps} />;
    case 'Select':
      return <MuiSelect {...commonProps} />;
    case 'RadioGroup':
      return <MuiRadioGroup {...commonProps} />;
    case 'Checkbox':
      return <MuiCheckbox {...commonProps} />;
    case 'Switch':
      return <MuiSwitch {...commonProps} />;
    case 'DatePicker':
      return <MuiDatePicker {...commonProps} />;
    case 'DateTimePicker':
      return <MuiDateTimePicker {...commonProps} />;
    case 'TimePicker':
      return <MuiTimePicker {...commonProps} />;
    case 'Slider':
      return <MuiSlider {...commonProps} />;
    case 'Rating':
      return <MuiRating {...commonProps} />;
    case 'Autocomplete':
      return <MuiAutocomplete {...commonProps} />;
    default:
      return <MuiTextField {...commonProps} />;
  }
}

/**
 * Zod スキーマから React フォームコンポーネントを生成する
 *
 * @param schema - Zod オブジェクトスキーマ
 * @param options - フォーム生成オプション
 * @returns React フォームコンポーネント
 *
 * @example
 * ```tsx
 * const userSchema = z.object({
 *   name: z.string().min(1, '名前は必須です'),
 *   email: z.string().email('有効なメールアドレスを入力してください'),
 *   role: z.enum(['admin', 'user', 'guest']),
 * });
 *
 * const UserForm = createFormFromSchema(userSchema, {
 *   labels: { name: '氏名', email: 'メールアドレス', role: '権限' },
 *   fieldConfig: {
 *     role: {
 *       component: 'Select',
 *       options: [
 *         { label: '管理者', value: 'admin' },
 *         { label: '一般', value: 'user' },
 *         { label: 'ゲスト', value: 'guest' },
 *       ],
 *     },
 *   },
 *   submitLabel: '保存',
 * });
 *
 * // 使用
 * <UserForm
 *   defaultValues={{ role: 'user' }}
 *   onSubmit={handleSubmit}
 *   onCancel={handleCancel}
 * />
 * ```
 */
export function createFormFromSchema<T extends z.ZodObject<z.ZodRawShape>>(
  schema: T,
  options: FormGeneratorOptions<T> = {}
): React.FC<GeneratedFormProps<z.infer<T>>> {
  const {
    labels = {},
    placeholders = {},
    helperTexts = {},
    fieldConfig = {},
    columns = 1,
    spacing = 2,
    fieldOrder,
    conditionalFields = [],
    submitLabel = '送信',
    cancelLabel = 'キャンセル',
    showCancel = false,
    showReset = false,
    variant = 'outlined',
    size = 'medium',
    fullWidth = true,
    booleanComponent = 'Switch',
  } = options;

  // スキーマを解析
  const parsedFields = parseSchema(schema);
  const schemaDefaults = extractDefaultValues(schema);

  // フィールドの順序を決定
  const orderedFieldNames = fieldOrder ?? (Object.keys(parsedFields) as (keyof z.infer<T>)[]);

  // 生成されるフォームコンポーネント
  function GeneratedForm(props: GeneratedFormProps<z.infer<T>>): React.ReactElement {
    const {
      defaultValues,
      values,
      onSubmit,
      onCancel,
      onChange,
      disabled = false,
      loading = false,
      readOnly = false,
      form: externalForm,
      className,
      sx,
    } = props;

    // react-hook-form を初期化（外部から渡されなければ内部で管理）
    const internalForm = useForm<z.infer<T>>({
      resolver: zodResolver(schema),
      defaultValues: {
        ...schemaDefaults,
        ...defaultValues,
      } as DefaultValues<z.infer<T>>,
      mode: 'onBlur',
    });

    const form = externalForm ?? internalForm;

    // 制御モードの値を同期
    useEffect(() => {
      if (values) {
        form.reset(values as DefaultValues<z.infer<T>>);
      }
    }, [values, form]);

    // 値変更コールバック
    useEffect(() => {
      if (onChange) {
        const subscription = form.watch((value) => {
          onChange(value as Partial<z.infer<T>>);
        });
        return () => subscription.unsubscribe();
      }
    }, [form, onChange]);

    // 条件付きフィールドの表示制御
    const { isFieldVisible } = useConditionalField({
      form,
      conditionalFields: conditionalFields as Parameters<typeof useConditionalField>[0]['conditionalFields'],
    });

    // 送信ハンドラ
    const handleSubmit = form.handleSubmit(async (data) => {
      await onSubmit(data);
    });

    // リセットハンドラ
    const handleReset = () => {
      form.reset({
        ...schemaDefaults,
        ...defaultValues,
      } as DefaultValues<z.infer<T>>);
    };

    return (
      <FormContainer onSubmit={handleSubmit} className={className} sx={sx}>
        <FormGrid columns={columns} spacing={spacing}>
          {orderedFieldNames.map((fieldName) => {
            const name = fieldName as string;
            const parsedInfo = parsedFields[name];

            if (!parsedInfo) return null;

            // 条件付きフィールドの表示判定
            if (!isFieldVisible(name)) {
              return null;
            }

            const config = fieldConfig[fieldName as keyof typeof fieldConfig];

            // Boolean 型のデフォルトコンポーネントを設定
            let component = resolveFieldComponent(parsedInfo, config);
            if (parsedInfo.zodType === 'ZodBoolean' && !config?.component) {
              component = booleanComponent;
            }

            return (
              <React.Fragment key={name}>
                {renderFieldComponent(component, {
                  name: name as Path<z.infer<T>>,
                  label: labels[fieldName as keyof typeof labels] as string | undefined ?? name,
                  placeholder: placeholders[fieldName as keyof typeof placeholders] as string | undefined,
                  helperText: helperTexts[fieldName as keyof typeof helperTexts] as string | undefined,
                  parsedInfo,
                  form,
                  variant,
                  size,
                  fullWidth,
                  disabled: disabled || loading,
                  readOnly,
                  config,
                })}
              </React.Fragment>
            );
          })}
        </FormGrid>

        <FormActions
          submitLabel={submitLabel}
          cancelLabel={cancelLabel}
          showCancel={showCancel}
          showReset={showReset}
          onCancel={onCancel}
          onReset={handleReset}
          loading={loading}
          disabled={disabled}
        />
      </FormContainer>
    );
  }

  // displayName を設定
  GeneratedForm.displayName = 'GeneratedForm';

  return GeneratedForm;
}
