use std::fmt::Formatter;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common_infrastructure::{
        balance::TokenBalance,
        order::{ Order},
        position::AccountPositions,
        trade::ClientTrade,
    },
    sandbox::account::account_config::AccountConfig,
    Exchange,
};
use crate::common_infrastructure::order::states::cancelled::Cancelled;
use crate::common_infrastructure::order::states::fills::{FullyFill, PartialFill};
use crate::common_infrastructure::order::states::open::Open;

/// NOTE: 如果需要记录交易所的时间戳，可以再添加一个专门的字段来表示交易所的时间，例如：    pub exchange_ts: DateTime<Utc> or i64
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AccountEvent
{
    pub exchange_timestamp: i64, // 交易所发送事件的时间,
    pub exchange: Exchange,      // 目标和源头交易所
    pub kind: AccountEventKind,  // 事件类型
}

/// 定义账户事件[`AccountEvent`]的类型。
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum AccountEventKind
{
    // Order Events
    OrdersOpen(Vec<Order<Open>>),
    OrdersNew(Vec<Order<Open>>),
    OrdersCancelled(Vec<Order<Cancelled>>),
    OrdersFilled(Vec<Order<FullyFill>>),
    OrdersPartiallyFilled(Vec<Order<PartialFill>>),
    Balance(TokenBalance),
    Trade(ClientTrade),
    Balances(Vec<TokenBalance>),
    Positions(AccountPositions),
    AccountConfig(AccountConfig),
    // OrderBookUpdate(OrderBookUpdate),
    // MarketStatus(MarketStatus),
    // MarginUpdate(MarginUpdate),
    // Transfer(Transfer),
    // Deposit(Deposit),
    // Withdrawal(Withdrawal),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct ClientOrderId(pub Uuid); // 客户端订单ID结构

// 为ClientOrderId实现格式化显示
impl std::fmt::Display for ClientOrderId
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::common_infrastructure::{balance::Balance, token::Token};
    use uuid::Uuid;

    #[test]
    fn account_event_should_serialize_and_deserialize_correctly()
    {
        let event = AccountEvent { exchange_timestamp: 1627845123,
                                   exchange: Exchange::Binance,
                                   kind: AccountEventKind::OrdersOpen(vec![]) };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: AccountEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn account_event_kind_should_serialize_and_deserialize_correctly()
    {
        let kind = AccountEventKind::OrdersNew(vec![]);
        let serialized = serde_json::to_string(&kind).unwrap();
        let deserialized: AccountEventKind = serde_json::from_str(&serialized).unwrap();
        assert_eq!(kind, deserialized);
    }

    #[test]
    fn client_order_id_should_format_correctly()
    {
        let uuid = Uuid::new_v4();
        let client_order_id = ClientOrderId(uuid);
        assert_eq!(format!("{}", client_order_id), uuid.to_string());
    }

    #[test]
    fn account_event_kind_should_handle_all_variants()
    {
        let kinds = vec![AccountEventKind::OrdersOpen(vec![]),
                         AccountEventKind::OrdersNew(vec![]),
                         AccountEventKind::OrdersCancelled(vec![]),
                         AccountEventKind::OrdersFilled(vec![]),
                         AccountEventKind::OrdersPartiallyFilled(vec![]),
                         AccountEventKind::Balance(TokenBalance::new(Token::from("BTC"), Balance::new(100.0, 50.0, 20000.0))),
                         // AccountEventKind::Trade(ClientTrade::default()),
                         AccountEventKind::Balances(vec![]),
                         /* AccountEventKind::Positions(AccountPositions::default()),
                          * AccountEventKind::AccountConfig(AccountConfig::default()), */];
        for kind in kinds {
            let serialized = serde_json::to_string(&kind).unwrap();
            let deserialized: AccountEventKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(kind, deserialized);
        }
    }
}
