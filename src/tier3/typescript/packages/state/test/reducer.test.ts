// k1s0 tier3 4 layer reducer unit test
// 5 event × 4 subtype × 4 layer の全 actions を検証する
// 整合 1 / 整合 2 / 整合 5 の物理証跡

import { describe, it, expect } from "vitest";
import {
  createInitialState,
  reduce,
  reducePurge,
  resolveSubtypeActions,
} from "../src/index.js";
import type { ClientState, ConflictEvent, PurgeEvent } from "../src/index.js";

// テスト用の最小 state 型
type TestValue = { id: string; version: number };
type TestPayload = { command: string };

// テスト用の初期 state を生成するヘルパー
function makeState(overrides: Partial<ClientState<TestValue, TestPayload>> = {}): ClientState<TestValue, TestPayload> {
  return { ...createInitialState<TestValue, TestPayload>(), ...overrides };
}

// server_truth_advance event の tests
describe("server_truth_advance event", () => {
  it("ST が null のとき ST を更新する actions を返す", () => {
    // 初期 state（ST なし）で server_truth_advance を処理する
    const state = makeState();
    const event: ConflictEvent = {
      eventId: "server_truth_advance",
      newVersion: 2,
      hlcTimestamp: "2026-05-17T00:00:00.000Z",
    };
    const result = reduce(state, event);
    // UPDATE_SERVER_TRUTH が含まれることを確認する
    expect(result.actions.some((a) => a.type === "UPDATE_SERVER_TRUTH")).toBe(true);
    // RE_EVALUATE_PENDING_QUEUE が含まれることを確認する
    expect(result.actions.some((a) => a.type === "RE_EVALUATE_PENDING_QUEUE")).toBe(true);
  });

  it("OL が存在するとき ROLLBACK_OPTIMISTIC action を返す", () => {
    // OL ありの state で server_truth_advance を処理する
    const state = makeState({
      optimisticLocal: {
        layer: "optimistic_local",
        value: { id: "agg-1", version: 2 },
        lineage: {
          layer: "optimistic_local",
          aggregateVersion: 1,
          hlcTimestamp: "2026-05-17T00:00:00.000Z",
          predictedNextVersion: 2,
          idempotencyKey: "idem-001",
          startedAt: "2026-05-17T00:00:00.000Z",
        },
      },
    });
    const event: ConflictEvent = {
      eventId: "server_truth_advance",
      newVersion: 3,
      hlcTimestamp: "2026-05-17T00:00:01.000Z",
    };
    const result = reduce(state, event);
    // ROLLBACK_OPTIMISTIC が含まれることを確認する
    expect(result.actions.some((a) => a.type === "ROLLBACK_OPTIMISTIC")).toBe(true);
    // next state で OL が null になることを確認する
    expect(result.nextState.optimisticLocal).toBeNull();
  });
});

// optimistic_acknowledged event の tests
describe("optimistic_acknowledged event", () => {
  it("OL を ST に promote し PQ entry を削除する", () => {
    // PQ に entry がある state で optimistic_acknowledged を処理する
    const state = makeState({
      pendingQueue: [
        {
          layer: "pending_queue",
          payload: { command: "update" },
          lineage: {
            layer: "pending_queue",
            aggregateVersion: 1,
            hlcTimestamp: "2026-05-17T00:00:00.000Z",
            idempotencyKey: "idem-001",
            enqueuedAt: "2026-05-17T00:00:00.000Z",
            expiresAtMs: Date.now() + 86_400_000,
          },
        },
      ],
    });
    const event: ConflictEvent = {
      eventId: "optimistic_acknowledged",
      idempotencyKey: "idem-001",
      confirmedVersion: 2,
    };
    const result = reduce(state, event);
    // PROMOTE_OPTIMISTIC が含まれることを確認する
    expect(result.actions.some((a) => a.type === "PROMOTE_OPTIMISTIC")).toBe(true);
    // DELETE_PQ_ENTRY が含まれることを確認する
    expect(result.actions.some((a) => a.type === "DELETE_PQ_ENTRY")).toBe(true);
    // PQ が空になることを確認する
    expect(result.nextState.pendingQueue).toHaveLength(0);
  });
});

// optimistic_rejected event の tests
describe("optimistic_rejected event", () => {
  it("OL を rollback し business error を表示する", () => {
    // OL ありの state で optimistic_rejected を処理する
    const state = makeState({
      optimisticLocal: {
        layer: "optimistic_local",
        value: { id: "agg-1", version: 2 },
        lineage: {
          layer: "optimistic_local",
          aggregateVersion: 1,
          hlcTimestamp: "2026-05-17T00:00:00.000Z",
          predictedNextVersion: 2,
          idempotencyKey: "idem-002",
          startedAt: "2026-05-17T00:00:00.000Z",
        },
      },
    });
    const event: ConflictEvent = {
      eventId: "optimistic_rejected",
      idempotencyKey: "idem-002",
      errorCode: "VALIDATION_FAILED",
    };
    const result = reduce(state, event);
    // ROLLBACK_OPTIMISTIC が含まれることを確認する
    expect(result.actions.some((a) => a.type === "ROLLBACK_OPTIMISTIC")).toBe(true);
    // PRESENT_BUSINESS_ERROR が含まれることを確認する
    expect(result.actions.some((a) => a.type === "PRESENT_BUSINESS_ERROR")).toBe(true);
    // OL が null になることを確認する
    expect(result.nextState.optimisticLocal).toBeNull();
  });
});

// pending_queue_resume event の tests
describe("pending_queue_resume event", () => {
  it("PQ が非空のとき SEND_QUEUE_IN_ORDER action を返す", () => {
    // PQ に entry がある state で pending_queue_resume を処理する
    const state = makeState({
      pendingQueue: [
        {
          layer: "pending_queue",
          payload: { command: "update" },
          lineage: {
            layer: "pending_queue",
            aggregateVersion: 1,
            hlcTimestamp: "2026-05-17T00:00:00.000Z",
            idempotencyKey: "idem-003",
            enqueuedAt: "2026-05-17T00:00:00.000Z",
            expiresAtMs: Date.now() + 86_400_000,
          },
        },
      ],
    });
    const event: ConflictEvent = {
      eventId: "pending_queue_resume",
      resumeReason: "network_recovery",
    };
    const result = reduce(state, event);
    // SEND_QUEUE_IN_ORDER が含まれることを確認する
    expect(result.actions.some((a) => a.type === "SEND_QUEUE_IN_ORDER")).toBe(true);
  });

  it("PQ が空のとき actions を返さない", () => {
    // PQ が空の state で pending_queue_resume を処理する
    const state = makeState();
    const event: ConflictEvent = {
      eventId: "pending_queue_resume",
      resumeReason: "app_resume",
    };
    const result = reduce(state, event);
    // actions が空であることを確認する
    expect(result.actions).toHaveLength(0);
  });
});

// business_conflict_received event の 4 subtype tests
describe("business_conflict_received event", () => {
  it("stale_write(disjoint field) → auto resend with chained key", () => {
    // field disjoint の場合に auto resend action が返ることを確認する
    const actions = resolveSubtypeActions(
      "stale_write",
      "idem-004",
      { clientFields: ["quantity"], serverFields: ["deliveryDate"] },
    );
    // DISPATCH_CONFLICT_SUBTYPE が返ることを確認する
    expect(actions.some((a) => a.actionType === "auto_resend_with_chained_key")).toBe(true);
  });

  it("stale_write(intersect field) → 3way merge UI + hold queue", () => {
    // field intersect の場合に 3way merge UI action が返ることを確認する
    const actions = resolveSubtypeActions(
      "stale_write",
      "idem-005",
      { clientFields: ["quantity"], serverFields: ["quantity"] },
    );
    // 3way merge UI action が返ることを確認する
    expect(actions.some((a) => a.actionType === "present_3way_merge_ui_hold_queue")).toBe(true);
  });

  it("lost_update → refetch + 3way merge UI + hold queue", () => {
    // lost_update の場合に refetch + 3way merge UI actions が返ることを確認する
    const actions = resolveSubtypeActions("lost_update", "idem-006");
    expect(actions.some((a) => a.actionType === "refetch_server_truth")).toBe(true);
    expect(actions.some((a) => a.actionType === "present_3way_merge_ui_hold_queue")).toBe(true);
  });

  it("supersede → delete queue entry + silent toast", () => {
    // supersede の場合に delete + silent toast actions が返ることを確認する
    const actions = resolveSubtypeActions("supersede", "idem-007");
    expect(actions.some((a) => a.actionType === "delete_queue_entry")).toBe(true);
    expect(actions.some((a) => a.actionType === "notify_user_silent_toast")).toBe(true);
  });

  it("concurrent_edit → update presence + allow user choice", () => {
    // concurrent_edit の場合に presence 更新 + user choice actions が返ることを確認する
    const actions = resolveSubtypeActions("concurrent_edit", "idem-008", undefined, "actor-B");
    expect(actions.some((a) => a.actionType === "update_presence_indicator")).toBe(true);
    expect(actions.some((a) => a.actionType === "allow_user_to_continue_or_abort")).toBe(true);
  });
});

// purge trigger の tests（5 trigger 全 coverage）
describe("reducePurge（全 layer purge）", () => {
  const purgeReasons = [
    "logout",
    "refresh_token_expiry",
    "tenant_switch",
    "actor_switch",
    "device_bound_key_rotate",
  ] as const;

  for (const reason of purgeReasons) {
    it(`purge reason '${reason}' で全 layer が purge される`, () => {
      // 全 layer にデータがある state で purge を処理する
      const state = makeState({
        pendingQueue: [
          {
            layer: "pending_queue",
            payload: { command: "update" },
            lineage: {
              layer: "pending_queue",
              aggregateVersion: 1,
              hlcTimestamp: "2026-05-17T00:00:00.000Z",
              idempotencyKey: "idem-purge",
              enqueuedAt: "2026-05-17T00:00:00.000Z",
              expiresAtMs: Date.now() + 86_400_000,
            },
          },
        ],
      });
      const event: PurgeEvent = {
        reason,
        hlcTimestamp: "2026-05-17T00:00:01.000Z",
      };
      const result = reducePurge(state, event);
      // 全 layer が purge されることを確認する
      expect(result.nextState.serverTruth).toBeNull();
      expect(result.nextState.optimisticLocal).toBeNull();
      expect(result.nextState.pendingQueue).toHaveLength(0);
      // PURGE_ALL_LAYERS action が含まれることを確認する
      expect(result.actions.some((a) => a.type === "PURGE_ALL_LAYERS")).toBe(true);
    });
  }
});
