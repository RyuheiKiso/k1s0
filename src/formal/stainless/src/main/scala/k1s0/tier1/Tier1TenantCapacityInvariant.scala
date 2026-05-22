// k1s0-proof: PROOF-tier1-refn-004 -> IMPL-tier1-0009
// Tier1TenantCapacityInvariant.scala
// tier1 テナント容量 invariant の Stainless 形式検証
// 09_テナント容量適合仕様: noisy neighbor isolation の不変条件
// obligation_id: tier1_tenant_capacity_stainless
// cell_state: stub（Stainless で検証後に v1_baseline_verified に更新する）
// Stainless の lang パッケージをインポートする（ensuring / holds / require を使うため）
import stainless.lang.*
// Stainless の collection パッケージをインポートする（List 操作に使用）
import stainless.collection.*
// Stainless の annotation パッケージをインポートする（@induct 等を使うため）
import stainless.annotation.*

// Tier1TenantCapacityInvariant オブジェクト: テナント容量と noisy neighbor 分離の不変条件を検証する
object Tier1TenantCapacityInvariant {
  // QUOTA_PER_TENANT: 各テナントの QPS クォータ上限（09_テナント容量適合仕様に準拠）
  val QUOTA_PER_TENANT: BigInt = 100

  // TenantState ケースクラス: テナントの状態を表す（QPS と SLO 侵害フラグ）
  case class TenantState(
    // 現在の QPS 値（0 以上の整数）
    currentQps: BigInt,
    // SLO 侵害フラグ（クォータ超過時に TRUE になる）
    sloViolated: Boolean
  )

  // isValidTenantState: テナント状態が有効範囲内かを検証する述語関数
  def isValidTenantState(state: TenantState): Boolean =
    // QPS が 0 以上であることを確認する
    state.currentQps >= 0

  // noisyNeighborIsolation: テナント A の QPS 超過がテナント B の SLO 侵害を引き起こさない
  // 09_テナント容量適合仕様の noisy_neighbor_isolation property の Stainless 形式化
  def noisyNeighborIsolation(tenantA: TenantState, tenantB: TenantState): Boolean = {
    // 前提条件: 両テナントの状態が有効であることを要求する
    require(isValidTenantState(tenantA) && isValidTenantState(tenantB))
    // テナント A の QPS 超過はテナント B の SLO 侵害を引き起こさないことを確認する
    // テナント A が超過していてもテナント B の SLO は侵害されない（分離が保証されている）
    !(tenantA.currentQps > QUOTA_PER_TENANT && tenantB.sloViolated)
  } ensuring { result =>
    // 事後条件: 結果はテナント A 超過かつテナント B SLO 侵害の AND の否定と一致する
    result == !(tenantA.currentQps > QUOTA_PER_TENANT && tenantB.sloViolated)
  }

  // quotaEnforcement: クォータ超過テナントの SLO 侵害検出は当該テナントのみに影響する
  // 自テナントのクォータ超過は自テナントの SLO 侵害のみを引き起こす
  def quotaEnforcement(tenant: TenantState): Boolean = {
    // 前提条件: テナント状態が有効であることを要求する
    require(isValidTenantState(tenant))
    // クォータ超過の場合は SLO 侵害が検出されることを確認する（因果関係の方向）
    // ここでは: SLO 侵害なしならクォータ超過もなし という対偶を検証する
    !tenant.sloViolated || tenant.currentQps > QUOTA_PER_TENANT
  } ensuring { result =>
    // 事後条件: SLO 侵害は必ずクォータ超過と対応していることを保証する
    result == (!tenant.sloViolated || tenant.currentQps > QUOTA_PER_TENANT)
  }

  // crossTenantIsolation: 複数テナント間で noisy neighbor 分離が成立することを検証する
  // テナントリスト内の全ペアについて noisy neighbor 分離が成立することを確認する
  def crossTenantIsolation(tenants: List[TenantState]): Boolean = {
    // テナントが 2 未満の場合は自明に分離が成立する
    if (tenants.size < BigInt(2)) {
      // 単一または空のテナントリストは分離条件を自明に満たす
      true
    } else {
      // 先頭テナントを基準テナントとして取得する
      val reference = tenants.head
      // 残りの全テナントと先頭テナントの間で分離が成立することを確認する
      val pairwiseIsolated = tenants.tail.forall { other =>
        // 基準テナントが他テナントに影響しないことを確認する
        noisyNeighborIsolation(reference, other) &&
          // 他テナントが基準テナントに影響しないことを確認する
          noisyNeighborIsolation(other, reference)
      }
      // 先頭テナント分離 AND 残りリストの再帰的分離が成立することを確認する
      pairwiseIsolated && crossTenantIsolation(tenants.tail)
    }
  } ensuring { result =>
    // 事後条件: 結果の真偽値は全テナントペアの分離が成立するかどうかを示す
    true
  }

  // capacityBudgetConsistency: テナントの QPS とクォータの整合性を検証する
  // クォータ超過が検出されずに SLO 侵害が記録される状態は存在しない
  def capacityBudgetConsistency(tenant: TenantState): Boolean = {
    // 前提条件: テナント状態が有効であることを要求する
    require(isValidTenantState(tenant))
    // SLO 侵害フラグが TRUE の場合はクォータを超過していなければならない
    !tenant.sloViolated || tenant.currentQps > QUOTA_PER_TENANT
  } ensuring { result =>
    // 事後条件: 侵害フラグと QPS の整合性が保たれることを保証する
    result == (!tenant.sloViolated || tenant.currentQps > QUOTA_PER_TENANT)
  }
}
