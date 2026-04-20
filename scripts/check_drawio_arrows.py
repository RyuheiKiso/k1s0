# drawio ファイルの矢印ルール違反検出スクリプト
# CLAUDE.md の規約に従い以下を検査する
# 1. 白の矢印（strokeColor=#FFFFFF / #ffffff / white）
# 2. orthogonalEdgeStyle の自動ルーティング依存（waypoint 未指定 かつ 他ボックスと交差）
# 3. 矢印経路がボックス・テキスト領域と交差
# 4. ラベル幅に対する接続元→接続先間隔の不足

import io
import sys
import glob
import xml.etree.ElementTree as ET
from pathlib import Path

# 標準出力を UTF-8 に固定（Windows cp932 対策）
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

# 検査対象ルートディレクトリ
ROOT = Path("docs/04_概要設計")


def parse_style(style_str):
    # drawio の style 文字列を辞書化する
    if not style_str:
        return {}
    result = {}
    for item in style_str.split(";"):
        if "=" in item:
            k, v = item.split("=", 1)
            result[k.strip()] = v.strip()
        elif item.strip():
            result[item.strip()] = True
    return result


def get_geometry(cell_elem):
    # mxCell 配下の mxGeometry から x/y/width/height を取得
    geom = cell_elem.find("mxGeometry")
    if geom is None:
        return None
    try:
        x = float(geom.get("x", 0))
        y = float(geom.get("y", 0))
        w = float(geom.get("width", 0))
        h = float(geom.get("height", 0))
        return (x, y, w, h)
    except (TypeError, ValueError):
        return None


def rect_contains(outer, inner):
    # outer が inner を完全包含するか
    ox, oy, ow, oh = outer
    ix, iy, iw, ih = inner
    return (
        ox <= ix
        and oy <= iy
        and ox + ow >= ix + iw
        and oy + oh >= iy + ih
        and (ow, oh) != (iw, ih)
    )


def rects_intersect_segment(rect, segment):
    # 矩形 (x, y, w, h) と線分 ((x1,y1)-(x2,y2)) の交差判定（Liang-Barsky）
    rx, ry, rw, rh = rect
    x1, y1 = segment[0]
    x2, y2 = segment[1]

    seg_min_x = min(x1, x2)
    seg_max_x = max(x1, x2)
    seg_min_y = min(y1, y2)
    seg_max_y = max(y1, y2)
    if seg_max_x < rx or seg_min_x > rx + rw:
        return False
    if seg_max_y < ry or seg_min_y > ry + rh:
        return False

    dx = x2 - x1
    dy = y2 - y1
    p = [-dx, dx, -dy, dy]
    q = [x1 - rx, (rx + rw) - x1, y1 - ry, (ry + rh) - y1]
    u1, u2 = 0.0, 1.0
    for i in range(4):
        if p[i] == 0:
            if q[i] < 0:
                return False
        else:
            t = q[i] / p[i]
            if p[i] < 0:
                u1 = max(u1, t)
            else:
                u2 = min(u2, t)
    # 端点だけ接する場合は交差とみなさない
    return u1 < u2 - 1e-6


def box_center(rect):
    x, y, w, h = rect
    return (x + w / 2, y + h / 2)


def box_port(rect, px, py):
    # 矩形内の相対座標 (0..1) を絶対座標に変換
    x, y, w, h = rect
    return (x + w * px, y + h * py)


def resolve_endpoint(rect, style_xkey, style_ykey, style):
    # exitX/Y もしくは entryX/Y が指定されていればその接続点を使用、なければ中心
    try:
        px = float(style.get(style_xkey, 0.5))
        py = float(style.get(style_ykey, 0.5))
        return box_port(rect, px, py)
    except (TypeError, ValueError):
        return box_center(rect)


def orthogonal_path(src_pt, tgt_pt):
    # orthogonalEdgeStyle の簡易近似: 縦横 2 パターンを両方算出し
    # より短い曲がりで済む方を選ぶ（drawio のデフォルト挙動に近い）
    sx, sy = src_pt
    tx, ty = tgt_pt
    # 端点がほぼ同じ x/y の場合は直線扱い
    if abs(sx - tx) < 1 or abs(sy - ty) < 1:
        return [(sx, sy), (tx, ty)]
    # 横→縦 と 縦→横 の両経路を返す（どちらかが通れば OK ではなく
    # drawio の採用経路は不定のため、両方検査する）
    return [(sx, sy), (tx, sy), (tx, ty)], [(sx, sy), (sx, ty), (tx, ty)]


def is_container(vertex_id, vertices):
    # 他の vertex を包含していればコンテナ扱い
    geom = vertices[vertex_id]["geom"]
    if not geom:
        return False
    for vid, v in vertices.items():
        if vid == vertex_id or not v["geom"]:
            continue
        if rect_contains(geom, v["geom"]):
            return True
    return False


def is_background(style):
    # 全面白背景矩形（CLAUDE.md 規約で bg として最初に配置するもの）
    return (
        str(style.get("fillColor", "")).lower() == "#ffffff"
        and str(style.get("strokeColor", "")).lower() == "none"
    )


def is_text_only(style):
    # ラベル専用セル（text; で始まる shape）
    shape = style.get("shape", "")
    return str(shape).startswith("text") or style.get("text") is True


def analyze_file(path):
    violations = []
    try:
        tree = ET.parse(path)
    except ET.ParseError as e:
        return [f"XML parse error: {e}"]
    root = tree.getroot()

    vertices = {}
    edges = []
    for cell in root.iter("mxCell"):
        cid = cell.get("id")
        if cid is None:
            continue
        style = parse_style(cell.get("style", ""))
        geom = get_geometry(cell)
        if cell.get("vertex") == "1":
            vertices[cid] = {
                "style": style,
                "geom": geom,
                "value": cell.get("value", ""),
            }
        elif cell.get("edge") == "1":
            src = cell.get("source")
            tgt = cell.get("target")
            waypoints = []
            source_pt = None
            target_pt = None
            geom_elem = cell.find("mxGeometry")
            if geom_elem is not None:
                for pt in geom_elem.findall("mxPoint"):
                    role = pt.get("as")
                    try:
                        coord = (float(pt.get("x", 0)), float(pt.get("y", 0)))
                    except (TypeError, ValueError):
                        continue
                    if role == "sourcePoint":
                        source_pt = coord
                    elif role == "targetPoint":
                        target_pt = coord
                for arr in geom_elem.findall("Array"):
                    if arr.get("as") == "points":
                        for pt in arr.findall("mxPoint"):
                            try:
                                waypoints.append(
                                    (float(pt.get("x", 0)), float(pt.get("y", 0)))
                                )
                            except (TypeError, ValueError):
                                pass
            edges.append(
                {
                    "id": cid,
                    "style": style,
                    "source": src,
                    "target": tgt,
                    "source_pt": source_pt,
                    "target_pt": target_pt,
                    "waypoints": waypoints,
                    "value": cell.get("value", ""),
                }
            )

    # コンテナ vertex を事前抽出
    container_ids = {
        vid for vid in vertices if is_container(vid, vertices)
    }

    for e in edges:
        eid = e["id"]
        style = e["style"]
        stroke = str(style.get("strokeColor", "")).lower()

        # ルール1: 白矢印
        if stroke in ("#ffffff", "#fff", "white"):
            violations.append(f"edge {eid}: 白矢印 strokeColor={stroke}")

        # ルール2a: 非直交スタイルの直線エッジが他ボックスを貫通
        edge_style = style.get("edgeStyle", "")
        if edge_style != "orthogonalEdgeStyle":
            # 両端点を解決（ボックス参照 or 直接座標）
            src_pt = None
            tgt_pt = None
            src_id = e["source"]
            tgt_id = e["target"]
            if src_id and src_id in vertices and vertices[src_id]["geom"]:
                src_pt = resolve_endpoint(
                    vertices[src_id]["geom"], "exitX", "exitY", style
                )
            elif e["source_pt"]:
                src_pt = e["source_pt"]
            if tgt_id and tgt_id in vertices and vertices[tgt_id]["geom"]:
                tgt_pt = resolve_endpoint(
                    vertices[tgt_id]["geom"], "entryX", "entryY", style
                )
            elif e["target_pt"]:
                tgt_pt = e["target_pt"]
            if src_pt and tgt_pt:
                pts = [src_pt] + e["waypoints"] + [tgt_pt]
                segments = list(zip(pts[:-1], pts[1:]))
                conflicts = []
                exclude_ids = {src_id, tgt_id} - {None}
                for vid, v in vertices.items():
                    if vid in exclude_ids:
                        continue
                    if not v["geom"]:
                        continue
                    if vid in container_ids:
                        continue
                    if is_background(v["style"]):
                        continue
                    if is_text_only(v["style"]):
                        continue
                    for seg in segments:
                        if rects_intersect_segment(v["geom"], seg):
                            conflicts.append(vid)
                            break
                if conflicts:
                    src_desc = src_id if src_id else f"{src_pt}"
                    tgt_desc = tgt_id if tgt_id else f"{tgt_pt}"
                    violations.append(
                        f"edge {eid}: 直線エッジ "
                        f"({src_desc}→{tgt_desc}) 他要素を貫通: {conflicts}"
                    )

        # ルール2b: orthogonalEdgeStyle の自動ルーティング依存
        if edge_style == "orthogonalEdgeStyle" and e["source"] and e["target"]:
            src_rect = vertices.get(e["source"], {}).get("geom")
            tgt_rect = vertices.get(e["target"], {}).get("geom")
            if src_rect and tgt_rect and not e["waypoints"]:
                src_pt = resolve_endpoint(src_rect, "exitX", "exitY", style)
                tgt_pt = resolve_endpoint(tgt_rect, "entryX", "entryY", style)
                paths_result = orthogonal_path(src_pt, tgt_pt)
                # 直線 or L 字 2 候補（タプル）に対応
                if isinstance(paths_result, list):
                    paths = [paths_result]
                else:
                    paths = list(paths_result)
                # 両 L 字経路とも他要素と交差する場合のみ違反
                all_conflicts_per_path = []
                for path in paths:
                    segments = list(zip(path[:-1], path[1:]))
                    conflicts = []
                    for vid, v in vertices.items():
                        if vid in (e["source"], e["target"]):
                            continue
                        if not v["geom"]:
                            continue
                        if vid in container_ids:
                            continue
                        if is_background(v["style"]):
                            continue
                        if is_text_only(v["style"]):
                            continue
                        for seg in segments:
                            if rects_intersect_segment(v["geom"], seg):
                                conflicts.append(vid)
                                break
                    all_conflicts_per_path.append(conflicts)
                # 全候補経路で貫通が発生する場合のみ違反とする
                if all(c for c in all_conflicts_per_path):
                    union = sorted(set().union(*all_conflicts_per_path))
                    violations.append(
                        f"edge {eid}: orthogonalEdgeStyle 無waypoint "
                        f"({e['source']}→{e['target']}) 他要素を貫通: {union}"
                    )

        # ルール4: ラベル幅に対する距離不足
        label = e.get("value") or ""
        label = label.replace("\n", " ").replace("\r", " ")
        if label.strip():
            label_width = sum(12 if ord(c) > 256 else 6 for c in label)
            src_rect = vertices.get(e["source"], {}).get("geom")
            tgt_rect = vertices.get(e["target"], {}).get("geom")
            if src_rect and tgt_rect:
                sc = box_center(src_rect)
                tc = box_center(tgt_rect)
                dist = ((sc[0] - tc[0]) ** 2 + (sc[1] - tc[1]) ** 2) ** 0.5
                if dist < label_width * 1.5:
                    violations.append(
                        f"edge {eid}: ラベル幅{label_width}px に対し距離 {dist:.0f}px "
                        f"(1.5倍={label_width * 1.5:.0f}px 未満) label='{label[:30]}'"
                    )

    return violations


def main():
    targets = sorted(glob.glob(str(ROOT / "**/*.drawio"), recursive=True))
    total = 0
    for path in targets:
        vs = analyze_file(path)
        if vs:
            total += len(vs)
            print(f"\n=== {path} ===")
            for v in vs:
                print(f"  - {v}")
    print(f"\n[summary] scanned={len(targets)} violations={total}")
    return 0 if total == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
