#!/usr/bin/env bash
# ティア間依存方向検証スクリプト (F-014)
# regions/ 配下のティア構造が正しい依存方向を維持しているかを検証する
#
# ティア階層:
#   system（基盤層）← service（中間層）← business（業務層）
#
# 許可される依存方向:
#   - system: 他ティアへの依存なし（自己参照のみ可）
#   - service: system への依存のみ許可（business への依存は禁止）
#   - business: system と service への依存を許可
#
# 使用方法:
#   scripts/ci/check-tier-deps.sh [regions-root]
#   regions-root: regions ディレクトリのパス（デフォルト: ./regions）
#
# 終了コード:
#   0: 違反なし
#   1: 依存方向違反を検出

set -euo pipefail

# ========================================
# 設定
# ========================================

# regions ディレクトリのルートパス（引数またはデフォルト値）
REGIONS_ROOT="${1:-./regions}"

# 違反カウンタ（0 = 違反なし）
VIOLATION_COUNT=0

# 色付き出力用の定義（ターミナル対応時のみ有効化）
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  NC='\033[0m'
else
  RED=''
  GREEN=''
  YELLOW=''
  NC=''
fi

# ========================================
# ユーティリティ関数
# ========================================

# 違反を報告し、カウンタをインクリメントする
report_violation() {
  local file="$1"
  local tier="$2"
  local forbidden_tier="$3"
  local detail="$4"
  echo -e "${RED}[VIOLATION]${NC} ${file}"
  echo -e "  ティア '${tier}' から禁止ティア '${forbidden_tier}' への依存を検出"
  echo -e "  詳細: ${detail}"
  echo ""
  VIOLATION_COUNT=$((VIOLATION_COUNT + 1))
}

# ファイルパスからティア名を抽出する
# REGIONS_ROOT 直下の最初のディレクトリ名がティア名となる
# 例: ./regions/system/library/go/auth/go.mod → system
extract_tier() {
  local filepath="$1"
  # REGIONS_ROOT のパスを除去し、最初のパスコンポーネントを取得する
  local relative_path="${filepath#"$REGIONS_ROOT"/}"
  echo "$relative_path" | cut -d'/' -f1
}

# ========================================
# Rust (Cargo.toml) の依存チェック
# ========================================

# Cargo.toml の path 依存からティア違反を検出する
# path = "..." 形式の相対パスを解析し、参照先のティアを特定する
check_rust_deps() {
  local cargo_file="$1"
  local current_tier
  current_tier=$(extract_tier "$cargo_file")

  # path 依存を抽出（path = "..." の値を取得）
  # Cargo.toml では依存クレートのパス指定が相対パスで記述される
  local path_deps
  path_deps=$(grep -oP 'path\s*=\s*"([^"]*)"' "$cargo_file" 2>/dev/null | sed 's/path\s*=\s*"//;s/"$//' || true)

  if [ -z "$path_deps" ]; then
    return
  fi

  # 各 path 依存について、参照先ティアを判定する
  while IFS= read -r dep_path; do
    # 相対パスを正規化して参照先ティアを特定する
    # パスに regions/XXX/ が含まれていればティア名を抽出
    local resolved_tier=""

    # パス内の各コンポーネントをチェックし、ティア名を探す
    if echo "$dep_path" | grep -q "system"; then
      resolved_tier="system"
    elif echo "$dep_path" | grep -q "service"; then
      resolved_tier="service"
    elif echo "$dep_path" | grep -q "business"; then
      resolved_tier="business"
    fi

    # 参照先ティアが特定できない場合はスキップ（外部依存の可能性）
    if [ -z "$resolved_tier" ]; then
      continue
    fi

    # ティア間の依存方向を検証する
    validate_tier_dependency "$cargo_file" "$current_tier" "$resolved_tier" "path = \"${dep_path}\""
  done <<< "$path_deps"
}

# ========================================
# Go (go.mod) の依存チェック
# ========================================

# go.mod の require ブロックからティア違反を検出する
# k1s0-platform のモジュールパスに含まれるティア名で判定する
check_go_deps() {
  local gomod_file="$1"
  local current_tier
  current_tier=$(extract_tier "$gomod_file")

  # k1s0-platform 関連の依存モジュールを抽出する
  # モジュールパスの命名規則: github.com/k1s0-platform/{tier}-{type}-{lang}-{name}
  local k1s0_deps
  k1s0_deps=$(grep -oP 'github\.com/k1s0-platform/[^\s]+' "$gomod_file" 2>/dev/null || true)

  if [ -z "$k1s0_deps" ]; then
    return
  fi

  # 各依存モジュールのティアを判定する
  while IFS= read -r dep_module; do
    local resolved_tier=""

    # モジュール名からティアを特定する
    # 命名規則: {tier}-{component-type}-{lang}-{name}
    if echo "$dep_module" | grep -qP 'k1s0-platform/system-'; then
      resolved_tier="system"
    elif echo "$dep_module" | grep -qP 'k1s0-platform/service-'; then
      resolved_tier="service"
    elif echo "$dep_module" | grep -qP 'k1s0-platform/business-'; then
      resolved_tier="business"
    fi

    # ティアが特定できない場合はスキップ
    if [ -z "$resolved_tier" ]; then
      continue
    fi

    # ティア間の依存方向を検証する
    validate_tier_dependency "$gomod_file" "$current_tier" "$resolved_tier" "module: ${dep_module}"
  done <<< "$k1s0_deps"
}

# ========================================
# TypeScript (package.json) の依存チェック
# ========================================

# package.json の dependencies / devDependencies からティア違反を検出する
# @k1s0/ スコープのパッケージ名やファイルパス参照でティアを判定する
check_ts_deps() {
  local package_file="$1"
  local current_tier
  current_tier=$(extract_tier "$package_file")

  # dependencies と devDependencies からパッケージ名・パス参照を抽出する
  # file: プロトコルのローカル参照と @k1s0/ スコープのパッケージを対象とする
  local dep_entries
  dep_entries=$(grep -oP '"(@k1s0/[^"]+|file:[^"]+)"' "$package_file" 2>/dev/null | sed 's/"//g' || true)

  if [ -z "$dep_entries" ]; then
    return
  fi

  # 各依存パッケージのティアを判定する
  while IFS= read -r dep_entry; do
    local resolved_tier=""

    # file: プロトコルの場合、パス内のティア名で判定する
    if echo "$dep_entry" | grep -q "^file:"; then
      if echo "$dep_entry" | grep -q "system"; then
        resolved_tier="system"
      elif echo "$dep_entry" | grep -q "service"; then
        resolved_tier="service"
      elif echo "$dep_entry" | grep -q "business"; then
        resolved_tier="business"
      fi
    else
      # @k1s0/ スコープのパッケージ名からティアを判定する
      # 命名規則: @k1s0/{tier}-{name}
      if echo "$dep_entry" | grep -qP '@k1s0/system-'; then
        resolved_tier="system"
      elif echo "$dep_entry" | grep -qP '@k1s0/service-'; then
        resolved_tier="service"
      elif echo "$dep_entry" | grep -qP '@k1s0/business-'; then
        resolved_tier="business"
      fi
    fi

    # ティアが特定できない場合はスキップ
    if [ -z "$resolved_tier" ]; then
      continue
    fi

    # ティア間の依存方向を検証する
    validate_tier_dependency "$package_file" "$current_tier" "$resolved_tier" "dep: ${dep_entry}"
  done <<< "$dep_entries"
}

# ========================================
# ティア依存方向の検証ロジック
# ========================================

# 依存方向ルールに基づいて違反を判定する
# 許可ルール:
#   system → (自分自身のみ)
#   service → system のみ
#   business → system, service のみ
validate_tier_dependency() {
  local file="$1"
  local from_tier="$2"
  local to_tier="$3"
  local detail="$4"

  # 同一ティア内の参照は常に許可
  if [ "$from_tier" = "$to_tier" ]; then
    return
  fi

  case "$from_tier" in
    system)
      # system ティアは他ティアに依存してはならない
      report_violation "$file" "$from_tier" "$to_tier" "$detail"
      ;;
    service)
      # service ティアは system のみ参照可能、business は禁止
      if [ "$to_tier" = "business" ]; then
        report_violation "$file" "$from_tier" "$to_tier" "$detail"
      fi
      # system への依存は許可（何もしない）
      ;;
    business)
      # business ティアは system と service を参照可能
      # 現状ではここに到達する違反パターンはないが、
      # 将来のティア追加に備えてガード条件を残す
      if [ "$to_tier" != "system" ] && [ "$to_tier" != "service" ]; then
        report_violation "$file" "$from_tier" "$to_tier" "$detail"
      fi
      ;;
    *)
      # 未知のティア名は警告を出力する
      echo -e "${YELLOW}[WARN]${NC} 未知のティア '${from_tier}' を検出: ${file}"
      ;;
  esac
}

# ========================================
# メイン処理
# ========================================

echo "======================================"
echo " ティア間依存方向検証"
echo " Tier Dependency Direction Check"
echo "======================================"
echo ""
echo "対象ディレクトリ: ${REGIONS_ROOT}"
echo "検証ルール: system ← service ← business（一方向のみ）"
echo ""

# regions ディレクトリの存在確認
if [ ! -d "$REGIONS_ROOT" ]; then
  echo -e "${RED}[ERROR]${NC} regions ディレクトリが見つかりません: ${REGIONS_ROOT}"
  exit 1
fi

# --- Rust Cargo.toml の検証 ---
echo -e "${YELLOW}[CHECK]${NC} Rust (Cargo.toml) の依存を検証中..."
# regions 配下の全 Cargo.toml を検索し、各ファイルをチェックする
rust_count=0
while IFS= read -r -d '' cargo_file; do
  check_rust_deps "$cargo_file"
  rust_count=$((rust_count + 1))
done < <(find "$REGIONS_ROOT" -name "Cargo.toml" -not -path "*/target/*" -print0 2>/dev/null)
echo "  検査ファイル数: ${rust_count}"
echo ""

# --- Go go.mod の検証 ---
echo -e "${YELLOW}[CHECK]${NC} Go (go.mod) の依存を検証中..."
# regions 配下の全 go.mod を検索し、各ファイルをチェックする
go_count=0
while IFS= read -r -d '' gomod_file; do
  check_go_deps "$gomod_file"
  go_count=$((go_count + 1))
done < <(find "$REGIONS_ROOT" -name "go.mod" -print0 2>/dev/null)
echo "  検査ファイル数: ${go_count}"
echo ""

# --- TypeScript package.json の検証 ---
echo -e "${YELLOW}[CHECK]${NC} TypeScript (package.json) の依存を検証中..."
# regions 配下の全 package.json を検索し、各ファイルをチェックする
# node_modules 配下は除外する
ts_count=0
while IFS= read -r -d '' package_file; do
  check_ts_deps "$package_file"
  ts_count=$((ts_count + 1))
done < <(find "$REGIONS_ROOT" -name "package.json" -not -path "*/node_modules/*" -print0 2>/dev/null)
echo "  検査ファイル数: ${ts_count}"
echo ""

# ========================================
# 結果サマリー
# ========================================

echo "======================================"
echo " 検証結果サマリー"
echo "======================================"
echo "  Rust (Cargo.toml):      ${rust_count} ファイル"
echo "  Go (go.mod):            ${go_count} ファイル"
echo "  TypeScript (package.json): ${ts_count} ファイル"
echo "  合計検査数:             $((rust_count + go_count + ts_count)) ファイル"
echo ""

if [ "$VIOLATION_COUNT" -gt 0 ]; then
  echo -e "${RED}[FAIL]${NC} ${VIOLATION_COUNT} 件のティア依存方向違反を検出しました"
  echo ""
  echo "修正方法:"
  echo "  - system ティアのコードが service/business を参照している場合 → 参照を削除"
  echo "  - service ティアのコードが business を参照している場合 → 共通ロジックを system に移動"
  echo "  - 詳細は docs/architecture/overview/コンセプト.md を参照してください"
  exit 1
else
  echo -e "${GREEN}[PASS]${NC} ティア依存方向違反はありません"
  exit 0
fi
