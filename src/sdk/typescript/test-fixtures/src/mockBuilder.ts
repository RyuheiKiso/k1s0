// src/sdk/typescript/test-fixtures/src/mockBuilder.ts
//
// k1s0 TypeScript SDK test-fixtures: Mock builder fluent API（領域 3、ADR-TEST-010 §3）。
// リリース時点で 3 service（State / Audit / PubSub）を提供。
// 採用初期で +3 (Workflow / Decision / Secret)、運用拡大時で残 6 を追加。

// Phase marker exception（採用初期で real 実装すべき builder を呼んだ時に投げる）
//
// Go の panic / Rust の FixtureError::Unimplemented / .NET の FixturePhaseUnsupportedException と対称。
// 標準 Error / TypeError ではなく ADR cite 付き独自 class で「設計上の段階展開」を明示する。
export class FixturePhaseUnsupportedError extends Error {
  constructor(
    readonly service: string,
    readonly phase: string,
  ) {
    super(`ADR-TEST-010 PHASE: ${service} mock builder は${phase}で実装（リリース時点 phase marker）`);
    this.name = 'FixturePhaseUnsupportedError';
  }
}

// 12 service の mock builder への entry point
export class MockBuilderRoot {
  constructor(private readonly defaultTenant: string) {}

  state(): StateMockBuilder {
    return new StateMockBuilder(this.defaultTenant);
  }

  audit(): AuditMockBuilder {
    return new AuditMockBuilder(this.defaultTenant);
  }

  pubsub(): PubSubMockBuilder {
    return new PubSubMockBuilder(this.defaultTenant);
  }

  // Workflow / Decision / Secret は採用初期で real 実装する phase marker
  workflow(): never {
    throw new FixturePhaseUnsupportedError('Workflow', '採用初期');
  }
  decision(): never {
    throw new FixturePhaseUnsupportedError('Decision', '採用初期');
  }
  secret(): never {
    throw new FixturePhaseUnsupportedError('Secret', '採用初期');
  }
}

// State service mock data の fluent builder
export class StateMockBuilder {
  private tenant: string;
  private key = '';
  private value: Uint8Array = new Uint8Array();
  private ttl = 0;

  constructor(tenant: string) {
    this.tenant = tenant;
  }

  withTenant(tenant: string): this { this.tenant = tenant; return this; }
  withKey(key: string): this { this.key = key; return this; }
  withValue(value: Uint8Array): this { this.value = value; return this; }
  withTtl(seconds: number): this { this.ttl = seconds; return this; }

  build(): StateEntry {
    return { tenant: this.tenant, key: this.key, value: this.value, ttl: this.ttl };
  }
}

export interface StateEntry {
  tenant: string;
  key: string;
  value: Uint8Array;
  ttl: number;
}

// Audit service mock data の fluent builder
export class AuditMockBuilder {
  private tenant: string;
  private entryCount = 0;
  private startSeq = 0;

  constructor(tenant: string) {
    this.tenant = tenant;
  }

  withTenant(tenant: string): this { this.tenant = tenant; return this; }
  withEntries(n: number): this { this.entryCount = n; return this; }
  withSequence(seq: number): this { this.startSeq = seq; return this; }

  build(): AuditEntry[] {
    const entries: AuditEntry[] = [];
    for (let i = 0; i < this.entryCount; i++) {
      entries.push({
        tenant: this.tenant,
        sequence: this.startSeq + i,
        // 採用初期で SHA-256 prev_id chain を計算
        prevId: '',
      });
    }
    return entries;
  }
}

export interface AuditEntry {
  tenant: string;
  sequence: number;
  prevId: string;
}

// PubSub service mock data の fluent builder
export class PubSubMockBuilder {
  private tenant: string;
  private topic = '';
  private messages = 0;
  private delayMs = 0;

  constructor(tenant: string) {
    this.tenant = tenant;
  }

  withTenant(tenant: string): this { this.tenant = tenant; return this; }
  withTopic(topic: string): this { this.topic = topic; return this; }
  withMessages(n: number): this { this.messages = n; return this; }
  withDelayMs(ms: number): this { this.delayMs = ms; return this; }

  build(): PubSubMessage[] {
    const msgs: PubSubMessage[] = [];
    for (let i = 0; i < this.messages; i++) {
      msgs.push({ tenant: this.tenant, topic: this.topic, seqId: i });
    }
    return msgs;
  }
}

export interface PubSubMessage {
  tenant: string;
  topic: string;
  seqId: number;
}
