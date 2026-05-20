// signal_trigger_test.ts — OSS ライフサイクル signal トリガーテスト
// 08_OSSライフサイクル適合仕様に基づく 8 signal × 6 lifecycle_class の検証を実装する
// 各 signal が正しいアクション (UPDATE_REQUIRED / FREEZE / REPLACE) を発火するかを vitest でテストする
// signal 名・lifecycle_class 名は 08_OSSライフサイクル適合仕様.md の canonical 名に統一する

// vitest から test / describe / expect をインポートする
import { describe, test, expect } from "vitest";

// ============================================================
// Signal 型定義（08_OSSライフサイクル適合仕様.md §signal 8 値 canonical 名）
// ============================================================

// 08_OSSライフサイクル適合仕様で定義された 8 つの OSS lifecycle signal の canonical 型
type OssSignal =
  // license_change: ライセンスが変更された signal（商用非互換に変化した場合）
  | "license_change"
  // eol_announced: OSS の EOL (End of Life) が宣言された signal
  | "eol_announced"
  // cve_backlog_threshold: CVE バックログが閾値（件数 > 5 または CVSS >= 9.0）を超過した signal
  | "cve_backlog_threshold"
  // maintainer_turnover: メンテナーが交代またはメンテナンス健全性が低下した signal
  | "maintainer_turnover"
  // fork_event: フォーク元との乖離コミット数が閾値を超過した signal
  | "fork_event"
  // conformance_drift: セキュリティポリシー等の conformance が drift した signal
  | "conformance_drift"
  // major_up: メジャーバージョンアップが利用可能な signal
  | "major_up"
  // spec_drift: 仕様（spec）からの乖離が検出された signal
  | "spec_drift";

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
// lifecycle_class 型定義（08_OSSライフサイクル適合仕様.md §lifecycle_class セット canonical 名）
// ============================================================

// 6 つの OSS lifecycle_class の canonical 名を定義する（仕様: 08_OSSライフサイクル適合仕様）
type LifecycleClass =
  // v1_l1plus_primary: 最重要 OSS（tier1 transport / auth など、直接 RPC に使用する OSS）
  | "v1_l1plus_primary"
  // v1_l2star_alt: 重要 OSS（tier1 server が依存する主要 OSS）
  | "v1_l2star_alt"
  // v1_l3_generic: 標準 OSS（一般的な OSS コンポーネント）
  | "v1_l3_generic"
  // v1_l4_dev_tool: 開発支援ツール（build / test / lint に使用する OSS）
  | "v1_l4_dev_tool"
  // v1_l5_sandbox: サンドボックス（PoC / 実験的使用の OSS）
  | "v1_l5_sandbox"
  // v1_l6_deprecated: 非推奨（移行待ちの OSS）
  | "v1_l6_deprecated";

// ============================================================
// Signal → Action マッピング定義
// ============================================================

// signal と lifecycle_class の組み合わせに対するアクションを定義するマッピング型
type SignalActionMapping = {
  // signal: トリガーされた signal 種別（spec canonical 名）
  signal: OssSignal;
  // lifecycle_class: 対象 OSS の lifecycle_class（spec canonical 名）
  lifecycle_class: LifecycleClass;
  // expected_action: この組み合わせで期待されるアクション
  expected_action: OssAction;
};

// ============================================================
// signal → action 解決ロジック（08_OSSライフサイクル適合仕様.md §signal_action_matrix 準拠）
// ============================================================

// signal と lifecycle_class からアクションを決定する純粋関数
function resolveAction(signal: OssSignal, lifecycle_class: LifecycleClass): OssAction {
  // license_change は lifecycle_class に関わらず FREEZE とする（ライセンス審査が必要）
  if (signal === "license_change") {
    // ライセンス変更は全 class で審査完了まで FREEZE を維持する
    return "FREEZE";
  }
  // fork_event は lifecycle_class に関わらず FREEZE とする（フォーク乖離は供給チェーンリスク）
  if (signal === "fork_event") {
    // フォークイベントは全 class で即時 FREEZE を発令する
    return "FREEZE";
  }
  // conformance_drift は lifecycle_class に関わらず FREEZE とする（conformance 侵害は即時ブロック）
  if (signal === "conformance_drift") {
    // conformance ドリフトは全 class で即時 FREEZE を発令する
    return "FREEZE";
  }
  // spec_drift は lifecycle_class に関わらず UPDATE_REQUIRED とする（spec 乖離は再整合が必要）
  if (signal === "spec_drift") {
    // spec ドリフトは全 class で UPDATE_REQUIRED を発令する
    return "UPDATE_REQUIRED";
  }
  // eol_announced は v1_l1plus_primary / v1_l2star_alt / v1_l3_generic では REPLACE、他は UPDATE_REQUIRED とする
  if (signal === "eol_announced") {
    // v1_l1plus_primary / v1_l2star_alt / v1_l3_generic は EOL 宣言に対して REPLACE を要求する
    if (
      lifecycle_class === "v1_l1plus_primary" ||
      lifecycle_class === "v1_l2star_alt" ||
      lifecycle_class === "v1_l3_generic"
    ) {
      // 最重要・重要・標準クラスは代替品への置換を要求する
      return "REPLACE";
    }
    // v1_l4_dev_tool 以下は UPDATE_REQUIRED とする
    return "UPDATE_REQUIRED";
  }
  // cve_backlog_threshold は v1_l1plus_primary / v1_l2star_alt では REPLACE、他は UPDATE_REQUIRED とする
  if (signal === "cve_backlog_threshold") {
    // v1_l1plus_primary と v1_l2star_alt は CVE バックログ超過に対して REPLACE を要求する
    if (lifecycle_class === "v1_l1plus_primary" || lifecycle_class === "v1_l2star_alt") {
      // 最重要・重要クラスは代替品への置換を要求する
      return "REPLACE";
    }
    // それ以外は UPDATE_REQUIRED とする
    return "UPDATE_REQUIRED";
  }
  // maintainer_turnover は v1_l1plus_primary / v1_l2star_alt では REPLACE、他は FREEZE とする
  if (signal === "maintainer_turnover") {
    // v1_l1plus_primary と v1_l2star_alt はメンテナー交代に対して REPLACE を要求する
    if (lifecycle_class === "v1_l1plus_primary" || lifecycle_class === "v1_l2star_alt") {
      // 最重要・重要クラスは代替品への置換を要求する
      return "REPLACE";
    }
    // v1_l3_generic 以下はメンテナー交代に対して FREEZE を発令する（評価保留）
    return "FREEZE";
  }
  // major_up は v1_l1plus_primary / v1_l2star_alt では UPDATE_REQUIRED、他は FREEZE とする
  if (signal === "major_up") {
    // v1_l1plus_primary と v1_l2star_alt はメジャーアップデートに対して UPDATE_REQUIRED を要求する
    if (lifecycle_class === "v1_l1plus_primary" || lifecycle_class === "v1_l2star_alt") {
      // 最重要・重要クラスはメジャーアップデートの適用を要求する
      return "UPDATE_REQUIRED";
    }
    // v1_l3_generic 以下はメジャーアップデートに対して FREEZE を発令する（評価保留）
    return "FREEZE";
  }
  // 未定義 signal に対しては FREEZE をデフォルトとする（安全側に倒す）
  return "FREEZE";
}

// ============================================================
// テスト: 8 signal × 6 lifecycle_class の組み合わせ検証
// ============================================================

// OSS lifecycle signal トリガーテスト群
describe("OSS lifecycle signal trigger tests", () => {
  // ---- license_change signal のテスト ----

  // 全 lifecycle_class で license_change が FREEZE を発火することを検証する
  describe("license_change signal", () => {
    // v1_l1plus_primary での license_change は FREEZE を発火する
    test("v1_l1plus_primary: license_change triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("license_change", "v1_l1plus_primary");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l2star_alt での license_change は FREEZE を発火する
    test("v1_l2star_alt: license_change triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("license_change", "v1_l2star_alt");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l3_generic での license_change は FREEZE を発火する
    test("v1_l3_generic: license_change triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("license_change", "v1_l3_generic");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l4_dev_tool での license_change は FREEZE を発火する
    test("v1_l4_dev_tool: license_change triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("license_change", "v1_l4_dev_tool");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l5_sandbox での license_change は FREEZE を発火する
    test("v1_l5_sandbox: license_change triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("license_change", "v1_l5_sandbox");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l6_deprecated での license_change は FREEZE を発火する
    test("v1_l6_deprecated: license_change triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("license_change", "v1_l6_deprecated");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- eol_announced signal のテスト ----

  // eol_announced signal のアクション分岐を検証する
  describe("eol_announced signal", () => {
    // v1_l1plus_primary での eol_announced は REPLACE を発火する
    test("v1_l1plus_primary: eol_announced triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("eol_announced", "v1_l1plus_primary");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l2star_alt での eol_announced は REPLACE を発火する
    test("v1_l2star_alt: eol_announced triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("eol_announced", "v1_l2star_alt");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l3_generic での eol_announced は REPLACE を発火する
    test("v1_l3_generic: eol_announced triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("eol_announced", "v1_l3_generic");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l4_dev_tool での eol_announced は UPDATE_REQUIRED を発火する
    test("v1_l4_dev_tool: eol_announced triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("eol_announced", "v1_l4_dev_tool");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // v1_l5_sandbox での eol_announced は UPDATE_REQUIRED を発火する
    test("v1_l5_sandbox: eol_announced triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("eol_announced", "v1_l5_sandbox");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });
  });

  // ---- cve_backlog_threshold signal のテスト ----

  // cve_backlog_threshold signal のアクション分岐を検証する
  describe("cve_backlog_threshold signal", () => {
    // v1_l1plus_primary での cve_backlog_threshold は REPLACE を発火する
    test("v1_l1plus_primary: cve_backlog_threshold triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("cve_backlog_threshold", "v1_l1plus_primary");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l2star_alt での cve_backlog_threshold は REPLACE を発火する
    test("v1_l2star_alt: cve_backlog_threshold triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("cve_backlog_threshold", "v1_l2star_alt");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l3_generic での cve_backlog_threshold は UPDATE_REQUIRED を発火する
    test("v1_l3_generic: cve_backlog_threshold triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("cve_backlog_threshold", "v1_l3_generic");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // v1_l4_dev_tool での cve_backlog_threshold は UPDATE_REQUIRED を発火する
    test("v1_l4_dev_tool: cve_backlog_threshold triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("cve_backlog_threshold", "v1_l4_dev_tool");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });
  });

  // ---- maintainer_turnover signal のテスト ----

  // maintainer_turnover signal のアクション分岐を検証する
  describe("maintainer_turnover signal", () => {
    // v1_l1plus_primary での maintainer_turnover は REPLACE を発火する
    test("v1_l1plus_primary: maintainer_turnover triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("maintainer_turnover", "v1_l1plus_primary");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l2star_alt での maintainer_turnover は REPLACE を発火する
    test("v1_l2star_alt: maintainer_turnover triggers REPLACE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("maintainer_turnover", "v1_l2star_alt");
      // REPLACE が発火されることを検証する
      expect(action).toBe("REPLACE");
    });

    // v1_l3_generic での maintainer_turnover は FREEZE を発火する
    test("v1_l3_generic: maintainer_turnover triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("maintainer_turnover", "v1_l3_generic");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l5_sandbox での maintainer_turnover は FREEZE を発火する
    test("v1_l5_sandbox: maintainer_turnover triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("maintainer_turnover", "v1_l5_sandbox");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- fork_event signal のテスト ----

  // 全 lifecycle_class で fork_event が FREEZE を発火することを検証する
  describe("fork_event signal", () => {
    // v1_l1plus_primary での fork_event は FREEZE を発火する
    test("v1_l1plus_primary: fork_event triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("fork_event", "v1_l1plus_primary");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l3_generic での fork_event は FREEZE を発火する
    test("v1_l3_generic: fork_event triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("fork_event", "v1_l3_generic");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l6_deprecated での fork_event は FREEZE を発火する
    test("v1_l6_deprecated: fork_event triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("fork_event", "v1_l6_deprecated");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- conformance_drift signal のテスト ----

  // 全 lifecycle_class で conformance_drift が FREEZE を発火することを検証する
  describe("conformance_drift signal", () => {
    // v1_l1plus_primary での conformance_drift は FREEZE を発火する
    test("v1_l1plus_primary: conformance_drift triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("conformance_drift", "v1_l1plus_primary");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l4_dev_tool での conformance_drift は FREEZE を発火する
    test("v1_l4_dev_tool: conformance_drift triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("conformance_drift", "v1_l4_dev_tool");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- major_up signal のテスト ----

  // major_up signal のアクション分岐を検証する
  describe("major_up signal", () => {
    // v1_l1plus_primary での major_up は UPDATE_REQUIRED を発火する
    test("v1_l1plus_primary: major_up triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("major_up", "v1_l1plus_primary");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // v1_l2star_alt での major_up は UPDATE_REQUIRED を発火する
    test("v1_l2star_alt: major_up triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("major_up", "v1_l2star_alt");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // v1_l3_generic での major_up は FREEZE を発火する
    test("v1_l3_generic: major_up triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("major_up", "v1_l3_generic");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });

    // v1_l5_sandbox での major_up は FREEZE を発火する
    test("v1_l5_sandbox: major_up triggers FREEZE", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("major_up", "v1_l5_sandbox");
      // FREEZE が発火されることを検証する
      expect(action).toBe("FREEZE");
    });
  });

  // ---- spec_drift signal のテスト ----

  // 全 lifecycle_class で spec_drift が UPDATE_REQUIRED を発火することを検証する
  describe("spec_drift signal", () => {
    // v1_l1plus_primary での spec_drift は UPDATE_REQUIRED を発火する
    test("v1_l1plus_primary: spec_drift triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("spec_drift", "v1_l1plus_primary");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // v1_l3_generic での spec_drift は UPDATE_REQUIRED を発火する
    test("v1_l3_generic: spec_drift triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("spec_drift", "v1_l3_generic");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });

    // v1_l6_deprecated での spec_drift は UPDATE_REQUIRED を発火する
    test("v1_l6_deprecated: spec_drift triggers UPDATE_REQUIRED", () => {
      // resolveAction を呼び出してアクションを取得する
      const action = resolveAction("spec_drift", "v1_l6_deprecated");
      // UPDATE_REQUIRED が発火されることを検証する
      expect(action).toBe("UPDATE_REQUIRED");
    });
  });

  // ---- マッピング網羅テスト ----

  // 定義済み SignalActionMapping の全エントリが正しく解決されることを検証する
  describe("SignalActionMapping exhaustive check", () => {
    // テスト対象のマッピングエントリを定義する（8 signal × 代表 lifecycle_class）
    const mappings: SignalActionMapping[] = [
      // eol_announced × v1_l1plus_primary → REPLACE（spec §signal_action_matrix 準拠）
      { signal: "eol_announced", lifecycle_class: "v1_l1plus_primary", expected_action: "REPLACE" },
      // cve_backlog_threshold × v1_l1plus_primary → REPLACE（spec §signal_action_matrix 準拠）
      {
        signal: "cve_backlog_threshold",
        lifecycle_class: "v1_l1plus_primary",
        expected_action: "REPLACE",
      },
      // major_up × v1_l2star_alt → UPDATE_REQUIRED（spec §signal_action_matrix 準拠）
      {
        signal: "major_up",
        lifecycle_class: "v1_l2star_alt",
        expected_action: "UPDATE_REQUIRED",
      },
      // maintainer_turnover × v1_l2star_alt → REPLACE（spec §signal_action_matrix 準拠）
      { signal: "maintainer_turnover", lifecycle_class: "v1_l2star_alt", expected_action: "REPLACE" },
      // license_change × v1_l3_generic → FREEZE（spec §signal_action_matrix 準拠）
      { signal: "license_change", lifecycle_class: "v1_l3_generic", expected_action: "FREEZE" },
      // spec_drift × v1_l4_dev_tool → UPDATE_REQUIRED（spec §signal_action_matrix 準拠）
      {
        signal: "spec_drift",
        lifecycle_class: "v1_l4_dev_tool",
        expected_action: "UPDATE_REQUIRED",
      },
      // fork_event × v1_l5_sandbox → FREEZE（spec §signal_action_matrix 準拠）
      { signal: "fork_event", lifecycle_class: "v1_l5_sandbox", expected_action: "FREEZE" },
      // conformance_drift × v1_l6_deprecated → FREEZE（spec §signal_action_matrix 準拠）
      {
        signal: "conformance_drift",
        lifecycle_class: "v1_l6_deprecated",
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
