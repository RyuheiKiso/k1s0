\* k1s0-proof: PROOF-tier1-tsafe-006 -> IMPL-tier1-0009
\* tier1_tenant_capacity_temporal_safety.tla
\* tier1 テナント容量 temporal safety: noisy neighbor によるテナント間 SLO 侵害禁止の形式検証
\* obligation_id: tier1_tenant_capacity_tsp
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: NoisyNeighborIsolation — テナント A の QPS 超過がテナント B の SLO 侵害を引き起こさない
---- MODULE tier1_tenant_capacity_temporal_safety ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* 定数: テナント数 2（2 テナント間の相互影響を検証するため最小構成で定義する）
N_TENANTS == 2

\* 定数: 各テナントの QPS クォータ上限（1 テナントあたりの最大許容 QPS）
QUOTA_PER_TENANT == 100

\* 変数宣言: tenant_qps は各テナントの現在 QPS を保持する関数（テナント ID → QPS 値）
VARIABLE
    \* @type: Int -> Int;
    tenant_qps

\* 変数宣言: tenant_slo_violated は各テナントの SLO 侵害フラグを保持する関数（テナント ID → Bool）
VARIABLE
    \* @type: Int -> Bool;
    tenant_slo_violated

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* 全テナントの QPS が 0 以上 QUOTA_PER_TENANT * 2 以下であることを保証する
    /\ \A t \in 1..N_TENANTS: tenant_qps[t] >= 0 /\ tenant_qps[t] <= QUOTA_PER_TENANT * 2
    \* 全テナントの SLO 侵害フラグが真偽値であることを保証する
    /\ \A t \in 1..N_TENANTS: tenant_slo_violated[t] \in BOOLEAN

\* 初期状態: 全テナントの QPS=0, SLO 侵害なしから開始する
Init ==
    \* 全テナントの QPS を 0 に初期化する
    /\ tenant_qps = [t \in 1..N_TENANTS |-> 0]
    \* 全テナントの SLO 侵害フラグを FALSE に初期化する
    /\ tenant_slo_violated = [t \in 1..N_TENANTS |-> FALSE]

\* アクション: テナント t の QPS を増加させる（負荷増大をシミュレートする）
IncreaseQPS(t) ==
    \* QPS が上限の 2 倍未満の場合のみ増加させる（状態空間を有限化するため）
    /\ tenant_qps[t] < QUOTA_PER_TENANT * 2
    \* テナント t の QPS を 10 増加させる（バースト的な負荷増大を模擬）
    /\ tenant_qps' = [tenant_qps EXCEPT ![t] = tenant_qps[t] + 10]
    \* tenant_slo_violated は変化しない
    /\ UNCHANGED tenant_slo_violated

\* アクション: テナント t の QPS を減少させる（負荷低下をシミュレートする）
DecreaseQPS(t) ==
    \* QPS が 0 超の場合のみ減少させる
    /\ tenant_qps[t] > 0
    \* テナント t の QPS を 10 減少させる
    /\ tenant_qps' = [tenant_qps EXCEPT ![t] = tenant_qps[t] - 10]
    \* tenant_slo_violated は変化しない
    /\ UNCHANGED tenant_slo_violated

\* アクション: テナント t の SLO 侵害を検出する（クォータ超過による SLO 違反を記録）
DetectViolation(t) ==
    \* テナント t の QPS がクォータを超過していることを前提とする
    /\ tenant_qps[t] > QUOTA_PER_TENANT
    \* テナント t の SLO 侵害フラグを TRUE に設定する（当該テナントのみ）
    /\ tenant_slo_violated' = [tenant_slo_violated EXCEPT ![t] = TRUE]
    \* tenant_qps は変化しない
    /\ UNCHANGED tenant_qps

\* 全遷移の定義: いずれかのテナントに対していずれかのアクションを実行する
Next ==
    \* 任意のテナント t に対して IncreaseQPS を実行する
    \/ \E t \in 1..N_TENANTS: IncreaseQPS(t)
    \* 任意のテナント t に対して DecreaseQPS を実行する
    \/ \E t \in 1..N_TENANTS: DecreaseQPS(t)
    \* 任意のテナント t に対して DetectViolation を実行する
    \/ \E t \in 1..N_TENANTS: DetectViolation(t)

\* safety property: noisy neighbor 分離（テナント A の超過がテナント B の SLO 侵害を引き起こさない）
\* 09_テナント容量適合仕様の noisy_neighbor_isolation を形式化する
NoisyNeighborIsolation ==
    \* 全てのテナントペアについて検証する
    \A t1, t2 \in 1..N_TENANTS:
        \* t1 と t2 が異なるテナントである場合
        t1 /= t2 =>
            \* t1 が QPS 超過していても t2 の SLO が侵害されないことを保証する
            ~(tenant_qps[t1] > QUOTA_PER_TENANT /\ tenant_slo_violated[t2] = TRUE)

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<tenant_qps, tenant_slo_violated>>

\* 定理: Spec が成立すれば NoisyNeighborIsolation が常に成立する
THEOREM Spec => []NoisyNeighborIsolation
====
