"""tools/impl_substance_check

src/ 配下の実装軸ごとに LOC・TODO マーカー・facade サイズを集計して
src/_meta/lock/impl_substance.lock.yaml を生成するツール。

Usage:
    python -m tools.impl_substance_check \
        [--repo-root .] \
        [--out src/_meta/lock/impl_substance.lock.yaml] \
        [--facade-paths src/_meta/lint/facade_paths.yaml]
"""
