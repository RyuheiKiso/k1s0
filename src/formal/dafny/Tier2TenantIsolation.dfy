// k1s0-proof: PROOF-tier2-prog-001 -> IMPL-tier2-0001
// Tier2TenantIsolation.dfy
// tier2 テナント分離 program correctness: クロステナントデータ漏洩の不在を証明
// obligation_id: tier2_tenant_isolation_pcp
// cell_state: stub（Dafny verifier 検証後に v1_baseline_verified に更新する）
// property: NoDataLeak — tenant_id が異なる row は互いに読み書きできないことを保証

// テナント ID を newtype で型安全に表現する
newtype TenantId = id: int | 0 <= id < 1000

// Row は tenant_id と payload を持つ
datatype Row = Row(tenant_id: TenantId, payload: string)

// テーブルはテナントごとに分離された行集合
class TenantIsolatedTable {
    // 各テナントの行集合（ghost で spec のみ使用）
    ghost var tenantData: map<TenantId, set<Row>>

    // 有効なテーブル状態の不変条件
    ghost predicate Valid()
        reads this
    {
        // 全行の tenant_id が対応するキーと一致する（クロステナント混入不在）
        forall tid :: tid in tenantData ==>
            forall row :: row in tenantData[tid] ==> row.tenant_id == tid
    }

    constructor()
        ensures Valid()
        ensures tenantData == map[]
    {
        tenantData := map[];
    }

    // テナント t が行 r を読める条件（自テナントの行のみ）
    method ReadRow(t: TenantId, r: Row) returns (allowed: bool)
        requires Valid()
        ensures allowed <==> r.tenant_id == t
    {
        allowed := r.tenant_id == t;
    }

    // テナント t が行 r を書き込む（クロステナント書込を拒否）
    method WriteRow(t: TenantId, r: Row)
        requires Valid()
        requires r.tenant_id == t       // テナント一致が必須の事前条件
        modifies this
        ensures Valid()
    {
        if t in tenantData {
            tenantData := tenantData[t := tenantData[t] + {r}];
        } else {
            tenantData := tenantData[t := {r}];
        }
    }
}

// NoDataLeak プロパティの検証（Dafny は自動証明）
lemma NoDataLeak(table: TenantIsolatedTable, t1: TenantId, t2: TenantId, r: Row)
    requires table.Valid()
    requires t1 != t2
    requires t1 in table.tenantData
    requires r in table.tenantData[t1]
    ensures r.tenant_id != t2    // t2 は t1 の行を読み出せない
{
    // Valid() の不変条件から r.tenant_id == t1 が導かれる
    // t1 != t2 なので r.tenant_id != t2 は自明
}
