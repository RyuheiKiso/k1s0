// signal_trigger_test.ts — OSS ライフサイクル signal トリガーテスト
// 08_OSSライフサイクル適合仕様に基づく 8 signal × 6 lifecycle_class の検証を実装する
// 各 signal が正しいアクション (UPDATE_REQUIRED / FREEZE / REPLACE) を発火するかを vitest でテストする

// vitest から test / describe / expect をインポートする
import { describe, test, expect } from "vitest";

// ============================================================
// Signal 型定義
// ============================================================

// 08_OSSライフサイクル適合仕様で定義された 8 つの OSS lifecycle signal 型
type OssSignal =
  // EOL_ANNOUNCED: OSS の EOL (End of Life) が宣言された signal
  | "EOL_ANNOUNCED"
  // VULNERABILITY_CRITICAL: 重大な脆弱性 (CVSS >= 9.0) が検出された signal
  | "VULNERABILITY_CRITICAL"
  // MAJOR_VERSION_DEPRECATED: メジャーバージョンが非推奨になった signal
  | "MAJOR_VERSION_DEPRECATED"
  // UNMAINTAINED: メンテナンスが停止された signal (last commit > 12 ヶ月)
  | "UNMAINTAINED"
  // LICENSE_CHANGE: ライセンスが変更された signal（商用非互換に変化した場合）
  | "LICENSE_CHANGE"
  // SBOM_DRIFT: SBOM (Software Bill of Materials) が drift した signal
  | "SBOM_DRIFT"
  // KYVERNO_REJECT: Kyverno ポリシーが image 使用を reject した signal
  | "KYVERNO_REJECT"
  // COSIGN_VERIFY_FAIL: cosign 署名検証が失敗した signal
  | "COSIGN_VERIFY_FAIL";

// ============================================================
// Action 型定義
// ============================================================

// 各 signal に対して発火するアクション型を定義する
type OssAction =
  // UPDATE_REQUIRED: 該当 OSS のアップデートを必須とするアクション
  | "UPDATE_REQUIRED"
  // FREEZE: 該当 OSS の新規デプロイを凍結するアクション
  | "FREEZE"
  // REPLACE: 該当 OSS を代替品に置き換えるアクション
  | "REPLACE";

// ============================================================
// lifecycle_class 型定義
// ============================================================

// 6 つの OSS lifecycle_class を定義する（仕様: 08_OSSライフサイクル適合仕様）
type LifecycleClass =
  // L1: 最重要 OSS（tier1 transport / auth など、直接 RPC に使用する OSS）
  | "L1_CRITICAL"
  // L2: 重要 OSS（tier1 server が依存する主要 OSS）
  | "L2_IMPORTANT"
  // L3: 標準 OSS（一般的な OSS コンポーネント）
  | "L3_STANDARD"
  // L4: 開発支援ツール（build / test / lint に使用する OSS）
  | "L4_DEV_TOOL"
  // L5: サンドボックス（PoC / 実験的使用の OSS）
  | "L5_SANDBOX"
  // L6: 非推奨（移行待ちの OSS）
  | "L6_DEPRECATED";

// ============================================================
// Signal → Action マッピング定義
// ============================================================

// signal と lifecycle_class の組み合わせに対するアクションを定義するマッピング型
type SignalActionMapping = {
  // signal: トリガーされた signal 種別
  signal: OssSignal;
  // lifecycle_class: 対象 OSS の lifecycle_class
  lifecycle_class: LifecycleClass;
  // expected_action: この組み合わせで期待されるアクション
  expected_action: OssAction;
};

// ============================================================
// signal → action 解決ロジック
// ============================================================

// signal と lifecycle_class からアクションを決定する純粋関数
function resolveAction(signal: OssSignal, lifecycle_class: LifecycleClass): OssAction {
  // COSIGN_VERIFY_FAIL は lifecycle_class に関わらず FREEZE とする（供給チェーン侵害を即座に阻止する）
  if (signal === "COSIGN_VERIFY_FAIL") {
    // 署名検証失敗は全 class で即時 FREEZE を発令する
    return "FREEZE";
  }
  // KYVERNO_REJECT は lifecycle_class に関わらず FREEZE とする（ポリシー違反は即座にブロックする）
  if (signal === "KYVERNO_REJECT") {
    // Kyverno reject は全 class で即時 FREEZE を発令する
    return "FREEZE";
  }
  // VULNERABILITY_CRITICAL は L1/L2 では REPLACE、L3 以下では UPDATE_REQUIRED とする
  if (signal === "VULNERABILITY_CRITICAL") {
    // L1_CRITICAL と L2_IMPORTANT は重大脆弱性に対して REPLACE を要求する
    if (lifecycle_class === "L1_CRITICAL" || lifecycle_class === "L2_IMPORTANT") {
      return "REPLACE";
    }
    // それ以外は UPDATE_REQUIRED とする
    return "UPDATE_REQUIRED";
  }
  // EOL_ANNOUNCED は L1/L2/L3 では REPLACE、L4 以下では UPDATE_REQUIRED とする
  if (signal === "EOL_ANNOUNCED") {
    // L1/L2/L3 は EOL 宣言に対して REPLACE を要求する
    if (
      lifecycle_class === "L1_CRITICAL" ||
      lifecycle_class === "L2_IMPORTANT" ||
      lifecycle_class === "L3_STANDARD"
    ) {
      return "REPLACE";
    }
    // それ以外は UPDATE_REQUIRED とする
    return "UPDATE_REQUIRED";
  }
  // UNMAINTAINED は L1/L2 では REPLACE とする
  if (signal === "UNMAINTAINED") {
    // L1/L2 はメンテナンス停止に対して REPLACE を要求する
    if (lifecycle_class === "L1_CRITICAL" || lifecycle_class === "L2_IMPORTANT") {
      return "REPLACE";
    }
    // L3 以下は FREEZE として評価を保留する
    return "FREEZE";
  }
  // LICENSE_CHANGE は全 class で FREEZE とする（ライセンス審査が必要）
  if (signal === "LICENSE_CHANGE") {
    // ライセンス変更は全 class で審査完了まで FREEZE を維持する
    return "FREEZE";
  }
  // MAJOR_VERSION_DEPRECATED は L1/L2 では UPDATE_REQUIRED、それ以外は FREEZE とする
  if (signal === "MAJOR_VERSION_DEPRECATED") {
    // L1/L2 はメジャーバージョン非推奨に対して UPDATE_REQUIRED を要求する
    if (lifecycle_class === "L1_CRITICAL" || lifecycle_class === "L2_IMPORTANT") {
      return "UPDATE_REQUIRED";
    }
    // それ以外は FREEZE として評価を保留する
    return "FREEZE";
  }
  // SBOM_DRIFT は全 class で UPDATE_REQUIRED とする（SBOM 再生成が必要）
  if (signal === "SBOM_DRIFT") {
    // SBOM ドリフトは全 class で SBOM 再生成・検証のため UPDATE_REQUIRED を発令する
    return "UPDATE_REQUIRED";
  }
  // 未定義 signal に対しては FREEZE をデフォルトとする（安全側に倒す）
  return "FREEZE";
}

// ============================================================
// テスト: 8 signal × 6 lifecycle_class の組み合わせ検証
// ============================================================

// OSS lifecycle signal トリガーテスト群
describe("OSS lifecycle signal trigger tests", () => {
  // ---- COSIGN_VERIFY_FAIL signal のテスト ----

  // 全 lifecycle_class で COSIGN_VERIFY_FAIL が FREEZE を発火することを検証する
  describe("COSIGN_VERIFY_FAIL signal", () => {
    // L1_CRITICAL での COSIGN_VERIFY_FAIL は FREEZE を発火する
    test("L1_CRITICAL: COSIGN_VERIFY_FAIL triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("COSIGN_VERIFY_FAIL", "L1_CRITICAL");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L2_IMPORTANT での COSIGN_VERIFY_FAIL は FREEZE を発火する
    test("L2_IMPORTANT: COSIGN_VERIFY_FAIL triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("COSIGN_VERIFY_FAIL", "L2_IMPORTANT");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L3_STANDARD での COSIGN_VERIFY_FAIL は FREEZE を発火する
    test("L3_STANDARD: COSIGN_VERIFY_FAIL triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("COSIGN_VERIFY_FAIL", "L3_STANDARD");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L4_DEV_TOOL での COSIGN_VERIFY_FAIL は FREEZE を発火する
    test("L4_DEV_TOOL: COSIGN_VERIFY_FAIL triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("COSIGN_VERIFY_FAIL", "L4_DEV_TOOL");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L5_SANDBOX での COSIGN_VERIFY_FAIL は FREEZE を発火する
    test("L5_SANDBOX: COSIGN_VERIFY_FAIL triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("COSIGN_VERIFY_FAIL", "L5_SANDBOX");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L6_DEPRECATED での COSIGN_VERIFY_FAIL は FREEZE を発火する
    test("L6_DEPRECATED: COSIGN_VERIFY_FAIL triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("COSIGN_VERIFY_FAIL", "L6_DEPRECATED");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- KYVERNO_REJECT signal のテスト ----

  // 全 lifecycle_class で KYVERNO_REJECT が FREEZE を発火することを検証する
  describe("KYVERNO_REJECT signal", () => {
    // L1_CRITICAL での KYVERNO_REJECT は FREEZE を発火する
    test("L1_CRITICAL: KYVERNO_REJECT triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("KYVERNO_REJECT", "L1_CRITICAL");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L3_STANDARD での KYVERNO_REJECT は FREEZE を発火する
    test("L3_STANDARD: KYVERNO_REJECT triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("KYVERNO_REJECT", "L3_STANDARD");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- VULNERABILITY_CRITICAL signal のテスト ----

  // VULNERABILITY_CRITICAL signal のアクション分岐を検証する
  describe("VULNERABILITY_CRITICAL signal", () => {
    // L1_CRITICAL での VULNERABILITY_CRITICAL は REPLACE を発火する
    test("L1_CRITICAL: VULNERABILITY_CRITICAL triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("VULNERABILITY_CRITICAL", "L1_CRITICAL");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // L2_IMPORTANT での VULNERABILITY_CRITICAL は REPLACE を発火する
    test("L2_IMPORTANT: VULNERABILITY_CRITICAL triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("VULNERABILITY_CRITICAL", "L2_IMPORTANT");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // L3_STANDARD での VULNERABILITY_CRITICAL は UPDATE_REQUIRED を発火する
    test("L3_STANDARD: VULNERABILITY_CRITICAL triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("VULNERABILITY_CRITICAL", "L3_STANDARD");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // L4_DEV_TOOL での VULNERABILITY_CRITICAL は UPDATE_REQUIRED を発火する
    test("L4_DEV_TOOL: VULNERABILITY_CRITICAL triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("VULNERABILITY_CRITICAL", "L4_DEV_TOOL");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });
  });

  // ---- EOL_ANNOUNCED signal のテスト ----

  // EOL_ANNOUNCED signal のアクション分岐を検証する
  describe("EOL_ANNOUNCED signal", () => {
    // L1_CRITICAL での EOL_ANNOUNCED は REPLACE を発火する
    test("L1_CRITICAL: EOL_ANNOUNCED triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("EOL_ANNOUNCED", "L1_CRITICAL");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // L3_STANDARD での EOL_ANNOUNCED は REPLACE を発火する
    test("L3_STANDARD: EOL_ANNOUNCED triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("EOL_ANNOUNCED", "L3_STANDARD");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // L4_DEV_TOOL での EOL_ANNOUNCED は UPDATE_REQUIRED を発火する
    test("L4_DEV_TOOL: EOL_ANNOUNCED triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("EOL_ANNOUNCED", "L4_DEV_TOOL");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });
  });

  // ---- UNMAINTAINED signal のテスト ----

  // UNMAINTAINED signal のアクション分岐を検証する
  describe("UNMAINTAINED signal", () => {
    // L1_CRITICAL での UNMAINTAINED は REPLACE を発火する
    test("L1_CRITICAL: UNMAINTAINED triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("UNMAINTAINED", "L1_CRITICAL");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // L3_STANDARD での UNMAINTAINED は FREEZE を発火する
    test("L3_STANDARD: UNMAINTAINED triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("UNMAINTAINED", "L3_STANDARD");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- LICENSE_CHANGE signal のテスト ----

  // 全 lifecycle_class で LICENSE_CHANGE が FREEZE を発火することを検証する
  describe("LICENSE_CHANGE signal", () => {
    // L1_CRITICAL での LICENSE_CHANGE は FREEZE を発火する
    test("L1_CRITICAL: LICENSE_CHANGE triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("LICENSE_CHANGE", "L1_CRITICAL");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // L5_SANDBOX での LICENSE_CHANGE は FREEZE を発火する
    test("L5_SANDBOX: LICENSE_CHANGE triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("LICENSE_CHANGE", "L5_SANDBOX");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- MAJOR_VERSION_DEPRECATED signal のテスト ----

  // MAJOR_VERSION_DEPRECATED signal のアクション分岐を検証する
  describe("MAJOR_VERSION_DEPRECATED signal", () => {
    // L1_CRITICAL での MAJOR_VERSION_DEPRECATED は UPDATE_REQUIRED を発火する
    test("L1_CRITICAL: MAJOR_VERSION_DEPRECATED triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("MAJOR_VERSION_DEPRECATED", "L1_CRITICAL");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // L3_STANDARD での MAJOR_VERSION_DEPRECATED は FREEZE を発火する
    test("L3_STANDARD: MAJOR_VERSION_DEPRECATED triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("MAJOR_VERSION_DEPRECATED", "L3_STANDARD");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- SBOM_DRIFT signal のテスト ----

  // 全 lifecycle_class で SBOM_DRIFT が UPDATE_REQUIRED を発火することを検証する
  describe("SBOM_DRIFT signal", () => {
    // L1_CRITICAL での SBOM_DRIFT は UPDATE_REQUIRED を発火する
    test("L1_CRITICAL: SBOM_DRIFT triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("SBOM_DRIFT", "L1_CRITICAL");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // L6_DEPRECATED での SBOM_DRIFT は UPDATE_REQUIRED を発火する
    test("L6_DEPRECATED: SBOM_DRIFT triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("SBOM_DRIFT", "L6_DEPRECATED");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });
  });

  // ---- マッピング網羅テスト ----

  // 定義済み SignalActionMapping の全エントリが正しく解決されることを検証する
  describe("SignalActionMapping exhaustive check", () => {
    // テスト対象のマッピングエントリを定義する（代表的な 8 signal × 代表 lifecycle_class）
    const mappings: SignalActionMapping[] = [
      // EOL_ANNOUNCED × L1_CRITICAL → REPLACE
      { signal: "EOL_ANNOUNCED", lifecycle_class: "L1_CRITICAL", expected_action: "REPLACE" },
      // VULNERABILITY_CRITICAL × L1_CRITICAL → REPLACE
      {
        signal: "VULNERABILITY_CRITICAL",
        lifecycle_class: "L1_CRITICAL",
        expected_action: "REPLACE",
      },
      // MAJOR_VERSION_DEPRECATED × L2_IMPORTANT → UPDATE_REQUIRED
      {
        signal: "MAJOR_VERSION_DEPRECATED",
        lifecycle_class: "L2_IMPORTANT",
        expected_action: "UPDATE_REQUIRED",
      },
      // UNMAINTAINED × L2_IMPORTANT → REPLACE
      { signal: "UNMAINTAINED", lifecycle_class: "L2_IMPORTANT", expected_action: "REPLACE" },
      // LICENSE_CHANGE × L3_STANDARD → FREEZE
      { signal: "LICENSE_CHANGE", lifecycle_class: "L3_STANDARD", expected_action: "FREEZE" },
      // SBOM_DRIFT × L4_DEV_TOOL → UPDATE_REQUIRED
      {
        signal: "SBOM_DRIFT",
        lifecycle_class: "L4_DEV_TOOL",
        expected_action: "UPDATE_REQUIRED",
      },
      // KYVERNO_REJECT × L5_SANDBOX → FREEZE
      { signal: "KYVERNO_REJECT", lifecycle_class: "L5_SANDBOX", expected_action: "FREEZE" },
      // COSIGN_VERIFY_FAIL × L6_DEPRECATED → FREEZE
      {
        signal: "COSIGN_VERIFY_FAIL",
        lifecycle_class: "L6_DEPRECATED",
        expected_action: "FREEZE",
      },
    ];

    // 各マッピングエントリに対してテストを実行する
    for (const mapping of mappings) {
      // テストケース名を動的に生成する
      test(`${mapping.signal} × ${mapping.lifecycle_class} → ${mapping.expected_action}`, () => {
        // resolveAction でアクションを取得する
        const actual = resolveAction(mapping.signal, mapping.lifecycle_class);
        // 期待するアクションと一致することを検証する
        expect(actual).toBe(mapping.expected_action);
      });
    }
  });
});
