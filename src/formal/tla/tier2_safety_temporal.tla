\* k1s0-proof: PROOF-tier2-tsafe-005 -> IMPL-tier2-0005
\* tier2_safety_temporal.tla
\* tier2 テナント分離 temporal safety: 他テナントへの write 禁止の形式検証
\* obligation_id: tier2_safety_005
\* cell_state: stub（Apalache で検証後に v1_baseline_verified に更新する）
\* property: TenantIsolation — テナント A の write がテナント B に影響を与えない
---- MODULE tier2_safety_temporal ----
\* 標準ライブラリ Naturals と TLC をインポートする（整数演算とモデル検査に使用）
EXTENDS Naturals, TLC

\* テナント ID の最大値を定義する（状態空間を有限に抑えるため）
MAX_TENANT == 1

\* 変数宣言: tenant_id は現在の操作元テナント ID を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    tenant_id

\* 変数宣言: target_tenant は write 対象のテナント ID を保持する（Apalache type: Int）
VARIABLE
    \* @type: Int;
    target_tenant

\* 変数宣言: write_allowed は write 操作が許可されているかを保持する（Apalache type: Bool）
VARIABLE
    \* @type: Bool;
    write_allowed

\* 型不変条件: 全変数が許容型・値域を満たすことを保証する
TypeInvariant ==
    \* tenant_id は 0 以上 MAX_TENANT 以下の整数でなければならない
    /\ tenant_id >= 0 /\ tenant_id <= MAX_TENANT
    \* target_tenant は 0 以上 MAX_TENANT 以下の整数でなければならない
    /\ target_tenant >= 0 /\ target_tenant <= MAX_TENANT
    \* write_allowed は真偽値でなければならない
    /\ write_allowed \in BOOLEAN

\* 初期状態: tenant_id=0, target_tenant=0, write_allowed=FALSE から開始する
Init ==
    \* tenant_id の初期値は 0（最初のテナント）に設定する
    /\ tenant_id = 0
    \* target_tenant の初期値は 0（自テナント）に設定する
    /\ target_tenant = 0
    \* write_allowed の初期値は FALSE（未許可）に設定する
    /\ write_allowed = FALSE

\* アクション: 同一テナントへの write を要求する（自テナント書き込みは許可される）
RequestSameTenantWrite ==
    \* target_tenant を tenant_id と同じ値に設定する
    /\ target_tenant' = tenant_id
    \* 同一テナントへの write は許可する
    /\ write_allowed' = TRUE
    \* tenant_id は変化しない
    /\ UNCHANGED tenant_id

\* アクション: 他テナントへの write を試みる（クロステナント書き込みは拒否される）
AttemptCrossTenantWrite ==
    \* target_tenant を tenant_id 以外の値に変更する（クロステナントを模擬する）
    \* tenant_id=0 なら target_tenant=1、tenant_id=1 なら target_tenant=0 にする
    /\ target_tenant' = 1 - tenant_id
    \* クロステナント write は拒否する（write_allowed = FALSE を維持する）
    /\ write_allowed' = FALSE
    \* tenant_id は変化しない
    /\ UNCHANGED tenant_id

\* アクション: 操作テナントを切り替える（次のテナントに移行する）
SwitchTenant ==
    \* tenant_id を循環させる（0→1→0）
    /\ tenant_id' = 1 - tenant_id
    \* tenant 切り替え時は write_allowed をリセットする
    /\ write_allowed' = FALSE
    \* target_tenant を自テナントに戻す
    /\ target_tenant' = 1 - tenant_id

\* アクション: Sink ステートのループ遷移（deadlock 回避用の no-op 遷移）
Stutter ==
    \* 全変数を変化させない（stutter 遷移）
    /\ UNCHANGED tenant_id
    /\ UNCHANGED target_tenant
    /\ UNCHANGED write_allowed

\* 全遷移の定義: 4 つのアクションのいずれかを実行する
Next ==
    \* 同一テナント write 要求アクションを選択する
    \/ RequestSameTenantWrite
    \* クロステナント write 試行アクションを選択する
    \/ AttemptCrossTenantWrite
    \* テナント切り替えアクションを選択する
    \/ SwitchTenant
    \* Stutter アクションを選択する
    \/ Stutter

\* safety property: write_allowed = TRUE の場合、tenant_id = target_tenant でなければならない
\* テナント A の write がテナント B に影響を与えないことを形式化する
TenantIsolation ==
    \* write が許可されている場合のみ自テナントへの write であることを保証する
    write_allowed = TRUE => tenant_id = target_tenant

\* spec 定義: 初期状態 + 次状態遷移の結合
Spec ==
    \* 初期条件 Init から出発する
    /\ Init
    \* Next を時間ステップごとに実行する（stuttering を許容）
    /\ [][Next]_<<tenant_id, target_tenant, write_allowed>>

\* 定理: Spec が成立すれば TenantIsolation が常に成立する
THEOREM Spec => []TenantIsolation
====
