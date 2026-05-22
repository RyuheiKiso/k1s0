-- lakefile.lean — k1s0 formal Lean 4 プロジェクト定義
-- Mathlib 不要の最小構成。HLC 単調性 lemma を起点とする。

import Lake
open Lake DSL

-- k1s0-formal パッケージ定義
package «k1s0Formal» where
  -- デフォルト設定のみ（追加 PackageConfig 不要）

-- K1s0Formal ライブラリターゲットを定義する
@[default_target]
lean_lib «K1s0Formal» where
  -- ルートモジュールを Tier1.HLC.Monotone に設定する
  roots := #[`Tier1.HLC.Monotone]
