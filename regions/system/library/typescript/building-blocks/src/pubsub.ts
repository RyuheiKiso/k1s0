import type { Component } from './component.js';

export interface Message {
  topic: string;
  data: Uint8Array;
  metadata: Record<string, string>;
  id: string;
}

export interface MessageHandler {
  handle(message: Message): Promise<void>;
}

export interface PubSub extends Component {
  publish(topic: string, data: Uint8Array, metadata?: Record<string, string>): Promise<void>;
  subscribe(topic: string, handler: MessageHandler): Promise<string>;
  unsubscribe(subscriptionId: string): Promise<void>;
}
