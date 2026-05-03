// src/sdk/typescript/test-fixtures/src/fixture.ts
//
// k1s0 TypeScript SDK test-fixtures: Setup / Teardown / Fixture class
// （領域 1 + 領域 2、ADR-TEST-010 §3）。
//
// Vitest beforeAll / afterAll 経由で利用する想定。

import { type Options, defaultOptions } from './options.js';
import { MockBuilderRoot } from './mockBuilder.js';

// Setup の戻り値。test 内で SDK client init / mock builder を取得する経路。
export class Fixture {
  // Setup に渡された Options（既定値補完済）
  readonly options: Required<Options>;
  // 12 service の mock data builder への entry point
  readonly mockBuilder: MockBuilderRoot;

  constructor(options: Required<Options>) {
    this.options = options;
    this.mockBuilder = new MockBuilderRoot(options.tenant);
  }

  // 後片付け（採用初期で tools/e2e/user/down.sh を child_process spawn）
  async teardown(): Promise<void> {
    // skeleton: 採用初期で down.sh spawn を実装
  }

  // tier1 facade Pod が Ready になるまで待機（採用初期で kubectl wait wrapper）
  async waitForTier1FacadeReady(): Promise<void> {
    // skeleton: 採用初期で k8s client 経由で実装
  }

  // SDK client 生成（採用初期で @k1s0/sdk-rpc.K1s0Client wrapper として実装）
  newSDKClient(tenant: string): SDKClient {
    return new SDKClient(tenant || this.options.tenant, this);
  }
}

// SDK client の薄い wrapper（採用初期で @k1s0/sdk-rpc を内包）
export class SDKClient {
  constructor(
    readonly tenant: string,
    private readonly fixture: Fixture,
  ) {}

  // State.Set RPC（採用初期で sdkRpc.state.set wrapper）
  async setState(_key: string, _value: Uint8Array): Promise<void> {
    // skeleton（採用初期で実装）
  }

  // State.Get RPC
  async getState(_key: string): Promise<Uint8Array | null> {
    return null;
  }
}

// fixtures namespace（4 言語対称の entry: fixtures.setup / fixtures.teardown）
export const fixtures = {
  // Setup: kind cluster 起動 + k1s0 install + SDK client の前提整備
  // 採用初期で kind / helm / kubectl の child_process spawn を実装。
  // リリース時点では skeleton（cluster 起動済前提で fixture を返す）。
  async setup(opts: Options = {}): Promise<Fixture> {
    // 既定値補完
    const merged: Required<Options> = {
      ...defaultOptions,
      ...opts,
      addOns: opts.addOns ?? defaultOptions.addOns,
    };
    return new Fixture(merged);
  },

  // 明示的 teardown（fixture.teardown() の薄い proxy、4 言語対称化のため）
  async teardown(fx: Fixture): Promise<void> {
    await fx.teardown();
  },
};
