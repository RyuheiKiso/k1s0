# diff コマンド

← [CLI 設計書](./)

## 目的

k1s0 リポジトリの Git diff を表示するユーティリティコマンド。ステージング済みおよび未ステージングの変更内容を確認できる。

## 使用例

```bash
# 未ステージングの変更を表示
k1s0 diff

# ステージング済みの変更を表示
k1s0 diff --staged

# 特定パスの差分
k1s0 diff -- feature/backend/rust/
```
