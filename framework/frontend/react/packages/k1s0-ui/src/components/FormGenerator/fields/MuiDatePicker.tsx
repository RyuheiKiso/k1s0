/**
 * MUI DatePicker ラッパー
 */

import React from 'react';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import { DateTimePicker } from '@mui/x-date-pickers/DateTimePicker';
import { TimePicker } from '@mui/x-date-pickers/TimePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import dayjs, { Dayjs } from 'dayjs';
import 'dayjs/locale/ja';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps } from '../types.js';

export function MuiDatePicker<T extends FieldValues>(
  props: FormFieldComponentProps<T>
): React.ReactElement {
  const {
    name,
    label,
    helperText,
    parsedInfo,
    form,
    disabled = false,
    readOnly = false,
  } = props;

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs} adapterLocale="ja">
      <Controller
        name={name}
        control={form.control}
        render={({ field, fieldState: { error } }) => (
          <DatePicker
            label={label}
            value={field.value ? dayjs(field.value) : null}
            onChange={(date: Dayjs | null) => {
              field.onChange(date?.toDate() ?? null);
            }}
            disabled={disabled}
            readOnly={readOnly}
            slotProps={{
              textField: {
                fullWidth: true,
                error: !!error,
                helperText: error?.message ?? helperText,
                required: parsedInfo.required,
              },
            }}
          />
        )}
      />
    </LocalizationProvider>
  );
}

export function MuiDateTimePicker<T extends FieldValues>(
  props: FormFieldComponentProps<T>
): React.ReactElement {
  const {
    name,
    label,
    helperText,
    parsedInfo,
    form,
    disabled = false,
    readOnly = false,
  } = props;

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs} adapterLocale="ja">
      <Controller
        name={name}
        control={form.control}
        render={({ field, fieldState: { error } }) => (
          <DateTimePicker
            label={label}
            value={field.value ? dayjs(field.value) : null}
            onChange={(date: Dayjs | null) => {
              field.onChange(date?.toDate() ?? null);
            }}
            disabled={disabled}
            readOnly={readOnly}
            slotProps={{
              textField: {
                fullWidth: true,
                error: !!error,
                helperText: error?.message ?? helperText,
                required: parsedInfo.required,
              },
            }}
          />
        )}
      />
    </LocalizationProvider>
  );
}

export function MuiTimePicker<T extends FieldValues>(
  props: FormFieldComponentProps<T>
): React.ReactElement {
  const {
    name,
    label,
    helperText,
    parsedInfo,
    form,
    disabled = false,
    readOnly = false,
  } = props;

  return (
    <LocalizationProvider dateAdapter={AdapterDayjs} adapterLocale="ja">
      <Controller
        name={name}
        control={form.control}
        render={({ field, fieldState: { error } }) => (
          <TimePicker
            label={label}
            value={field.value ? dayjs(field.value) : null}
            onChange={(date: Dayjs | null) => {
              field.onChange(date?.toDate() ?? null);
            }}
            disabled={disabled}
            readOnly={readOnly}
            slotProps={{
              textField: {
                fullWidth: true,
                error: !!error,
                helperText: error?.message ?? helperText,
                required: parsedInfo.required,
              },
            }}
          />
        )}
      />
    </LocalizationProvider>
  );
}
