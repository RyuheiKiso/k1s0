-- CrossCutting/Bff/TokenExpiry.lean — BFF トークン失効後アクセス拒否公理
-- obligation_id: cross_bff_axiom_078 (v1_property_axiom, tool_kind=lean4)
-- statement: 有効期限切れのトークンは必ずアクセスが拒否されることを証明する
-- tool: Lean 4 (omega を使用)
-- k1s0-proof: PROOF-cross-bff-lean4-001 -> IMPL-crosscutting-bff-001
-- k1s0-impl: IMPL-crosscutting-bff-001 realizes=FR-cross-bff-001

namespace K1s0Formal.CrossCutting.Bff

-- JWT トークンの抽象表現
structure JwtToken where
  -- sub: サブジェクト (ユーザー ID)
  sub       : Nat
  -- exp: 有効期限 (Unix timestamp 相当の自然数)
  exp       : Nat
  -- tenant_id: テナント ID
  tenant_id : Nat
  deriving Repr

-- 現在時刻 (自然数で抽象化)
abbrev Now := Nat

-- トークン検証: 有効期限内かどうかを確認する
def tokenValid (tok : JwtToken) (now : Now) : Bool :=
  -- now < exp であれば有効 (exp は exclusive なので ≤ でなく < を使う)
  now < tok.exp

-- アクセス決定: トークンが有効でなければ拒否
def accessAllowed (tok : JwtToken) (now : Now) : Bool :=
  tokenValid tok now

-- 主定理: トークンが失効していれば必ずアクセスが拒否される
theorem expired_token_denied (tok : JwtToken) (now : Now)
    (hexp : tok.exp ≤ now) :
    accessAllowed tok now = false := by
  -- accessAllowed の定義を展開する
  simp [accessAllowed, tokenValid]
  -- tok.exp ≤ now なので ¬(now < tok.exp)
  omega

-- 補題: 有効なトークンは現在時刻より exp が大きい
theorem valid_token_not_expired (tok : JwtToken) (now : Now)
    (hvalid : tokenValid tok now = true) :
    tok.exp > now := by
  simp [tokenValid] at hvalid
  exact hvalid

-- 定理: 時刻が進むとトークンの有効性は単調減少 (一度失効したら復活しない)
theorem token_validity_monotone_decrease (tok : JwtToken) (t1 t2 : Now)
    (hle : t1 ≤ t2) (hexp : accessAllowed tok t1 = false) :
    accessAllowed tok t2 = false := by
  -- t1 で拒否されているなら tok.exp ≤ t1
  simp [accessAllowed, tokenValid] at *
  -- t1 ≤ t2 と tok.exp ≤ t1 から tok.exp ≤ t2 が導かれる
  omega

-- 系: 全テナントに対して有効期限チェックは一様
theorem expiry_check_tenant_invariant (tok : JwtToken) (now : Now)
    (hexp : tok.exp ≤ now) (tid : Nat) :
    accessAllowed { tok with tenant_id := tid } now = false := by
  exact expired_token_denied { tok with tenant_id := tid } now hexp

end K1s0Formal.CrossCutting.Bff
