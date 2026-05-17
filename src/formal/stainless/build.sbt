// Stainless 0.9.8 の build.sbt
// プロジェクト名を k1s0-formal に設定する
name := "k1s0-formal"
// バージョンを 0.1.0 に設定する
version := "0.1.0"
// Scala 3.3.3 を使用する（Stainless 0.9.8 が対応する LTS バージョン）
scalaVersion := "3.3.3"

// 依存ライブラリ: Stainless core を追加する
libraryDependencies ++= Seq(
  // Stainless core ライブラリを追加する
  "ch.epfl.lara" %% "stainless-core" % "0.9.8"
)

// Stainless verification の有効化設定
stainlessEnabled := true
// 検証対象クラスを AtomicThreeTableWrite のみに限定する
stainlessVerifyOnly := Seq("k1s0.data.AtomicThreeTableWrite")
