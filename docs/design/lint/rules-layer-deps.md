# K040-K047: 層間依存関係検査

← [Lint 設計書](./)

---

## 概要

3層アーキテクチャ（framework -> domain -> feature）における層間の依存関係を検証します。

## K040: 層間依存の基本違反

```
重要度: Error
対象: 全ての manifest.json
```

層間依存の基本ルール:
- feature は domain または framework に依存可能
- domain は framework に依存可能
- framework は何にも依存しない（最下層）

違反例:
- domain 層の manifest.json に `domain` フィールドが設定されている
- framework 層の manifest.json に `domain` フィールドが設定されている

## K041: domain が見つからない

```
重要度: Error
対象: feature 層の manifest.json
```

feature が依存する domain が存在するか確認します。

検査対象:
- manifest.json の `domain` フィールド
- manifest.json の `dependencies.domain` オブジェクト

## K042: domain バージョン制約不整合

```
重要度: Error
対象: feature 層の manifest.json
```

feature が指定するバージョン制約と、domain の実際のバージョンが一致するか確認します。

例:
```json
// feature の manifest.json
{
  "domain": "manufacturing",
  "domain_version": "^1.2.0"
}
```

```json
// domain の manifest.json
{
  "version": "0.9.0"  // ^1.2.0 を満たさない -> K042 違反
}
```

## K043: 循環依存の検出

```
重要度: Error
対象: domain 層の manifest.json
```

domain 間の循環依存を検出します。

例:
- domain-a が domain-b に依存
- domain-b が domain-a に依存
-> 循環依存

## K044: 非推奨 domain の使用

```
重要度: Warning
対象: feature 層の manifest.json
```

非推奨（deprecated）の domain を使用している場合に警告します。

## K045: min_framework_version 違反

```
重要度: Warning
対象: domain 層、feature 層の manifest.json
```

domain の `min_framework_version` と k1s0 のバージョンを比較し、
要件を満たしていない場合に警告します。

## K046: breaking_changes の影響

```
重要度: Warning
対象: feature 層の manifest.json
```

依存する domain に破壊的変更がある場合に警告します。

## K047: domain 層の version 未設定

```
重要度: Error
対象: domain 層の manifest.json
```

domain 層には `version` フィールドが必須です。
