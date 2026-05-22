// k1s0-proof: PROOF-tier2-prog-002 -> IMPL-tier2-0002
// Tier2Enforcement.dfy
// tier2 強制機構 program correctness: tier2 コードが tier1 公開 API 以外を参照しないことを証明
// obligation_id: tier2_enforcement_pcp
// property: Tier1ApiOnlyAccess — tier2 は tier1 の facade API のみ参照可能

datatype ApiKind = Tier1Facade | Tier1Internal | Tier2Internal | Tier3Internal

// import 参照を表す型
datatype Import = Import(target: ApiKind, caller_tier: int)

// tier2 からの import が許可されるか判定する
function IsTier2ImportAllowed(imp: Import): bool
    requires imp.caller_tier == 2
{
    // tier2 は tier1 facade のみ参照可能（internal は不可）
    imp.target == Tier1Facade || imp.target == Tier2Internal
}

// 全 import の検証
function ValidateTier2Imports(imports: seq<Import>): bool
    requires forall i :: 0 <= i < |imports| ==> imports[i].caller_tier == 2
{
    forall i :: 0 <= i < |imports| ==> IsTier2ImportAllowed(imports[i])
}

// Tier1ApiOnlyAccess: tier2 が tier1 internal を参照すると必ず失敗
lemma Tier1InternalIsForbidden(imp: Import)
    requires imp.caller_tier == 2
    requires imp.target == Tier1Internal
    ensures !IsTier2ImportAllowed(imp)
{
    // IsTier2ImportAllowed は tier1 internal を false にする
}

// 違反 import を含む list は ValidateTier2Imports で false
lemma ViolationImportFails(imports: seq<Import>, violating: Import)
    requires forall i :: 0 <= i < |imports| ==> imports[i].caller_tier == 2
    requires violating in imports
    requires violating.caller_tier == 2
    requires violating.target == Tier1Internal
    ensures !ValidateTier2Imports(imports)
{
    var idx :| 0 <= idx < |imports| && imports[idx] == violating;
    Tier1InternalIsForbidden(violating);
}
