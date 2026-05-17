---
id: arch.client.distribution_semver_policy
axis: client
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.client.client_index
  - arch.client.sdk_distribution_policy
covered_by:
  defense_in_depth_layers: [A, B, E]
  proof_classes:
    - v1_program_correctness_proof
---

# client 配布 SemVer 方針

## 一文方針
- 全言語 SDK は同一 MAJOR.MINOR を同時に release する（lockstep SemVer）。skew は PATCH のみ許容、MAJOR.MINOR skew は CI fail。

## バージョン規約
- 形式: `MAJOR.MINOR.PATCH-PRERELEASE+BUILD`（SemVer 2.0.0）
- 全言語 SDK で同一の MAJOR.MINOR を持つ
- PATCH は言語別 hotfix のみで増えてよい（waterfall release 許容）

## 1.0.0 完璧 release
- 1.0.0 は 5 distribution_class 全てが conformance test green
- 全 9 言語（.NET / Java / Node / Browser TS / Tauri / Rust / Python / Ruby / Go）が同 1.0.0 同時 release
- 1.0.0 後の MINOR up は purely additive のみ。既存 RPC の signature 変更 / capability_class 削除は MAJOR up

## MAJOR up（破壊的変更）の手続
- tier1 14 OSS lifecycle 適合仕様の deprecation_window と同期:
  - `deprecation_window`: 180 日（最低 6 ヶ月）
  - 1 LTS overlap: 旧 MAJOR と新 MAJOR が 1 LTS 期間 並走
  - 移行 toolchain: tier1 が破壊的変更 1 つにつき 1 つの codemod / migration script を提供（codemod は SDK 言語ごと）
  - 14 軸 lifecycle_event_corpus に MAJOR up event を登録、`signal_source_endpoint` で外部監視
- 旧 MAJOR の sunset: deprecation_window 経過後、Harbor 上の旧パッケージは pull 可能 / push 不可で永久保管

## MINOR up（後方互換）の規約
- purely additive のみ:
  - 新 RPC 追加（既存 RPC は変更不可）
  - 新 capability_class の追加（旧 class 削除は MAJOR）
  - 新 OTel SemConv 追加（旧 attribute は alias 維持）
  - 新 distribution_class の追加（旧は維持）
  - 新オプション field（proto は optional / 新 reserved 番号での追加）
- MINOR up でも全言語同時 release。Buf の breaking change detection を CI gate

## PATCH（hotfix）の規約
- 言語別 hotfix は purely バグ修正 / セキュリティパッチのみ。public API 追加 / 削除 / 変更を含めない
- hotfix は waterfall release 許容、ただし `sdk_inventory.lock.yaml` に release timestamp を記録、14 軸 signal で監視
- hotfix waterfall は 30 日以内に他言語へ同等修正 propagate が必須

## release lockstep の物理 enforce
- 全言語 SDK の release は同一 GitHub Actions workflow か Tekton pipeline で並列 trigger
- `sdk_inventory.lock.yaml` に MAJOR.MINOR の skew が検出された場合、CI fail
- tag は git に `v1.0.0` 1 つだけ、cosign signed。各言語 package registry には同 version tag をそれぞれ push

## pre-release / canary
- alpha / beta / rc は同 lockstep を維持（全言語 `1.0.0-rc.3` を同時 release）
- canary は GitHub Actions で「main branch ごとに `-alpha.N`」を全言語同時に push

## 配信経路
- 各言語 official registry（`npm.org` / `nuget.org` / `maven central` / `pypi.org` / `rubygems.org` / `crates.io` / `proxy.golang.org`）に push
- enterprise 利用は Harbor mirror（NuGet repo / Maven repo / PyPI repo / RubyGems repo / cargo mirror / Verdaccio / Athens）経由を強制。infra Kyverno admission policy が Harbor 以外の registry 参照を reject
- 全 package は Cosign signed（npm は sigstore、PyPI は PEP 740 sigstore signing、cargo は Harbor の Cosign signed blob）

## SBOM / 脆弱性スキャン
- 全 release artifact について Syft で SBOM 生成、Grype で脆弱性スキャン CI 必須。critical / high CVE 検出は release block
- SBOM は in-toto attestation で chain of custody を記録（security15 threat-model 整合）

## deprecation 通知
- SDK 起動時 / 周期的 health probe で server に SDK version を送信、server が「deprecated」と判定した場合 SDK は `OnSdkDeprecated` event を発火
- deprecation 警告は console.warn / .NET ILogger / SLF4J Warn に出力、deprecation_window 残日数を含む
- sunset 後の SDK は server から `426 Upgrade Required` を受領、SDK は `OnSdkRetired` event を発火

## build reproducibility
- 全 SDK package の build reproducibility は 10_security/17_build_provenance 適合仕様 の `v1_library_sdk_release` class が単一所有（Nix flake bit identical / multi-builder consensus 3/3 / reviewer attest 2 名 / SLSA L4）
- 9 言語の SDK は Nix flake で言語横断 hermetic build。Bazel remote-exec + Nix sandbox + Buildkit `RUN --network=none` の三重 sandbox

## 採用しない選択肢
- 言語別の独立 SemVer
- 言語別の release date skew（半年遅れの Python release 等）
- PATCH での API 追加
- GA 後の MAJOR 同時複数並走（v1 と v2 と v3）

## 至高路線における立ち位置
- 「Python 版だけ release を遅らせる」を採らない。lockstep 必須、全言語同時
- 「破壊的変更を入れたいから MINOR で済ませる」を採らない

## 関連参照
- [client 設計方針 index](README.md)
- [SDK 配布構成方針](01_SDK配布構成方針.md)
- [build_provenance 適合仕様](../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md)
