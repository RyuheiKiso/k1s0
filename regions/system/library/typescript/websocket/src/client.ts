import type { WsMessage, ConnectionState } from './types.js';

export interface WsClient {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  send(message: WsMessage): Promise<void>;
  receive(): Promise<WsMessage>;
  readonly state: ConnectionState;
}

export class InMemoryWsClient implements WsClient {
  private _state: ConnectionState = 'disconnected';
  private sendBuffer: WsMessage[] = [];
  private receiveBuffer: WsMessage[] = [];
  private receiveResolvers: Array<(msg: WsMessage) => void> = [];

  get state(): ConnectionState {
    return this._state;
  }

  async connect(): Promise<void> {
    if (this._state === 'connected') {
      throw new Error('Already connected');
    }
    this._state = 'connected';
  }

  async disconnect(): Promise<void> {
    if (this._state === 'disconnected') {
      throw new Error('Already disconnected');
    }
    this._state = 'disconnected';
  }

  async send(message: WsMessage): Promise<void> {
    if (this._state !== 'connected') {
      throw new Error('Not connected');
    }
    this.sendBuffer.push(message);
  }

  async receive(): Promise<WsMessage> {
    if (this._state !== 'connected') {
      throw new Error('Not connected');
    }
    if (this.receiveBuffer.length > 0) {
      return this.receiveBuffer.shift()!;
    }
    return new Promise<WsMessage>((resolve) => {
      this.receiveResolvers.push(resolve);
    });
  }

  injectMessage(msg: WsMessage): void {
    if (this.receiveResolvers.length > 0) {
      const resolve = this.receiveResolvers.shift()!;
      resolve(msg);
    } else {
      this.receiveBuffer.push(msg);
    }
  }

  getSentMessages(): WsMessage[] {
    const msgs = [...this.sendBuffer];
    this.sendBuffer = [];
    return msgs;
  }
}
