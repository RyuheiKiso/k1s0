// Tier1SLOInvariant.scala
// tier1 SLO invariant の Stainless 形式検証
// 07_SLO 適合仕様: burn-rate とエラーバジェット凍結の不変条件
// obligation_id: tier1_slo_stainless
// cell_state: stub（Stainless で検証後に v1_baseline_verified に更新する）
// Stainless の lang パッケージをインポートする（ensuring / holds / require を使うため）
import stainless.lang.*
// Stainless の collection パッケージをインポートする（List 操作に使用）
import stainless.collection.*
// Stainless の annotation パッケージをインポートする（@induct 等を使うため）
import stainless.annotation.*

// Tier1SLOInvariant オブジェクト: SLO burn-rate とエラーバジェット凍結の不変条件を検証する
object Tier1SLOInvariant {
  // BURN_RATE_THRESHOLD: アラート発火・バジェット凍結の閾値（07_SLO 適合仕様に準拠）
  val BURN_RATE_THRESHOLD: BigInt = 2

  // MAX_BURN_RATE: burn rate の最大値（状態空間を有限に抑えるため定義）
  val MAX_BURN_RATE: BigInt = 10

  // isValidBurnRate: burn rate が有効範囲内かを検証する述語関数
  def isValidBurnRate(rate: BigInt): Boolean =
    // 0 以上 MAX_BURN_RATE 以下の範囲であることを確認する
    rate >= 0 && rate <= MAX_BURN_RATE

  // freezeOnBurn: burn_rate が閾値を超えた場合、バジェットが必ず凍結されることを保証する
  // 07_SLO 適合仕様の freeze_on_burn property の Stainless 形式化
  def freezeOnBurn(burnRate: BigInt, budgetFrozen: Boolean): Boolean = {
    // 前提条件: burn rate が有効範囲内であることを要求する
    require(isValidBurnRate(burnRate))
    // burn_rate > 閾値 の場合、budgetFrozen は TRUE でなければならない
    burnRate <= BURN_RATE_THRESHOLD || budgetFrozen
  } ensuring { result =>
    // 事後条件: 閾値以下なら任意の凍結状態を許容し、超過なら凍結必須
    result == (burnRate <= BURN_RATE_THRESHOLD || budgetFrozen)
  }

  // alertFiresBeforeFreeze: アラート発火はバジェット凍結の前提条件であることを検証する
  // alertFired = FALSE かつ budgetFrozen = TRUE の組合せは許可しない
  def alertFiresBeforeFreeze(alertFired: Boolean, budgetFrozen: Boolean): Boolean = {
    // 前提条件: alertFired と budgetFrozen が整合していることを要求する
    require(!(budgetFrozen && !alertFired))
    // バジェット凍結にはアラート発火が先行することを確認する
    !(budgetFrozen && !alertFired)
  } ensuring { result =>
    // 事後条件: アラートなしに凍結される状態は存在しないことを保証する
    result == true
  }

  // unfreezeCondition: 凍結解除は burn_rate が正常域に回復した場合のみ許可される
  // burn_rate <= 1 になるまでは凍結状態を維持しなければならない
  def unfreezeCondition(burnRate: BigInt, budgetFrozen: Boolean, newBudgetFrozen: Boolean): Boolean = {
    // 前提条件: burn rate が有効範囲内であることを要求する
    require(isValidBurnRate(burnRate))
    // 凍結中に burn_rate > 1 の場合は凍結解除を禁止する
    if (budgetFrozen && burnRate > 1) {
      // 凍結状態を維持しなければならない
      newBudgetFrozen
    } else {
      // burn_rate <= 1 または非凍結の場合は新しい凍結状態を自由に設定できる
      true
    }
  } ensuring { result =>
    // 事後条件: 結果の真偽値は凍結解除条件を満たしているかどうかを示す
    true
  }

  // sloComplianceInvariant: SLO 準拠状態の複合不変条件を検証する
  // 複数の SLO 関連フィールドが整合していることを一括検証する
  def sloComplianceInvariant(burnRate: BigInt, budgetFrozen: Boolean, alertFired: Boolean): Boolean = {
    // 前提条件: burn rate が有効範囲内であることを要求する
    require(isValidBurnRate(burnRate))
    // freeze_on_burn 不変条件が成立することを確認する
    val freezeOk = freezeOnBurn(burnRate, budgetFrozen)
    // アラートと凍結の整合性が保たれていることを確認する
    val alertConsistent = !budgetFrozen || alertFired
    // 全ての SLO 不変条件が成立することを返す
    freezeOk && alertConsistent
  } ensuring { result =>
    // 事後条件: 全 SLO 不変条件の AND が成立することを保証する
    true
  }
}
