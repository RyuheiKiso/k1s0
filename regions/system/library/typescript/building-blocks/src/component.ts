export type ComponentStatus = 'uninitialized' | 'ready' | 'degraded' | 'closed' | 'error';

export interface Component {
  readonly name: string;
  readonly componentType: string;
  init(): Promise<void>;
  close(): Promise<void>;
  status(): Promise<ComponentStatus>;
  metadata(): Record<string, string>;
}
