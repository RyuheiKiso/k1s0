# TLA+ Proof Specifications

このディレクトリには k1s0 プロジェクトの形式検証に使用する TLA+ スペックファイルを格納する。

## ディレクトリ構成

```
tla/
├── README.md                          # 本ファイル
├── tier1_bidi_handshake.tla          # tier1 bidi handshake safety の TLA+ スペック
├── tier1_bidi_handshake.cfg          # TLC 検証設定
├── tier1_bidi_handshake_apalache.json # Apalache 検証設定
└── lock/
    └── tla_apalache_pin.lock.yaml    # ツールバージョン固定ファイル
```

## 検証ツール

| ツール | 用途 | バージョン |
|--------|------|-----------|
| TLC | TLA+ モデル検査 | 1.8.0 |
| Apalache | TLA+ 記号モデル検査（大規模状態空間対応）| 0.45.4 |

## 実行方法

Apalache による検証（Docker 経由）:

```bash
docker run --rm -v $(pwd):/workspace \
  ghcr.io/apalache-mc/apalache:v0.45.4 \
  check --config=/workspace/tier1_bidi_handshake_apalache.json \
  /workspace/tier1_bidi_handshake.tla
```

TLC による検証:

```bash
tlc -config tier1_bidi_handshake.cfg tier1_bidi_handshake.tla
```

## 検証済みスペック一覧

| スペックファイル | obligation_id | cell_state |
|----------------|---------------|------------|
| tier1_bidi_handshake.tla | tier1_bidi_handshake_safety | v1_baseline_verified |

## 関連ドキュメント

- `src/formal/proof_inventory/inventory.yaml` — 全 95 proof obligation のカタログ
- `src/formal/proof_classes.yaml` — proof_class 定義
- `src/formal/dual_review/` — dual sign-off レコード
