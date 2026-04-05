use std::num::NonZeroUsize;
// L-002 監査対応: async コンテキスト内での std::sync::Mutex によるデッドロックリスクを排除する
// tokio::sync::Mutex を使用することで、非同期タスクが Mutex のロック待ちで
// tokio のスレッドをブロックしないようにする
use tokio::sync::Mutex;

use lru::LruCache;

use crate::domain::entity::condition::{Combinator, ConditionNode, Operator};

/// 条件評価の最大再帰深度。これを超えるとスタックオーバーフローを引き起こす可能性があるため制限する
const MAX_EVALUATION_DEPTH: usize = 32;

/// 正規表現コンパイル結果をキャッシュする構造体
/// RUST-HIGH-003 対応: 毎回コンパイルする代わりにキャッシュを利用し ReDoS リスクを軽減する
/// L-002 監査対応: regex_cache を tokio::sync::Mutex で保護する
pub struct ConditionEvaluator {
    regex_cache: Mutex<LruCache<String, regex::Regex>>,
}

impl ConditionEvaluator {
    /// ConditionEvaluator を生成する。
    /// キャッシュサイズは 256 パターン（メモリ効率とヒット率のバランス）
    pub fn new() -> Self {
        Self {
            regex_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(256).expect("256 is non-zero"),
            )),
        }
    }

    /// 条件ノードを評価する（外部向けエントリーポイント）
    /// L-002 監査対応: tokio::sync::Mutex の .lock().await が必要なため async fn にする
    pub async fn evaluate(
        &self,
        condition: &ConditionNode,
        context: &serde_json::Value,
    ) -> Result<bool, String> {
        self.evaluate_inner(condition, context, 0).await
    }

    /// 条件ノードを評価する（再帰深度を追跡する内部実装）
    /// L-002 監査対応: tokio::sync::Mutex の .lock().await を使うため async 再帰が必要。
    /// Rust の async fn は再帰できないため Box::pin でヒープ確保し再帰を実現する
    #[allow(clippy::manual_async_fn)]
    fn evaluate_inner<'a>(
        &'a self,
        condition: &'a ConditionNode,
        context: &'a serde_json::Value,
        depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send + 'a>> {
        Box::pin(async move {
            // 再帰深度が上限を超えた場合はエラーを返し、スタックオーバーフローを防止する
            if depth > MAX_EVALUATION_DEPTH {
                return Err(format!(
                    "条件評価の再帰深度が上限（{}）を超えました。ルールのネストが深すぎます",
                    MAX_EVALUATION_DEPTH
                ));
            }

            if let Some(ref combinator) = condition.combinator {
                let children = condition
                    .children
                    .as_ref()
                    .ok_or_else(|| "combinator node must have children".to_string())?;

                match combinator {
                    // All コンビネータ: 全子ノードが true の場合のみ true を返す
                    Combinator::All => {
                        for child in children {
                            // L-002 監査対応: 再帰呼び出しは async なので .await が必要
                            if !self.evaluate_inner(child, context, depth + 1).await? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    // Any コンビネータ: いずれか1つの子ノードが true なら true を返す
                    Combinator::Any => {
                        for child in children {
                            // L-002 監査対応: 再帰呼び出しは async なので .await が必要
                            if self.evaluate_inner(child, context, depth + 1).await? {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    }
                    // None コンビネータ: 全子ノードが false の場合のみ true を返す
                    Combinator::None => {
                        for child in children {
                            // L-002 監査対応: 再帰呼び出しは async なので .await が必要
                            if self.evaluate_inner(child, context, depth + 1).await? {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                }
            } else {
                // リーフ条件: フィールド値とオペレータで評価する
                let field = condition
                    .field
                    .as_ref()
                    .ok_or_else(|| "leaf condition must have 'field'".to_string())?;
                let operator = condition
                    .operator
                    .as_ref()
                    .ok_or_else(|| "leaf condition must have 'operator'".to_string())?;

                let actual = Self::resolve_field(context, field);
                let expected = condition.value.as_ref();

                // L-002 監査対応: evaluate_operator は regex の場合に .await が必要
                self.evaluate_operator(operator, actual, expected).await
            }
        })
    }

    /// コンテキストからドット記法のフィールドパスを解決する
    fn resolve_field<'a>(
        context: &'a serde_json::Value,
        path: &str,
    ) -> Option<&'a serde_json::Value> {
        let mut current = context;
        for part in path.split('.') {
            current = current.get(part)?;
        }
        Some(current)
    }

    /// 演算子に応じて条件を評価する
    /// L-002 監査対応: Regex 評価で tokio::sync::Mutex の .lock().await が必要なため async にする
    async fn evaluate_operator(
        &self,
        operator: &Operator,
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        match operator {
            Operator::Eq => Ok(actual == expected),
            Operator::Ne => Ok(actual != expected),
            Operator::Gt => Self::compare_numeric(actual, expected, |a, b| a > b),
            Operator::Gte => Self::compare_numeric(actual, expected, |a, b| a >= b),
            Operator::Lt => Self::compare_numeric(actual, expected, |a, b| a < b),
            Operator::Lte => Self::compare_numeric(actual, expected, |a, b| a <= b),
            Operator::In => Self::evaluate_in(actual, expected),
            Operator::NotIn => Self::evaluate_in(actual, expected).map(|v| !v),
            Operator::Contains => Self::evaluate_contains(actual, expected),
            // L-002 監査対応: tokio::sync::Mutex の .await が必要なため .await を追加する
            Operator::Regex => self.evaluate_regex(actual, expected).await,
        }
    }

    /// 数値比較を行う汎用ヘルパー
    fn compare_numeric(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
        cmp: fn(f64, f64) -> bool,
    ) -> Result<bool, String> {
        let a = actual
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "actual value is not a number".to_string())?;
        let b = expected
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "expected value is not a number".to_string())?;
        Ok(cmp(a, b))
    }

    /// In 演算子: 期待値配列に実際値が含まれるかを評価する
    fn evaluate_in(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        let actual_val =
            actual.ok_or_else(|| "actual value is null for 'in' operator".to_string())?;
        let arr = expected
            .and_then(|v| v.as_array())
            .ok_or_else(|| "expected value for 'in' operator must be an array".to_string())?;
        Ok(arr.contains(actual_val))
    }

    /// Contains 演算子: 文字列が部分文字列を含むかを評価する
    fn evaluate_contains(
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        let haystack = actual
            .and_then(|v| v.as_str())
            .ok_or_else(|| "actual value is not a string for 'contains'".to_string())?;
        let needle = expected
            .and_then(|v| v.as_str())
            .ok_or_else(|| "expected value is not a string for 'contains'".to_string())?;
        Ok(haystack.contains(needle))
    }

    /// 正規表現マッチングを行う。LruCache でコンパイル済みパターンを再利用する。
    /// ReDoS 緩和: 長大なパターン（1024 文字超）を拒否する
    /// L-002 監査対応: tokio::sync::Mutex の .lock().await を使用するため async fn にする
    async fn evaluate_regex(
        &self,
        actual: Option<&serde_json::Value>,
        expected: Option<&serde_json::Value>,
    ) -> Result<bool, String> {
        let text = actual
            .and_then(|v| v.as_str())
            .ok_or_else(|| "actual value is not a string for 'regex'".to_string())?;
        let pattern = expected
            .and_then(|v| v.as_str())
            .ok_or_else(|| "expected value is not a string for 'regex'".to_string())?;

        // ReDoS 緩和: 長大なパターンを拒否する（1024 文字を上限とする）
        if pattern.len() > 1024 {
            return Err(format!(
                "regex pattern too long: {} chars (max 1024)",
                pattern.len()
            ));
        }

        // L-002 監査対応: tokio::sync::Mutex は .lock() が async なため .await が必要
        // std::sync::Mutex と異なり、ロック取得中に .await ポイントを跨いでも安全
        let mut cache = self.regex_cache.lock().await;

        if let Some(re) = cache.get(pattern) {
            return Ok(re.is_match(text));
        }

        // キャッシュミス: 正規表現をコンパイルしてキャッシュに追加する
        let re = regex::Regex::new(pattern)
            .map_err(|e| format!("invalid regex pattern '{}': {}", pattern, e))?;
        let result = re.is_match(text);
        cache.put(pattern.to_string(), re);
        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::service::condition_parser::ConditionParser;

    // L-002 監査対応: evaluate が async になったため、テストヘルパーも async にする
    async fn eval(condition_json: serde_json::Value, context: serde_json::Value) -> bool {
        let node = ConditionParser::parse(&condition_json).unwrap();
        let evaluator = ConditionEvaluator::new();
        evaluator.evaluate(&node, &context).await.unwrap()
    }

    #[tokio::test]
    async fn test_eq_match() {
        assert!(eval(
            serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
            serde_json::json!({"status": "active"}),
        ).await);
    }

    #[tokio::test]
    async fn test_eq_no_match() {
        assert!(!eval(
            serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
            serde_json::json!({"status": "inactive"}),
        ).await);
    }

    #[tokio::test]
    async fn test_ne() {
        assert!(eval(
            serde_json::json!({"field": "status", "operator": "ne", "value": "deleted"}),
            serde_json::json!({"status": "active"}),
        ).await);
    }

    #[tokio::test]
    async fn test_gt() {
        assert!(eval(
            serde_json::json!({"field": "amount", "operator": "gt", "value": 100}),
            serde_json::json!({"amount": 200}),
        ).await);
    }

    #[tokio::test]
    async fn test_gte() {
        assert!(eval(
            serde_json::json!({"field": "amount", "operator": "gte", "value": 100}),
            serde_json::json!({"amount": 100}),
        ).await);
    }

    #[tokio::test]
    async fn test_lt() {
        assert!(eval(
            serde_json::json!({"field": "score", "operator": "lt", "value": 50}),
            serde_json::json!({"score": 30}),
        ).await);
    }

    #[tokio::test]
    async fn test_lte() {
        assert!(eval(
            serde_json::json!({"field": "score", "operator": "lte", "value": 50}),
            serde_json::json!({"score": 50}),
        ).await);
    }

    #[tokio::test]
    async fn test_in() {
        assert!(eval(
            serde_json::json!({"field": "region", "operator": "in", "value": ["JP", "US"]}),
            serde_json::json!({"region": "JP"}),
        ).await);
    }

    #[tokio::test]
    async fn test_not_in() {
        assert!(eval(
            serde_json::json!({"field": "region", "operator": "not_in", "value": ["JP", "US"]}),
            serde_json::json!({"region": "UK"}),
        ).await);
    }

    #[tokio::test]
    async fn test_contains() {
        assert!(eval(
            serde_json::json!({"field": "name", "operator": "contains", "value": "special"}),
            serde_json::json!({"name": "this is special item"}),
        ).await);
    }

    #[tokio::test]
    async fn test_regex() {
        assert!(eval(
            serde_json::json!({"field": "code", "operator": "regex", "value": "^TAX-\\d{4}$"}),
            serde_json::json!({"code": "TAX-1234"}),
        ).await);
    }

    /// 正規表現キャッシュが同じパターンで再利用されることを確認する
    #[tokio::test]
    async fn test_regex_cache_reuse() {
        let evaluator = ConditionEvaluator::new();
        let node = ConditionParser::parse(
            &serde_json::json!({"field": "code", "operator": "regex", "value": "^TAX-\\d{4}$"}),
        )
        .unwrap();
        let context = serde_json::json!({"code": "TAX-1234"});
        // 2回評価してもキャッシュが壊れないことを確認する
        assert!(evaluator.evaluate(&node, &context).await.unwrap());
        assert!(evaluator.evaluate(&node, &context).await.unwrap());
    }

    /// 長大な正規表現パターンが拒否されることを確認する（ReDoS 緩和）
    #[tokio::test]
    async fn test_regex_too_long_pattern_rejected() {
        let evaluator = ConditionEvaluator::new();
        let long_pattern = "a".repeat(1025);
        let node = ConditionParser::parse(
            &serde_json::json!({"field": "text", "operator": "regex", "value": long_pattern}),
        )
        .unwrap();
        let context = serde_json::json!({"text": "aaa"});
        let result = evaluator.evaluate(&node, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too long"));
    }

    #[tokio::test]
    async fn test_nested_field_dot_notation() {
        assert!(eval(
            serde_json::json!({"field": "item.category", "operator": "eq", "value": "food"}),
            serde_json::json!({"item": {"category": "food"}}),
        ).await);
    }

    #[tokio::test]
    async fn test_all_combinator() {
        assert!(eval(
            serde_json::json!({
                "all": [
                    {"field": "item.category", "operator": "eq", "value": "food"},
                    {"field": "item.is_takeout", "operator": "eq", "value": true}
                ]
            }),
            serde_json::json!({"item": {"category": "food", "is_takeout": true}}),
        ).await);
    }

    #[tokio::test]
    async fn test_all_combinator_fail() {
        assert!(!eval(
            serde_json::json!({
                "all": [
                    {"field": "item.category", "operator": "eq", "value": "food"},
                    {"field": "item.is_takeout", "operator": "eq", "value": true}
                ]
            }),
            serde_json::json!({"item": {"category": "food", "is_takeout": false}}),
        ).await);
    }

    #[tokio::test]
    async fn test_any_combinator() {
        assert!(eval(
            serde_json::json!({
                "any": [
                    {"field": "status", "operator": "eq", "value": "active"},
                    {"field": "status", "operator": "eq", "value": "pending"}
                ]
            }),
            serde_json::json!({"status": "pending"}),
        ).await);
    }

    #[tokio::test]
    async fn test_none_combinator() {
        assert!(eval(
            serde_json::json!({
                "none": [
                    {"field": "status", "operator": "eq", "value": "deleted"},
                    {"field": "status", "operator": "eq", "value": "banned"}
                ]
            }),
            serde_json::json!({"status": "active"}),
        ).await);
    }

    #[tokio::test]
    async fn test_missing_field_returns_none() {
        assert!(!eval(
            serde_json::json!({"field": "nonexistent", "operator": "eq", "value": "x"}),
            serde_json::json!({"status": "active"}),
        ).await);
    }

    /// 33段以上のネストでエラーになることを検証する（スタックオーバーフロー防止テスト）
    #[tokio::test]
    async fn evaluate_depth_limit_prevents_stack_overflow() {
        // MAX_EVALUATION_DEPTH(32) を超える34段ネストの条件ツリーを構築する
        let leaf = ConditionNode {
            combinator: None,
            children: None,
            field: Some("status".to_string()),
            operator: Some(Operator::Eq),
            value: Some(serde_json::json!("active")),
        };
        // リーフノードを34段の All コンビネータでラップする
        let mut node = leaf;
        for _ in 0..34 {
            node = ConditionNode {
                combinator: Some(Combinator::All),
                children: Some(vec![node]),
                field: None,
                operator: None,
                value: None,
            };
        }
        let context = serde_json::json!({"status": "active"});
        let evaluator = ConditionEvaluator::new();
        let result = evaluator.evaluate(&node, &context).await;
        assert!(
            result.is_err(),
            "深度制限超過時はエラーが返される必要があります"
        );
        assert!(result.unwrap_err().contains("再帰深度"));
    }

    /// 最大深度以内のネストは正常に評価されることを検証する
    #[tokio::test]
    async fn evaluate_within_depth_limit_succeeds() {
        // MAX_EVALUATION_DEPTH(32) 以内の30段ネストは成功する
        let leaf = ConditionNode {
            combinator: None,
            children: None,
            field: Some("status".to_string()),
            operator: Some(Operator::Eq),
            value: Some(serde_json::json!("active")),
        };
        let mut node = leaf;
        for _ in 0..30 {
            node = ConditionNode {
                combinator: Some(Combinator::All),
                children: Some(vec![node]),
                field: None,
                operator: None,
                value: None,
            };
        }
        let context = serde_json::json!({"status": "active"});
        let evaluator = ConditionEvaluator::new();
        let result = evaluator.evaluate(&node, &context).await;
        assert!(
            result.is_ok(),
            "深度制限以内のネストは正常に評価される必要があります"
        );
        assert!(result.unwrap());
    }
}
