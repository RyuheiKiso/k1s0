\* k1s0-proof: PROOF-tier1-tsafe-007 -> IMPL-tier1-0003
\* tier1_observability_signal_order.tla
\* tier1 観測信号順序保証: ログ・トレース・メトリクス信号が因果順序を維持する temporal safety
\* obligation_id: tier1_observability_tsp
\* property: SignalCausalOrder — 先行 span が完了した後にのみ後続 span が collector に届く
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
---- MODULE tier1_observability_signal_order ----
\* OpenTelemetry 信号の因果順序モデル
EXTENDS Naturals, Sequences, TLC

\* signal_type は 3 種類（log / trace / metric）
CONSTANTS Log, Trace, Metric
\* SignalTypes は 3 型の和集合
SignalTypes == {Log, Trace, Metric}

\* 送信済み信号キュー（タイムスタンプ付き）
VARIABLES emitted, delivered

TypeInvariant ==
    /\ emitted \in Seq(SignalTypes)
    /\ delivered \in Seq(SignalTypes)

\* 初期状態: 信号キューは空
Init ==
    /\ emitted = <<>>
    /\ delivered = <<>>

\* 信号を追加する（any タイプ）
EmitSignal(sig) ==
    /\ emitted' = Append(emitted, sig)
    /\ delivered' = delivered

\* 配信: emitted の先頭を delivered に移動する（FIFO 順序保証）
DeliverSignal ==
    /\ emitted # <<>>
    /\ delivered' = Append(delivered, Head(emitted))
    /\ emitted' = Tail(emitted)

Next == \/ \E sig \in SignalTypes : EmitSignal(sig)
        \/ DeliverSignal

\* temporal safety: delivered は emitted の prefix であること（因果順序）
SignalCausalOrder ==
    \A i \in DOMAIN delivered : delivered[i] = emitted[i]

Spec == Init /\ [][Next]_<<emitted, delivered>>

THEOREM Spec => []SignalCausalOrder
====
