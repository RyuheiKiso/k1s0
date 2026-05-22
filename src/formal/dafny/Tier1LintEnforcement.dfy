// k1s0-proof: PROOF-tier1-prog-001 -> IMPL-tier1-0010
// Tier1LintEnforcement.dfy
// tier1 lint 強制機構 program correctness: 禁止 API 使用が必ず検出されることを証明
// obligation_id: tier1_lint_enforcement_pcp
// property: BannedApiAlwaysDetected — 禁止シンボルを含むコードは必ず lint エラーとなる

// 禁止シンボルの集合（型安全に表現）
datatype Symbol = BannedApi(name: string) | AllowedApi(name: string)

// ソースコード内のシンボル使用を表現
datatype Usage = Usage(symbol: Symbol, location: string)

// lint 結果
datatype LintResult = Pass | Fail(violations: seq<Usage>)

// lint エンジンの検査関数
function CheckUsages(usages: seq<Usage>): LintResult {
    var violations := FindViolations(usages, []);
    if |violations| == 0 then Pass else Fail(violations)
}

// 違反のみを抽出する（全 Usage を走査して BannedApi のみ返す）
function FindViolations(usages: seq<Usage>, acc: seq<Usage>): seq<Usage>
    decreases |usages|
{
    if |usages| == 0 then acc
    else
        var head := usages[0];
        var rest := usages[1..];
        match head.symbol {
            case BannedApi(_) => FindViolations(rest, acc + [head])
            case AllowedApi(_) => FindViolations(rest, acc)
        }
}

// BannedApiAlwaysDetected: BannedApi を含む Usage リストは必ず Fail になる
lemma BannedApiAlwaysDetected(usages: seq<Usage>, banned: Usage)
    requires banned in usages
    requires banned.symbol.BannedApi?
    ensures CheckUsages(usages).Fail?
{
    // FindViolations は banned を violations に含めるので、結果は Fail
    var violations := FindViolations(usages, []);
    assert banned in violations by {
        FindViolationsContainsBanned(usages, [], banned);
    }
    assert |violations| > 0;
}

// 補助補題: FindViolations は BannedApi を必ず violations に含める
lemma FindViolationsContainsBanned(usages: seq<Usage>, acc: seq<Usage>, banned: Usage)
    requires banned in usages
    requires banned.symbol.BannedApi?
    ensures banned in FindViolations(usages, acc)
    decreases |usages|
{
    if usages[0] == banned {
        // head が banned の場合、acc に追加されて伝播する
    } else {
        // 再帰ケース
        FindViolationsContainsBanned(usages[1..],
            if usages[0].symbol.BannedApi? then acc + [usages[0]] else acc,
            banned);
    }
}
