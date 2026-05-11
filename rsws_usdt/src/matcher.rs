//! 订单金额匹配器

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// 匹配策略
#[derive(Debug, Clone, Copy)]
pub enum MatchStrategy {
    /// 精确匹配: 支付金额 == 订单金额
    Exact,

    /// 范围匹配: 订单金额 - 容差 <= 支付金额 <= 订单金额 + 容差
    Range { tolerance: Decimal },

    /// 唯一小数位: 订单金额 = 基础金额 + 唯一标识小数位
    /// 例如: 10.001, 10.002, 10.003...
    UniqueDecimal {
        /// 基础金额
        base_amount: Decimal,
        /// 小数位数 (如 3 表示精确到 0.001)
        decimal_places: u32,
    },
}

/// 待匹配的订单信息
#[derive(Debug, Clone)]
pub struct PendingOrder {
    /// 订单 ID
    pub order_id: i64,

    /// 用户 ID
    pub user_id: i64,

    /// 订单金额
    pub amount: Decimal,

    /// 收款地址
    pub wallet_address: String,

    /// 网络类型
    pub network: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// 匹配结果
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// 是否匹配成功
    pub matched: bool,

    /// 匹配的订单 ID
    pub order_id: Option<i64>,

    /// 匹配金额
    pub matched_amount: Option<Decimal>,

    /// 匹配类型
    pub match_type: MatchType,
}

#[derive(Debug, Clone)]
pub enum MatchType {
    /// 精确匹配
    Exact,

    /// 范围匹配
    Range,

    /// 唯一小数位匹配
    UniqueDecimal,

    /// 未匹配
    None,
}

/// 订单匹配器
pub struct OrderMatcher {
    strategy: MatchStrategy,
}

impl OrderMatcher {
    /// 创建新匹配器
    pub fn new(strategy: MatchStrategy) -> Self {
        Self { strategy }
    }

    /// 创建默认匹配器 (精确匹配)
    pub fn exact() -> Self {
        Self::new(MatchStrategy::Exact)
    }

    /// 创建范围匹配器
    pub fn range(tolerance: Decimal) -> Self {
        Self::new(MatchStrategy::Range { tolerance })
    }

    /// 创建唯一小数位匹配器
    pub fn unique_decimal(base_amount: Decimal, decimal_places: u32) -> Self {
        Self::new(MatchStrategy::UniqueDecimal {
            base_amount,
            decimal_places,
        })
    }

    /// 匹配交易金额到订单
    ///
    /// # 参数
    /// - `tx_amount`: 交易金额
    /// - `tx_to`: 交易接收地址
    /// - `orders`: 待匹配订单列表
    ///
    /// # 返回
    /// 匹配结果
    pub fn match_order(
        &self,
        tx_amount: Decimal,
        tx_to: &str,
        orders: &[PendingOrder],
    ) -> MatchResult {
        // 先筛选收款地址匹配的订单
        let matching_orders: Vec<_> = orders
            .iter()
            .filter(|o| o.wallet_address.eq_ignore_ascii_case(tx_to))
            .collect();

        if matching_orders.is_empty() {
            return MatchResult {
                matched: false,
                order_id: None,
                matched_amount: None,
                match_type: MatchType::None,
            };
        }

        // 根据策略匹配
        match &self.strategy {
            MatchStrategy::Exact => {
                // 精确匹配
                for order in matching_orders {
                    if order.amount == tx_amount {
                        return MatchResult {
                            matched: true,
                            order_id: Some(order.order_id),
                            matched_amount: Some(tx_amount),
                            match_type: MatchType::Exact,
                        };
                    }
                }
            }
            MatchStrategy::Range { tolerance } => {
                // 范围匹配
                for order in matching_orders {
                    let min = order.amount - tolerance;
                    let max = order.amount + tolerance;
                    if tx_amount >= min && tx_amount <= max {
                        return MatchResult {
                            matched: true,
                            order_id: Some(order.order_id),
                            matched_amount: Some(tx_amount),
                            match_type: MatchType::Range,
                        };
                    }
                }
            }
            MatchStrategy::UniqueDecimal {
                base_amount,
                decimal_places,
            } => {
                // 唯一小数位匹配
                // 提取交易金额的小数部分
                let decimal_part = tx_amount - base_amount;
                let multiplier = Decimal::from(10u64.pow(*decimal_places));
                let unique_id = (decimal_part * multiplier).round();

                // 检查是否为有效的唯一标识 (整数且 > 0)
                if unique_id > Decimal::ZERO && unique_id.fract() == Decimal::ZERO {
                    // 查找对应订单
                    for order in matching_orders {
                        let order_decimal = order.amount - base_amount;
                        let order_unique = (order_decimal * multiplier).round();
                        if order_unique == unique_id {
                            return MatchResult {
                                matched: true,
                                order_id: Some(order.order_id),
                                matched_amount: Some(tx_amount),
                                match_type: MatchType::UniqueDecimal,
                            };
                        }
                    }
                }
            }
        }

        MatchResult {
            matched: false,
            order_id: None,
            matched_amount: None,
            match_type: MatchType::None,
        }
    }

    /// 为订单生成唯一支付金额
    ///
    /// 使用唯一小数位策略时，为每个订单生成唯一金额
    pub fn generate_unique_amount(
        &self,
        order_id: i64,
        base_amount: Decimal,
        decimal_places: u32,
    ) -> Decimal {
        let multiplier = Decimal::from(10u64.pow(decimal_places));
        let unique_part = Decimal::from(order_id % 1000 + 1) / multiplier; // 使用订单 ID 后 3 位
        base_amount + unique_part
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let matcher = OrderMatcher::exact();
        let orders = vec![PendingOrder {
            order_id: 1,
            user_id: 1,
            amount: Decimal::from(10),
            wallet_address: "T123".to_string(),
            network: "tron".to_string(),
            created_at: Utc::now(),
            expires_at: None,
        }];

        let result = matcher.match_order(Decimal::from(10), "T123", &orders);

        assert!(result.matched);
        assert_eq!(result.order_id, Some(1));
    }

    #[test]
    fn test_range_match() {
        let matcher = OrderMatcher::range(Decimal::new(1, 1)); // 容差 0.1
        let orders = vec![PendingOrder {
            order_id: 1,
            user_id: 1,
            amount: Decimal::from(10),
            wallet_address: "T123".to_string(),
            network: "tron".to_string(),
            created_at: Utc::now(),
            expires_at: None,
        }];

        let result = matcher.match_order(
            Decimal::new(1005, 2), // 10.05
            "T123",
            &orders,
        );

        assert!(result.matched);
    }
}
