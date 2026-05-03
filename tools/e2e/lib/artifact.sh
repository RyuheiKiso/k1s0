#!/usr/bin/env bash
#
# tools/e2e/lib/artifact.sh — owner / user 共通 artifact 集約 helper
#
# 設計正典:
#   ADR-TEST-011（release tag ゲート: artifact sha256 検証）
#   docs/05_実装/30_CI_CD設計/35_e2e_test_design/40_release_tag_gate/02_artifact_保管.md
#
# 直接実行は不可。owner/up.sh / user/up.sh / Makefile target から `source` で読み込む。
# 前提: tools/e2e/lib/common.sh が source 済
#
# 提供関数:
#   e2e_collect_cluster_info — kubectl version / nodes / get all -A をテキスト化
#   e2e_collect_dmesg        — host の dmesg を artifact 化（owner 用、OOM 監視）
#   e2e_archive_artifacts    — artifact ディレクトリを tar.zst 化 + sha256 計算

if [[ -n "${E2E_LIB_ARTIFACT_LOADED:-}" ]]; then
    return 0
fi
readonly E2E_LIB_ARTIFACT_LOADED=1

# kubectl version / nodes / get all -A / Cilium / Longhorn / MetalLB status を 1 ファイルにまとめる
# 出力先: ${artifact_dir}/cluster-info.txt
e2e_collect_cluster_info() {
    # 引数 1: artifact ディレクトリ
    local artifact_dir="$1"
    mkdir -p "${artifact_dir}"
    local out_file="${artifact_dir}/cluster-info.txt"

    {
        echo "# k1s0 e2e cluster-info — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo ""
        echo "## kubectl version"
        echo '```'
        kubectl version 2>&1 || true
        echo '```'
        echo ""
        echo "## kubectl get nodes -o wide"
        echo '```'
        kubectl get nodes -o wide 2>&1 || true
        echo '```'
        echo ""
        echo "## kubectl get all -A（要約、最大 100 行）"
        echo '```'
        kubectl get all -A 2>&1 | head -100 || true
        echo '```'
        echo ""
        echo "## CNI status (Cilium または Calico)"
        echo '```'
        kubectl get pods -n cilium-system 2>/dev/null || \
            kubectl get pods -n calico-system 2>/dev/null || \
            echo "(CNI namespace 未検出)"
        echo '```'
        echo ""
        echo "## CSI status (Longhorn、owner suite のみ)"
        echo '```'
        kubectl get pods -n longhorn-system 2>/dev/null || \
            echo "(longhorn-system 未配置 = user suite)"
        echo '```'
        echo ""
        echo "## LB status (MetalLB、owner suite のみ)"
        echo '```'
        kubectl get pods -n metallb-system 2>/dev/null || \
            echo "(metallb-system 未配置 = user suite)"
        echo '```'
        echo ""
        echo "## StorageClass 一覧"
        echo '```'
        kubectl get storageclass 2>&1 || true
        echo '```'
    } > "${out_file}"

    e2e_log "cluster-info 出力: ${out_file}"
}

# host の dmesg を artifact 化（owner 専用、OOM / kernel error 監視のため）
# 直近 10000 行を tail で取得し、tar に同梱する想定の txt として出力
e2e_collect_dmesg() {
    # 引数 1: artifact ディレクトリ
    local artifact_dir="$1"
    mkdir -p "${artifact_dir}"
    local out_file="${artifact_dir}/dmesg.txt"

    # sudo dmesg は権限が要る、失敗しても fatal にしない（CI / 一部環境では取得不能）
    if sudo -n dmesg --time-format=iso 2>/dev/null | tail -10000 > "${out_file}"; then
        e2e_log "dmesg 出力: ${out_file}"
    else
        e2e_warn "dmesg 取得失敗（sudo 不可 / 権限不足）、空ファイルを生成"
        : > "${out_file}"
    fi
}

# artifact ディレクトリを tar.zst 化、sha256sum を計算して所定ファイルに記録
# 戻り値: 標準出力に sha256（64 文字 HEX）
e2e_archive_artifacts() {
    # 引数 1: artifact ディレクトリ（tar 対象）/ 引数 2: 出力 tar.zst パス
    local artifact_dir="$1"
    local tar_path="$2"

    if [[ ! -d "${artifact_dir}" ]]; then
        e2e_fail "artifact ディレクトリ不在: ${artifact_dir}"
    fi

    # zstd が存在すれば zstd -19、不在なら gzip にフォールバック（cut.sh と整合）
    if command -v zstd >/dev/null 2>&1; then
        e2e_log "artifact 集約: ${tar_path} (zstd -19)"
        tar -cf - -C "$(dirname "${artifact_dir}")" "$(basename "${artifact_dir}")" \
            | zstd -19 -o "${tar_path}"
    else
        # zstd 不在時は .tar.gz にフォールバック（パスの拡張子は呼び出し側で調整）
        local fallback_tar="${tar_path%.tar.zst}.tar.gz"
        e2e_warn "zstd 不在、gzip にフォールバック: ${fallback_tar}"
        tar -czf "${fallback_tar}" -C "$(dirname "${artifact_dir}")" "$(basename "${artifact_dir}")"
        tar_path="${fallback_tar}"
    fi

    # sha256sum を計算して標準出力に出す（呼び出し側が capture）
    e2e_sha256 "${tar_path}"
}
