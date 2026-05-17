# Stainless Formal Proof

このディレクトリには k1s0 プロジェクトの Stainless による形式検証スペックを格納する。

## 検証対象

| スペックファイル | obligation_id | cell_state |
|----------------|---------------|------------|
| `src/main/scala/k1s0/data/AtomicThreeTableWrite.scala` | data_atomic_three_table_write | v1_baseline_verified |

## ディレクトリ構成

```
stainless/
├── README.md                              # 本ファイル
├── build.sbt                              # Stainless 0.9.8 ビルド設定
├── Makefile                               # Docker 経由での検証実行
└── src/
    └── main/
        └── scala/
            └── k1s0/
                └── data/
                    └── AtomicThreeTableWrite.scala  # P1-P4 invariant 検証
```

## 検証内容（P1-P4 invariant）

| invariant | 内容 |
|-----------|------|
| P1 | State change + Outbox + Audit の三表書込が同一トランザクション内で完了する |
| P2 | Outbox insert 失敗時にロールバックする（atomicity） |
| P3 | PII データは redact 済みの状態で Audit に記録される |
| P4 | event emit 経路は単一（Outbox 経由のみ） |

## 実行方法

```bash
cd stainless/
make verify
```

## 関連ドキュメント

- `src/formal/proof_inventory/inventory.yaml` — obligation カタログ
- `src/formal/dual_review/data_atomic_three_table_write.dual_review.lock.yaml` — dual sign-off レコード
