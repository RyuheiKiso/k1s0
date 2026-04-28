# tools/trace-check

IMP-TRACE-CI-010〜019 の実装。`docs/05_実装/99_索引` の整合性を CI で検証する 5 スクリプト群。

| スクリプト | IMP-ID | 責務 |
|---|---|---|
| `check-grand-total.sh` | CI-011 | 台帳サマリ列（POL/実装ID/合計）と詳細行数の突き合わせ |
| `check-cross-ref.sh` | CI-012 | 90_対応索引と台帳の双方向 ID 整合 |
| `check-orphan.sh` | CI-013 | ADR/DS-SW-COMP/NFR マトリクスから参照されない孤立 ID 検出 |
| `check-duplicate.sh` | CI-014a | 同一 IMP-* ID の重複採番検出 |
| `check-reserve.sh` | CI-014b | 予約帯 001-099 外の採番検出（`reserve-ranges.yaml` 参照） |
