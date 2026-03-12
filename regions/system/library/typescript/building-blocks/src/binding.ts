import type { Component } from './component.js';

export interface BindingData {
  data: Uint8Array;
  metadata: Record<string, string>;
}

export interface BindingResponse {
  data: Uint8Array;
  metadata: Record<string, string>;
}

export interface InputBinding extends Component {
  read(): Promise<BindingData>;
}

export interface OutputBinding extends Component {
  invoke(operation: string, data: Uint8Array, metadata?: Record<string, string>): Promise<BindingResponse>;
}
