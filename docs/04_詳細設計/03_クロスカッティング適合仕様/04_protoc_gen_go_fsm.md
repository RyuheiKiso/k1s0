---
id: detail.tier2.protoc_gen_go_fsm
axis: tier2
phase: cross_cutting
kind: cross_cut_spec
status: draft
depends_on:
  - arch.tier2.tier2_index
  - detail.tier2.tier2_enforcement
  - detail.client.sdk_distribution_conformance
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D]
  proof_classes:
    - v1_property_axiom_proof
    - v1_program_correctness_proof
---

# protoc-gen-k1s0-go-fsm（v1）

## 位置づけ
- 06_tier2 状態遷移パターン / 言語スタック / 12_client/15 クライアント SDK 配布適合仕様 と双方向 lock
- aggregate 状態遷移を「型レベルで不正遷移を防止」する規律を、Rust / C# / TypeScript と同等に Go でも compile-time に達成するための protoc plugin

## 不可避性
- Go は sum type / sealed class を持たず、proto3 codegen は単一 message struct + state enum を生成。`Draft.confirm() → Confirmed` の型遷移を type system で禁止する経路が標準 codegen では存在せず、runtime guard に縮退する
- Go の generics は type parameter 値による method 限定を持たない（`func (o OrderTyped[Draft]) Confirm()` の構文は成立しない）。phantom type pattern を Rust / C# と同型に Go へ移植することは言語仕様で不能のため採らない
- 代わりに「状態ごとに別 concrete struct を生成し、各 struct には許可された遷移 method のみを生やす」typestate via nominal types を採用する。protoc 標準 plugin は生成しないため本 plugin で proto annotation → 状態別 struct 自動生成を新設

## proto annotation
proto file に `k1s0/fsm.proto` を import し、aggregate message に annotation を付ける:

```
import "k1s0/fsm/v1/fsm.proto";

message Order {
  option (k1s0.fsm.v1.aggregate) = {
    state_field: "state",
    states: ["Draft", "Confirmed", "Shipped", "Cancelled"],
    transitions: [
      { from: "Draft",     to: "Confirmed",  by: "confirm" },
      { from: "Confirmed", to: "Shipped",    by: "ship"    },
      { from: "Draft",     to: "Cancelled",  by: "cancel"  },
      { from: "Confirmed", to: "Cancelled",  by: "cancel"  }
    ]
  };

  string id = 1;
  OrderState state = 2;
  ...
}
```

## 生成コード（Go）
- sealed marker interface（unexported method `isOrderState()` で sealed）
- 状態別 concrete struct（`DraftOrder` / `ConfirmedOrder` / `ShippedOrder` / `CancelledOrder`、typestate via nominal types）
- transitions: 許可された遷移のみ method を生やす（`DraftOrder.Confirm() → ConfirmedOrder` など）。`ShippedOrder` / `CancelledOrder` には遷移 method 無し
- load 経路: `LoadOrder(raw *Order) (OrderState, error)` で DB から読んだ raw を sealed interface に振り分け、利用側は type switch で網羅

不正遷移（`ShippedOrder.Confirm()` / `CancelledOrder.Ship()` 等）はそもそも当該 method が存在しないため compile fail（type system が型遷移を強制）。

## 4 言語等価強度（compile-time + lint + property の三層合算）
- Rust: phantom type + sealed trait（既存 codegen で対応、compile-time に bypass 不可）
- C#: sealed record + phantom type parameter + Roslyn analyzer（既存 codegen で対応、compile-time + analyzer で bypass 検出）
- TS: branded type（intersection with unique symbol）+ phantom type parameter + ESLint custom rule（既存 codegen で対応、compile-time + lint で bypass 検出）
- Go: 本 plugin で 状態別 concrete struct + package-internal sealed interface（unexported method による sealed）を生成（typestate via nominal types）。Go の structural typing は compile-time の真の sealed を保証できないため下記の三層 enforcement で Rust phantom type と同等の強度に近づける:
  - 層 G-1: package-sealed pattern（unexported method `isOrderState()` を sealed interface に持たせ、package 外で状態 struct を満たす型を後付け実装させない）
  - 層 G-2: golangci-lint の `forbidigo` で `reflect.New` / `unsafe.Pointer` 経由の状態 struct 構築を application boundary で ban（`forbidden_call.lock.yaml` の単一の真）
  - 層 G-3: 19_検証規律適合仕様 の `property_axiom` で「`LoadOrder` + 生成済み遷移 method 以外から状態 struct が構築不能」を runtime 検査（go-exhaustive + ast walker + 値生成テストで確率的に bypass 不可性を確認）

結果、4 言語すべてで「不正遷移は compile fail（Rust / C# / TS）または compile fail + lint + property の三層検査（Go）」が成立する。「保証強度は等価」表現は撤回し、「等価強度」（Rust: compile-time / Go: package-sealed + lint + property / TS: branded type + lint / .NET: closed hierarchy + analyzer）として粒度を honest に明示。Go の bypass 経路は本三層で closure するが、reflection / unsafe を含む標準 library 系統は allowlist で除外される領域が残る（lint 完全 ban は標準 library import 自体を阻むため不採用）。

## bypass 防止
- reflection / unsafe / type assertion 経由で `ConfirmedOrder` から `DraftOrder` へ強制 cast したり、`unsafe.Pointer` で state struct を直接生成して `raw.State` との不整合を作る経路は Go では塞ぎきれない（reflect.New / unsafe.Pointer は標準 library 含めて禁止できない）。本機構は best-effort lint で確率を下げる位置付け、application boundary での bypass を完全 enforce する手段は持たない
- 防御:
  - golangci-lint カスタム rule で `reflect.New` の `DraftOrder` / `ConfirmedOrder` / `ShippedOrder` / `CancelledOrder` 系生成を ban（`forbidden_call.lock.yaml`）
  - 同 rule で `unsafe.Pointer` を経由した state struct 生成 / state struct 間の cast を ban
  - tier2 application 側で state struct を `LoadOrder` / 生成済み遷移 method 以外で構築しない invariant を CI 検査（go-exhaustive + ast walker による direct-construct 検出）
  - 上記 lint は application boundary が対象、標準 library 実装等の系統的に必要な reflect / unsafe 利用は allowlist で除外

## proto registration
- proto file は Buf で管理、`k1s0/fsm/v1/fsm.proto` は Buf Schema Registry に登録
- `protoc-gen-k1s0-go-fsm` は Buf plugin として配布、`buf.gen.yaml` に plugin entry を追加するだけで生成

## 副作用
- generics 導入で Go 1.18+ floor が前提（既存規律で 1.21+ なので問題なし）
- 生成コード量は現行の 1.5 倍程度（state ごと wrapper 型 + transition method の組合せ）。build time +5〜10% 想定
- load 経路で switch statement が必要、forgot pattern を防ぐため CI で全 enum 値が switch されているかを exhaustivity check（go-exhaustive）

## 整合
- 整合 1: 06_tier2 状態遷移パターン の「型レベル不正遷移防止」は本 plugin 経由で 4 言語同等成立。runtime guard 縮退記述は撤回
- 整合 2: 06_tier2 言語スタック の Rust/C#/Go/TS 同等保証は本 plugin で技術的に閉じる
- 整合 3: 12_client/15 クライアント SDK 配布適合仕様 の「全 SDK で aggregate 操作 API 同型」は状態 type 表現の言語別差異（Rust phantom type / C# phantom type / Go 状態別 nominal type / TS tagged union）を `naming-rule.yaml` で吸収して proto FQN 等価で担保
- 整合 4: 19_検証規律適合仕様 に「不正遷移コードサンプルが各言語で compile fail することを golden test」を追加
- 整合 5: 11_ops/14_強制機構 / 13_test/14_強制機構 に「`reflect.New` / unsafe による phantom type bypass は CI block」を追加

## 関連参照
- [tier2 設計方針 index](../../03_概要設計/03_tier2設計方針/README.md)
- [tier2 強制機構](../02_強制機構/02_tier2強制機構.md)
- [クライアント SDK 配布適合仕様](../01_適合仕様/18_クライアントSDK配布適合仕様.md)
- [検証規律適合仕様](../01_適合仕様/19_検証規律適合仕様.md)
