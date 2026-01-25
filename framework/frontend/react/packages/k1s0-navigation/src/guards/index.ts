/**
 * Guards エクスポート
 */

export {
  PermissionGuard,
  useHasPermission,
  useHasAllPermissions,
  useHasAnyPermission,
} from './PermissionGuard';
export type { PermissionGuardProps } from './PermissionGuard';

export {
  FlagGuard,
  useHasFlag,
  useHasAllFlags,
  useHasAnyFlag,
} from './FlagGuard';
export type { FlagGuardProps } from './FlagGuard';
