export type GuardType = 'auth_required' | 'role_required' | 'redirect_if_authenticated';
export type TransitionType = 'fade' | 'slide' | 'modal';
export type ParamType = 'string' | 'int' | 'uuid';

export interface NavigationParam {
  name: string;
  type: ParamType;
}

export interface NavigationGuard {
  id: string;
  type: GuardType;
  redirect_to: string;
  roles?: string[];
}

export interface NavigationRoute {
  id: string;
  path: string;
  component_id?: string;
  guards?: string[];
  transition?: TransitionType;
  redirect_to?: string;
  children?: NavigationRoute[];
  params?: NavigationParam[];
}

export interface NavigationResponse {
  routes: NavigationRoute[];
  guards: NavigationGuard[];
}

export type ComponentRegistry = Record<
  string,
  React.ComponentType<any> | (() => Promise<{ default: React.ComponentType<any> }>)
>;
