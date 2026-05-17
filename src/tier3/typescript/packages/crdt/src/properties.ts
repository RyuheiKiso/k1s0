// k1s0 tier3 CRDT 収束法則プロパティ検証
// CRDT（Conflict-free Replicated Data Type）が満たすべき数学的法則を関数として提供する
// fast-check などの PBT（Property-Based Testing）ライブラリから呼び出して収束性を検証する
// 参考: "A comprehensive study of CRDTs" (Shapiro et al., 2011)

// 適用関数の型（状態に操作を適用して新しい状態を返す pure function）
// state: 現在の状態
// op: 適用する操作
// 戻り値: 操作を適用した後の新しい状態
export type ApplyFn<T> = (state: T, op: T) => T;

// 2 つの状態が等価であるかを判定する比較関数の型
// 等価判定はアプリケーション固有のセマンティクスに依存するため外部から渡す
export type EqualityFn<T> = (a: T, b: T) => boolean;

// isCommutative: 交換法則の検証
// CRDT が交換律を満たす場合、操作の適用順序によらず結果が同一になる
// 定義: apply(apply(state, op1), op2) === apply(apply(state, op2), op1)
// op1: 最初の操作
// op2: 2 番目の操作
// apply: 操作を状態に適用する純粋関数
// equalsFn: 等価判定関数（省略時は JSON 文字列比較を使用する）
// 戻り値: 交換律が成立すれば true を返す
export function isCommutative<T>(
    // 初期状態（操作を適用する前の状態）
    initialState: T,
    // 最初の操作
    op1: T,
    // 2 番目の操作
    op2: T,
    // 状態に操作を適用する純粋関数
    apply: ApplyFn<T>,
    // 等価判定関数（省略時は JSON 直列化で比較する）
    equalsFn?: EqualityFn<T>,
): boolean {
    // 操作を op1 → op2 の順に適用した結果を計算する
    const resultA = apply(apply(initialState, op1), op2);
    // 操作を op2 → op1 の順に適用した結果を計算する
    const resultB = apply(apply(initialState, op2), op1);
    // 等価判定関数が指定されている場合はそれを使用する
    if (equalsFn !== undefined) {
        // カスタム等価判定で交換律を確認する
        return equalsFn(resultA, resultB);
    }
    // 等価判定関数が省略された場合は JSON 文字列比較を使用する
    // JSON.stringify の順序非依存性に注意: オブジェクトのプロパティ順が一致していることが前提
    return JSON.stringify(resultA) === JSON.stringify(resultB);
}

// isAssociative: 結合法則の検証
// CRDT が結合律を満たす場合、操作のグループ化順序によらず結果が同一になる
// 定義: apply(apply(apply(state, op1), op2), op3) === apply(apply(state, op1), apply(state, op2, op3))
// 注意: CRDT の merge が結合律を満たすかを検証する（モノイド则の一部）
// op1: 最初の操作
// op2: 2 番目の操作
// op3: 3 番目の操作
// apply: 操作を状態に適用する純粋関数
// equalsFn: 等価判定関数（省略時は JSON 文字列比較を使用する）
// 戻り値: 結合律が成立すれば true を返す
export function isAssociative<T>(
    // 初期状態
    initialState: T,
    // 最初の操作
    op1: T,
    // 2 番目の操作
    op2: T,
    // 3 番目の操作
    op3: T,
    // 状態に操作を適用する純粋関数
    apply: ApplyFn<T>,
    // 等価判定関数（省略時は JSON 直列化で比較する）
    equalsFn?: EqualityFn<T>,
): boolean {
    // 左結合: (state op1 op2) op3 の順に適用する
    const leftAssoc = apply(apply(apply(initialState, op1), op2), op3);
    // 右結合: state op1 (op2 op3) の順に適用する
    // op2 と op3 を先にまとめて適用してから op1 と組み合わせる
    const rightAssoc = apply(apply(initialState, op1), apply(apply(initialState, op2), op3));
    // 等価判定関数が指定されている場合はそれを使用する
    if (equalsFn !== undefined) {
        // カスタム等価判定で結合律を確認する
        return equalsFn(leftAssoc, rightAssoc);
    }
    // JSON 文字列比較で結合律を確認する
    return JSON.stringify(leftAssoc) === JSON.stringify(rightAssoc);
}

// isIdempotent: 冪等法則の検証
// CRDT が冪等律を満たす場合、同一操作を複数回適用しても結果が変わらない
// 定義: apply(apply(state, op), op) === apply(state, op)
// これはネットワーク重複配送に対する安全性の保証に直結する
// op: 検証する操作（同一操作を 2 回適用しても結果が同じになるべき）
// apply: 操作を状態に適用する純粋関数
// equalsFn: 等価判定関数（省略時は JSON 文字列比較を使用する）
// 戻り値: 冪等律が成立すれば true を返す
export function isIdempotent<T>(
    // 初期状態
    initialState: T,
    // 検証する操作
    op: T,
    // 状態に操作を適用する純粋関数
    apply: ApplyFn<T>,
    // 等価判定関数（省略時は JSON 直列化で比較する）
    equalsFn?: EqualityFn<T>,
): boolean {
    // 操作を 1 回適用した結果を計算する
    const onceApplied = apply(initialState, op);
    // 操作を 2 回適用した結果を計算する（冪等性の検証）
    const twiceApplied = apply(onceApplied, op);
    // 等価判定関数が指定されている場合はそれを使用する
    if (equalsFn !== undefined) {
        // カスタム等価判定で冪等律を確認する
        return equalsFn(onceApplied, twiceApplied);
    }
    // JSON 文字列比較で冪等律を確認する
    return JSON.stringify(onceApplied) === JSON.stringify(twiceApplied);
}

// checkCRDTConvergence: 3 法則を一括で検証するヘルパー関数
// CRDT の完全な収束性（交換律 + 結合律 + 冪等律）を一度に確認する
// fast-check の property テストから直接呼び出すことを想定している
// initialState: 初期状態
// op1: 最初の操作
// op2: 2 番目の操作
// op3: 3 番目の操作（結合律検証に使用）
// apply: 操作を状態に適用する純粋関数
// 戻り値: 3 法則の検証結果をまとめた記録
export function checkCRDTConvergence<T>(
    // 初期状態
    initialState: T,
    // 最初の操作
    op1: T,
    // 2 番目の操作
    op2: T,
    // 3 番目の操作（結合律検証用）
    op3: T,
    // 状態に操作を適用する純粋関数
    apply: ApplyFn<T>,
    // 等価判定関数（省略時は JSON 直列化で比較する）
    equalsFn?: EqualityFn<T>,
): { commutative: boolean; associative: boolean; idempotent: boolean; allPassed: boolean } {
    // 交換律を検証する（op1 と op2 の順序を入れ替えても結果が同じであることを確認）
    const commutative = isCommutative(initialState, op1, op2, apply, equalsFn);
    // 結合律を検証する（op1 / op2 / op3 のグループ化順序によらず結果が同じであることを確認）
    const associative = isAssociative(initialState, op1, op2, op3, apply, equalsFn);
    // 冪等律を検証する（op1 を 2 回適用しても 1 回適用と同じであることを確認）
    const idempotent = isIdempotent(initialState, op1, apply, equalsFn);
    // 全法則が成立するかを集約する
    const allPassed = commutative && associative && idempotent;
    // 検証結果オブジェクトを返す
    return { commutative, associative, idempotent, allPassed };
}
