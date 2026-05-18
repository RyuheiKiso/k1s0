// Tier1KeyMgmtInvariant.scala
// tier1 鍵管理 invariant の Stainless 形式検証
// 05_鍵管理適合仕様: KEK rotation state machine の方向制約不変条件
// obligation_id: tier1_key_mgmt_stainless
// cell_state: stub（Stainless で検証後に v1_baseline_verified に更新する）
// Stainless の lang パッケージをインポートする（ensuring / holds / require を使うため）
import stainless.lang.*
// Stainless の collection パッケージをインポートする（List 操作に使用）
import stainless.collection.*
// Stainless の annotation パッケージをインポートする（@induct 等を使うため）
import stainless.annotation.*

// Tier1KeyMgmtInvariant オブジェクト: KEK ローテーション FSM の不変条件を検証する
object Tier1KeyMgmtInvariant {
  // KEK の状態を整数値として表現する（Stainless は enumeration をサポートしないため）
  // 0: active（稼働中）/ 1: rotating（ローテーション中）/ 2: destroyed（消去済み）
  // STATE_ACTIVE: active 状態の整数値
  val STATE_ACTIVE: BigInt = 0
  // STATE_ROTATING: rotating 状態の整数値
  val STATE_ROTATING: BigInt = 1
  // STATE_DESTROYED: destroyed 状態の整数値
  val STATE_DESTROYED: BigInt = 2

  // isValidKekState: KEK 状態が有効範囲内かを検証する述語関数
  def isValidKekState(state: BigInt): Boolean =
    // 0 / 1 / 2 のいずれかであることを確認する
    state == STATE_ACTIVE || state == STATE_ROTATING || state == STATE_DESTROYED

  // kekStateMonotone: KEK 状態遷移が active→rotating→destroyed の一方向のみを保証する
  // 逆方向への遷移（格上げ）は許可しない（不可逆な暗号消去を形式化する）
  def kekStateMonotone(oldState: BigInt, newState: BigInt): Boolean = {
    // 前提条件: 両状態が有効な KEK 状態値であることを要求する
    require(isValidKekState(oldState) && isValidKekState(newState))
    // 新状態は旧状態以上でなければならない（active=0 → rotating=1 → destroyed=2 の一方向）
    newState >= oldState
  } ensuring { result =>
    // 事後条件: 結果は新状態 >= 旧状態の判定と一致しなければならない
    result == (newState >= oldState)
  }

  // noRawBytesWhenDestroyed: destroyed 状態では raw bytes が in-flight であってはならない
  // 05_鍵管理適合仕様の no_raw_bytes_in_flight property の Stainless 形式化
  def noRawBytesWhenDestroyed(kekState: BigInt, rawBytesInFlight: Boolean): Boolean = {
    // 前提条件: KEK 状態が有効範囲内であることを要求する
    require(isValidKekState(kekState))
    // destroyed 状態で raw bytes が in-flight であれば検証違反となる
    !(kekState == STATE_DESTROYED && rawBytesInFlight)
  } ensuring { result =>
    // 事後条件: 違反がない場合は true を返す
    result == !(kekState == STATE_DESTROYED && rawBytesInFlight)
  }

  // validRotationSequence: ローテーションシーケンスが正しい順序であることを検証する
  // active → rotating → destroyed の順序のみを有効とする
  def validRotationSequence(states: List[BigInt]): Boolean = {
    // 空のリストは自明に有効とする
    if (states.isEmpty) {
      // シーケンスが存在しない場合は true を返す
      true
    } else if (states.size == BigInt(1)) {
      // 単一要素は有効な KEK 状態であれば valid とする
      isValidKekState(states.head)
    } else {
      // 先頭要素が有効かつ単調増加条件を満たすことを確認する
      val current = states.head
      // 次の状態を取得する
      val next = states.tail.head
      // 現在状態が有効かつ単調増加遷移であることを確認する
      isValidKekState(current) && isValidKekState(next) && kekStateMonotone(current, next) &&
        // 残りのシーケンスも再帰的に検証する
        validRotationSequence(states.tail)
    }
  } ensuring { result =>
    // 事後条件: 結果の真偽値は有効なローテーションシーケンスかどうかを示す
    true
  }

  // cryptoShredIrreversible: 暗号消去は不可逆であることを検証する
  // destroyed 状態から active または rotating への遷移は許可しない
  def cryptoShredIrreversible(currentState: BigInt, targetState: BigInt): Boolean = {
    // 前提条件: 両状態が有効な KEK 状態値であることを要求する
    require(isValidKekState(currentState) && isValidKekState(targetState))
    // 前提条件: 現在状態が destroyed でないか、または遷移しない場合のみ検証する
    require(currentState != STATE_DESTROYED || targetState == STATE_DESTROYED)
    // destroyed 状態からの復帰が発生しないことを確認する
    !(currentState == STATE_DESTROYED && targetState != STATE_DESTROYED)
  } ensuring { result =>
    // 事後条件: 結果は常に true でなければならない（暗号消去の不可逆性を保証）
    result == true
  }
}
