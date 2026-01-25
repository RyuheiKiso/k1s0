/**
 * ナビゲーション設定スキーマ
 *
 * zod を使用したバリデーション定義
 */

import { z } from 'zod';

/** 権限・フラグ条件スキーマ */
export const RequiresConditionSchema = z.object({
  permissions: z.array(z.string()).optional(),
  flags: z.array(z.string()).optional(),
});

/** ルート定義スキーマ */
export const RouteConfigSchema = z
  .object({
    path: z.string().min(1, 'path は必須です'),
    redirect_to: z.string().optional(),
    screen_id: z.string().optional(),
    title: z.string().optional(),
    requires: RequiresConditionSchema.optional(),
  })
  .refine(
    (data) => data.redirect_to !== undefined || data.screen_id !== undefined,
    {
      message: 'redirect_to または screen_id のいずれかが必要です',
    }
  );

/** メニュー項目スキーマ */
export const MenuItemConfigSchema = z.object({
  label: z.string().min(1, 'label は必須です'),
  to: z.string().min(1, 'to は必須です'),
  icon: z.string().optional(),
  requires: RequiresConditionSchema.optional(),
});

/** メニューグループスキーマ */
export const MenuGroupConfigSchema = z.object({
  id: z.string().min(1, 'id は必須です'),
  label: z.string().min(1, 'label は必須です'),
  items: z.array(MenuItemConfigSchema).min(1, '最低1つのメニュー項目が必要です'),
});

/** フロー遷移条件スキーマ */
export const FlowTransitionWhenSchema = z.object({
  required_form_keys: z.array(z.string()).optional(),
  flags: z.array(z.string()).optional(),
});

/** フロー遷移スキーマ */
export const FlowTransitionConfigSchema = z.object({
  from: z.string().min(1, 'from は必須です'),
  event: z.string().min(1, 'event は必須です'),
  to: z.string().min(1, 'to は必須です'),
  when: FlowTransitionWhenSchema.optional(),
});

/** フローノードスキーマ */
export const FlowNodeConfigSchema = z.object({
  node_id: z.string().min(1, 'node_id は必須です'),
  screen_id: z.string().min(1, 'screen_id は必須です'),
});

/** フロー定義スキーマ */
export const FlowConfigSchema = z.object({
  id: z.string().min(1, 'id は必須です'),
  title: z.string().min(1, 'title は必須です'),
  start: z.object({
    screen_id: z.string().min(1, 'start.screen_id は必須です'),
  }),
  requires: RequiresConditionSchema.optional(),
  nodes: z.array(FlowNodeConfigSchema).min(1, '最低1つのノードが必要です'),
  transitions: z.array(FlowTransitionConfigSchema),
  on_error: z
    .object({
      redirect_to: z.string().min(1, 'redirect_to は必須です'),
    })
    .optional(),
});

/** ナビゲーション設定全体スキーマ */
export const NavigationConfigSchema = z.object({
  version: z.number().int().positive('version は正の整数である必要があります'),
  routes: z.array(RouteConfigSchema).min(1, '最低1つのルートが必要です'),
  menu: z.array(MenuGroupConfigSchema),
  flows: z.array(FlowConfigSchema).optional(),
});

/** バリデーション結果 */
export interface ValidationResult {
  success: boolean;
  errors: string[];
}

/**
 * ナビゲーション設定のバリデーション
 */
export function validateNavigationConfig(config: unknown): ValidationResult {
  const result = NavigationConfigSchema.safeParse(config);

  if (result.success) {
    return { success: true, errors: [] };
  }

  const errors = result.error.errors.map((err) => {
    const path = err.path.join('.');
    return path ? `${path}: ${err.message}` : err.message;
  });

  return { success: false, errors };
}

/**
 * 設定の整合性チェック
 *
 * - routes で参照される screen_id が登録されているか
 * - flows で参照される screen_id が登録されているか
 * - flows の遷移が有効なノードを参照しているか
 */
export function validateConfigIntegrity(
  config: z.infer<typeof NavigationConfigSchema>,
  registeredScreenIds: Set<string>
): ValidationResult {
  const errors: string[] = [];

  // routes の screen_id チェック
  for (const route of config.routes) {
    if (route.screen_id && !registeredScreenIds.has(route.screen_id)) {
      errors.push(
        `routes: screen_id "${route.screen_id}" は登録されていません (path: ${route.path})`
      );
    }
  }

  // flows のチェック
  if (config.flows) {
    for (const flow of config.flows) {
      const nodeIds = new Set(flow.nodes.map((n) => n.node_id));

      // start の screen_id チェック
      if (!registeredScreenIds.has(flow.start.screen_id)) {
        errors.push(
          `flows.${flow.id}: start.screen_id "${flow.start.screen_id}" は登録されていません`
        );
      }

      // nodes の screen_id チェック
      for (const node of flow.nodes) {
        if (!registeredScreenIds.has(node.screen_id)) {
          errors.push(
            `flows.${flow.id}.nodes: screen_id "${node.screen_id}" は登録されていません (node_id: ${node.node_id})`
          );
        }
      }

      // transitions の from/to チェック
      for (const transition of flow.transitions) {
        if (!nodeIds.has(transition.from)) {
          errors.push(
            `flows.${flow.id}.transitions: from "${transition.from}" は存在しないノードです`
          );
        }
        if (!nodeIds.has(transition.to)) {
          errors.push(
            `flows.${flow.id}.transitions: to "${transition.to}" は存在しないノードです`
          );
        }
      }
    }
  }

  return {
    success: errors.length === 0,
    errors,
  };
}
