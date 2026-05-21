// k1s0 tier3 CRDT property-based test
// 11_クライアント状態適合仕様.md §CRDT の 3 条件を property test で検証する
// 3 条件: Commutative（交換則）/ Associative（結合則）/ Idempotent（冪等性）

// vitest テストフレームワークをインポートする
import { describe, it, expect } from "vitest";
// CRDT モジュールの関数をインポートする（src/tier3/typescript/packages/crdt/src/index.ts）
import {
  mergeLWW,
  createGSet,
  addToGSet,
  mergeGSet,
  incrementClock,
  mergeClock,
} from "../src/index";
// LWWRegister 型をインポートする
import type { LWWRegister, VectorClock } from "../src/index";

// CRDT 3 条件の property test スイート
describe("CRDT property tests — 3 invariants", () => {
  // LWWRegister の Idempotent 性テスト
  // 同じ register を 2 回 merge しても結果が変わらないことを検証する
  it("LWWRegister.mergeLWW is idempotent", () => {
    // 古い register を初期化する
    const regA: LWWRegister<string> = {
      // 初期値を設定する
      value: "init",
      // HLC タイムスタンプを設定する（古い値）
      hlcTimestamp: "2024-01-01T00:00:00.000Z",
      // actor ID を設定する
      actorId: "actor-1",
    };
    // 更新後の register を作成する（新しい HLC タイムスタンプ）
    const regB: LWWRegister<string> = {
      // 更新後の値を設定する
      value: "hello",
      // 新しい HLC タイムスタンプを設定する
      hlcTimestamp: "2024-01-01T00:01:00.000Z",
      // actor ID を設定する
      actorId: "actor-1",
    };
    // A.merge(B) を 1 回実行する
    const merged1 = mergeLWW(regA, regB);
    // さらにもう 1 回同じ値で merge する（冪等性確認）
    const merged2 = mergeLWW(merged1, regB);
    // 結果が等しいことを確認する（冪等性: merge 回数に関係なく同じ結果）
    expect(merged1.value).toBe(merged2.value);
    // タイムスタンプも等しいことを確認する
    expect(merged1.hlcTimestamp).toBe(merged2.hlcTimestamp);
  });

  // GSet の Commutative 性テスト
  // mergeGSet(A, B) と mergeGSet(B, A) の結果が等しいことを検証する
  it("mergeGSet is commutative", () => {
    // セット A を初期化して要素を追加する
    const setA = addToGSet(addToGSet(createGSet<string>(), "apple"), "banana");
    // セット B を初期化して要素を追加する
    const setB = addToGSet(addToGSet(createGSet<string>(), "cherry"), "apple");
    // A.merge(B) を計算する
    const mergeAB = mergeGSet(setA, setB);
    // B.merge(A) を計算する（交換則確認）
    const mergeBA = mergeGSet(setB, setA);
    // 両方の要素数が等しいことを確認する（交換則: 順序によらず同じ結果）
    expect(mergeAB.elements.size).toBe(mergeBA.elements.size);
    // A.merge(B) の全要素が B.merge(A) にも含まれることを確認する
    for (const elem of mergeAB.elements) {
      // 各要素の存在を確認する
      expect(mergeBA.elements.has(elem)).toBe(true);
    }
  });

  // GSet の Associative 性テスト
  // mergeGSet(mergeGSet(A, B), C) と mergeGSet(A, mergeGSet(B, C)) が等しいことを検証する
  it("mergeGSet is associative", () => {
    // セット A を初期化する
    const setA = addToGSet(addToGSet(createGSet<number>(), 1), 2);
    // セット B を初期化する
    const setB = addToGSet(addToGSet(createGSet<number>(), 3), 4);
    // セット C を初期化する
    const setC = addToGSet(addToGSet(createGSet<number>(), 5), 2);
    // (A.merge(B)).merge(C) を計算する
    const left = mergeGSet(mergeGSet(setA, setB), setC);
    // A.merge(B.merge(C)) を計算する（結合則確認）
    const right = mergeGSet(setA, mergeGSet(setB, setC));
    // 結果が等しいことを確認する（結合則: グループ化によらず同じ結果）
    expect(left.elements.size).toBe(right.elements.size);
    // left の全要素が right にも含まれることを確認する
    for (const elem of left.elements) {
      // 各要素の存在を確認する
      expect(right.elements.has(elem)).toBe(true);
    }
  });

  // GSet の Idempotent 性テスト
  // mergeGSet(A, A) が A と等しいことを検証する
  it("mergeGSet is idempotent", () => {
    // セットを初期化する
    const setA = addToGSet(addToGSet(createGSet<string>(), "x"), "y");
    // 自分自身と merge する（冪等性確認）
    const merged = mergeGSet(setA, setA);
    // 要素数が変わらないことを確認する
    expect(merged.elements.size).toBe(setA.elements.size);
  });

  // VectorClock の Idempotent 性テスト
  // mergeClock(vc, vc) が vc と等しいことを検証する
  it("mergeClock is idempotent", () => {
    // VectorClock を初期化して値を設定する
    let vc: VectorClock = {};
    // node-1 を increment する
    vc = incrementClock(vc, "node-1");
    // node-2 を increment する
    vc = incrementClock(vc, "node-2");
    // node-1 をもう 1 回 increment する
    vc = incrementClock(vc, "node-1");
    // 自分自身と merge する（冪等性確認）
    const merged = mergeClock(vc, vc);
    // node-1 の値が変わらないことを確認する
    expect(merged["node-1"]).toBe(vc["node-1"]);
    // node-2 の値が変わらないことを確認する
    expect(merged["node-2"]).toBe(vc["node-2"]);
  });

  // VectorClock の Commutative 性テスト
  // mergeClock(A, B) と mergeClock(B, A) が等しいことを検証する
  it("mergeClock is commutative", () => {
    // VectorClock A を初期化する
    const clockA: VectorClock = { "node-1": 3, "node-2": 1 };
    // VectorClock B を初期化する
    const clockB: VectorClock = { "node-1": 1, "node-3": 5 };
    // A.merge(B) を計算する
    const mergeAB = mergeClock(clockA, clockB);
    // B.merge(A) を計算する（交換則確認）
    const mergeBA = mergeClock(clockB, clockA);
    // node-1 の値が等しいことを確認する（各 actor の max が取られる）
    expect(mergeAB["node-1"]).toBe(mergeBA["node-1"]);
    // node-2 の値が等しいことを確認する
    expect(mergeAB["node-2"]).toBe(mergeBA["node-2"]);
    // node-3 の値が等しいことを確認する
    expect(mergeAB["node-3"]).toBe(mergeBA["node-3"]);
  });
});
