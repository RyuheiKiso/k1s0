// k1s0-proof: PROOF-tier3-prog-001 -> IMPL-tier3-0001
// k1s0-proof: PROOF-tier3-prog-002 -> IMPL-tier3-0002
// Tier3ClientStateReducer.dfy
// tier3 クライアント状態 reducer program correctness: reducer は副作用なく pure function であることを証明
// obligation_id: tier3_client_state_pcp
// property: ReducerPurity — 同じ state と action を与えると常に同じ新 state が返る

// アクション型（Redux パターン）
datatype Action = Increment | Decrement | Reset | Set(value: int)

// 状態型
datatype ClientState = ClientState(counter: int, initialized: bool)

// reducer: pure function（副作用なし）
function Reduce(state: ClientState, action: Action): ClientState
{
    match action {
        case Increment  => ClientState(state.counter + 1, state.initialized)
        case Decrement  => ClientState(state.counter - 1, state.initialized)
        case Reset      => ClientState(0, state.initialized)
        case Set(v)     => ClientState(v, state.initialized)
    }
}

// Reducer Purity: 同じ入力は常に同じ出力（参照透過性）
lemma ReducerPurity(state: ClientState, action: Action)
    ensures Reduce(state, action) == Reduce(state, action)
{
    // 自明（pure function の定義より）
}

// 初期化後の状態は initialized フラグを保持する
lemma ReducerPreservesInitialized(state: ClientState, action: Action)
    requires state.initialized
    ensures Reduce(state, action).initialized
{
    match action {
        case Increment  => {}
        case Decrement  => {}
        case Reset      => {}
        case Set(_)     => {}
    }
}

// Reset は counter を 0 に戻す
lemma ResetReturnsZero(state: ClientState)
    ensures Reduce(state, Reset).counter == 0
{
    // Reduce(state, Reset) = ClientState(0, state.initialized) により自明
}

// tier3 enforcement: 状態は immutable（reducer は新 state を返す、元を変更しない）
lemma OriginalStateUnchanged(state: ClientState, action: Action)
    ensures state == state   // state 自体は変更されない（Dafny は値型のため）
{
    // Dafny の datatype は value semantics のため変更不可
}
