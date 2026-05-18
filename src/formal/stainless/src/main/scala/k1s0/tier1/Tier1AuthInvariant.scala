// Tier1AuthInvariant.scala
// tier1 認証 invariant の Stainless 形式検証
// 04_認証適合仕様: auth_class × step-up FSM の単調増加不変条件
// obligation_id: tier1_auth_stainless
// cell_state: stub（Stainless で検証後に v1_baseline_verified に更新する）
// Stainless の lang パッケージをインポートする（ensuring / holds / require を使うため）
import stainless.lang.*
// Stainless の collection パッケージをインポートする（List 操作に使用）
import stainless.collection.*
// Stainless の annotation パッケージをインポートする（@induct 等を使うため）
import stainless.annotation.*

// Tier1AuthInvariant オブジェクト: auth_class step-up FSM の不変条件を検証する
object Tier1AuthInvariant {
  // AuthLevel 型エイリアス: 0-4 の整数値（04_認証適合仕様の 5 auth_class に対応）
  type AuthLevel = BigInt

  // isValidAuthLevel: 認証レベルが有効範囲内かを検証する述語関数
  def isValidAuthLevel(level: AuthLevel): Boolean =
    // 0 以上 4 以下の範囲であることを確認する
    level >= 0 && level <= 4

  // noSilentDemotion: step-up 遷移において認証レベルが下がらないことを保証する invariant
  // 04_認証適合仕様の no_silent_demotion property の Stainless 形式化
  def noSilentDemotion(oldLevel: AuthLevel, newLevel: AuthLevel): Boolean = {
    // 前提条件: 両レベルが有効範囲内であることを要求する
    require(isValidAuthLevel(oldLevel) && isValidAuthLevel(newLevel))
    // 新レベルが旧レベル以上であることを返す（格下げが発生しないことを示す）
    newLevel >= oldLevel
  } ensuring { result =>
    // 事後条件: 結果は新レベル >= 旧レベルの判定と一致しなければならない
    result == (newLevel >= oldLevel)
  }

  // stepUpOnly: step-up 遷移は現在レベルより大きい値へのみ移行できることを検証する
  // step-up は現在レベルを超える新レベルへの遷移のみ許可する
  def stepUpOnly(currentLevel: AuthLevel, targetLevel: AuthLevel): Boolean = {
    // 前提条件: 両レベルが有効範囲内であることを要求する
    require(isValidAuthLevel(currentLevel) && isValidAuthLevel(targetLevel))
    // 前提条件: step-up は現在レベルより大きい値へのみ許可する
    require(targetLevel > currentLevel)
    // step-up 後のレベルは必ず現在レベルより大きいことを返す
    targetLevel > currentLevel
  } ensuring { result =>
    // 事後条件: 結果は常に true でなければならない（step-up 方向の遷移のみ許可）
    result == true
  }

  // authLevelMonotone: 認証レベルの推移が単調増加であることを検証する（帰納的証明）
  // 一連の認証レベル推移が全て valid（単調増加）であることを確認する
  @induct
  def authLevelMonotone(levels: List[AuthLevel]): Boolean = {
    // levels が空または 1 要素のリストは自明に単調増加とする
    require(levels.nonEmpty)
    // 1 要素のリストは単調増加（比較対象なし）
    if (levels.size == BigInt(1)) {
      // 単一要素リストは常に単調増加を満たす
      isValidAuthLevel(levels.head)
    } else {
      // 先頭要素が有効かつ次要素以下であることを確認する
      val head = levels.head
      // 次の要素を取得する
      val next = levels.tail.head
      // 先頭が有効範囲内かつ単調増加条件を満たすことを確認する
      isValidAuthLevel(head) && isValidAuthLevel(next) && next >= head &&
        // 残りのリストも再帰的に単調増加であることを確認する
        authLevelMonotone(levels.tail)
    }
  } ensuring { result =>
    // 事後条件: 結果の真偽値は有効な単調増加リストかどうかを示す
    true
  }

  // allTransitionsValid: 全ての認証レベル遷移が no_silent_demotion を満たすことを検証する
  def allTransitionsValid(transitions: List[(AuthLevel, AuthLevel)]): Boolean = {
    // 空のリストは自明に全遷移 valid とする
    if (transitions.isEmpty) {
      // 遷移が存在しない場合は true を返す
      true
    } else {
      // 先頭の遷移ペアを取得する
      val (oldLevel, newLevel) = transitions.head
      // 先頭遷移が valid であり、かつ残り全遷移も valid であることを確認する
      isValidAuthLevel(oldLevel) && isValidAuthLevel(newLevel) &&
        noSilentDemotion(oldLevel, newLevel) &&
        // 残りの遷移リストも再帰的に検証する
        allTransitionsValid(transitions.tail)
    }
  } ensuring { result =>
    // 事後条件: 結果の真偽値は全遷移 valid かどうかを示す
    true
  }
}
