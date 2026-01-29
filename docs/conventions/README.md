# 規約ドキュメント

本ディレクトリには、k1s0 プロジェクトの各種規約を格納する。

## 規約一覧

| ドキュメント | 説明 |
|-------------|------|
| [サービス構成規約](service-structure.md) | サービスのディレクトリ構成、必須ファイル、命名規則 |
| [設定と秘密情報の規約](config-and-secrets.md) | 設定ファイル、秘密情報の取り扱い |
| [API 契約管理規約](api-contracts.md) | gRPC/REST の契約管理、互換性ルール |
| [観測性規約](observability.md) | ログ/トレース/メトリクスの出力規約 |
| [エラーハンドリング規約](error-handling.md) | エラー表現、エラーコード、レスポンス形式 |
| [バージョニング規約](versioning.md) | SemVer、互換性ポリシー、manifest スキーマ |
| [Domain 境界ガイドライン](domain-boundaries.md) | domain 層の境界判断基準 |
| [非推奨化ポリシー](deprecation-policy.md) | domain の非推奨化プロセス |

## 規約の位置づけ

- 規約は **MUST（必須）** と **SHOULD（推奨）** に分かれる
- MUST は `k1s0 lint` および CI で自動検査される
- 規約の追加・変更は ADR で記録する

## 規約違反の検知

```bash
# CLI で規約違反を検査
k1s0 lint

# または Windows 向けスクリプト
.\scripts\lint.ps1
```

## 関連ドキュメント

- [ADR](../adr/README.md): アーキテクチャ決定記録
- [構想.md](../../work/構想.md): 全体方針
- [プラン.md](../../work/プラン.md): 実装計画
