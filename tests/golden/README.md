# tests/golden — 出力固定 Snapshot テスト

`k1s0-scaffold` CLI の出力など、確定済みの成果物を snapshot し、変更時に diff を検出する。

## 構造

```text
golden/
├── README.md
├── scaffold-outputs/      # k1s0-scaffold が生成する 4 ServiceType の expected
│   ├── tier2-go-service/
│   │   └── expected.tar.gz
│   ├── tier2-dotnet-service/
│   ├── tier3-bff/
│   └── tier3-web/
└── diff-tool/
    └── compare-outputs.sh
```

## 実行

```bash
tests/golden/diff-tool/compare-outputs.sh tier2-go-service
```

scaffold が生成する `/tmp/scaffold-out` と `expected.tar.gz` を `diff -r` で比較し、差分があれば exit 1 する。意図的な scaffold テンプレ更新時は `expected.tar.gz` を再生成して PR に含める。

## CI

`.github/workflows/_reusable-test.yml` の `golden` job が PR 毎に 4 種すべて検証する。テンプレ変更を含む PR では PR テンプレートで「expected.tar.gz を更新したか」を確認する。
