/// k1s0 UI Design System
///
/// Provides Material 3 based theming, common widgets, form validation,
/// feedback components, and loading/error state widgets.
library k1s0_ui;

export 'src/feedback/dialog.dart';
export 'src/feedback/feedback_provider.dart';
export 'src/feedback/snackbar.dart';
export 'src/form/form_container.dart';
export 'src/form/form_field_widget.dart';
export 'src/form/validation.dart';
export 'src/state/empty_state.dart';
export 'src/state/error_state.dart';
export 'src/state/loading.dart';
export 'src/theme/k1s0_colors.dart';
export 'src/theme/k1s0_spacing.dart';
export 'src/theme/k1s0_theme.dart';
export 'src/theme/k1s0_typography.dart';
export 'src/theme/theme_provider.dart';
export 'src/widgets/buttons.dart';
export 'src/widgets/cards.dart';
export 'src/widgets/text_fields.dart';

// DataTable
export 'src/widgets/data_table/k1s0_data_table.dart';
export 'src/widgets/data_table/k1s0_column.dart';
export 'src/widgets/data_table/k1s0_sort_model.dart';
export 'src/widgets/data_table/k1s0_selection.dart';
export 'src/widgets/data_table/controllers/data_table_controller.dart';
export 'src/widgets/data_table/components/data_table_header.dart';
export 'src/widgets/data_table/components/data_table_row.dart';
export 'src/widgets/data_table/components/data_table_cell.dart';
export 'src/widgets/data_table/components/data_table_pagination.dart';
export 'src/widgets/data_table/components/data_table_loading.dart';
export 'src/widgets/data_table/components/data_table_empty.dart';

// Form Generator
export 'src/widgets/form/k1s0_form.dart';
export 'src/widgets/form/k1s0_form_schema.dart';
export 'src/widgets/form/k1s0_field_type.dart';
export 'src/widgets/form/controllers/form_controller.dart';
export 'src/widgets/form/components/form_container.dart' hide K1s0FormContainer;
export 'src/widgets/form/components/form_field_wrapper.dart';
export 'src/widgets/form/components/form_grid.dart';
export 'src/widgets/form/components/form_actions.dart';
export 'src/widgets/form/fields/text_form_field.dart';
export 'src/widgets/form/fields/dropdown_form_field.dart';
export 'src/widgets/form/fields/radio_form_field.dart';
export 'src/widgets/form/fields/checkbox_form_field.dart';
export 'src/widgets/form/fields/switch_form_field.dart';
export 'src/widgets/form/fields/date_form_field.dart';
export 'src/widgets/form/fields/slider_form_field.dart';
export 'src/widgets/form/validators/validators.dart';
