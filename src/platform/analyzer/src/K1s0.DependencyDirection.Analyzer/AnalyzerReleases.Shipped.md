## Release 0.1.0

### New Rules

Rule ID        | Category            | Severity | Notes
---------------|---------------------|----------|------
K1S0DEPDIR0001 | DependencyDirection | Error    | SDK は tier2 を参照不可（IMP-DIR-ROOT-002）
K1S0DEPDIR0002 | DependencyDirection | Error    | SDK は tier3 を参照不可（IMP-DIR-ROOT-002）
K1S0DEPDIR0003 | DependencyDirection | Error    | tier2 は tier3 を参照不可（IMP-DIR-ROOT-002）
K1S0DEPDIR0004 | DependencyDirection | Error    | tier1 は上位層（sdk/tier2/tier3）を参照不可（IMP-DIR-ROOT-002）
