/// NOTE code below is to be merged later
use serde::{Deserialize, Serialize};

use crate::{
    common_skeleton::{
        datafeed::event::MarketEvent,
        instrument::{kind::InstrumentKind, Instrument},
        token::Token,
    },
    simulated_exchange::load_from_clickhouse::queries_operations::ClickhouseTrade,
    Exchange,
};
use crate::common_skeleton::instrument::kind::InstrumentKind::Perpetual;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
#[allow(non_snake_case)]
pub struct WsTrade
{
    #[serde(alias = "instrument_id", alias = "inst_id")]
    instId: String, // NOTE nomenclature of instrument ID ?
    #[serde(alias = "side")]
    side: String,
    #[serde(alias = "price")]
    px: String,
    #[serde(alias = "timestamp")]
    ts: String,
}

// NOTE 这是按照Okex交易所API数据类型构建的 WebsocketTrade 数据结构，回测选用。
impl MarketEvent<WsTrade>
{
    pub fn from_ws_trade(ws_trade: WsTrade, base: String, quote: String, instrument: InstrumentKind, exchange: Exchange) -> Self
    {
        let exchange_time = ws_trade.ts.parse::<i64>().unwrap_or(0);
        let received_time = ws_trade.ts.parse::<i64>().unwrap_or(0); // NOTE 注意这是不对的 应该加上一个标准化的随机延迟。

        let instrument = Instrument { base: Token::from(base),
                                      quote: Token::from(quote),
                                      kind: instrument };

        MarketEvent { exchange_time,
                      received_time,
                      exchange: Exchange(exchange.to_string()),

                      instrument,
                      kind: ws_trade }
    }
}

// NOTE 这是按照Clickhouse中存储的数据类型构建的 WebsocketTrade 数据结构，回测选用。
impl MarketEvent<ClickhouseTrade>
{
    pub fn from_swap_trade_clickhouse(trade: ClickhouseTrade, base: String, quote: String, exchange: Exchange) -> Self
    {
        let exchange_time = trade.timestamp;
        let received_time = trade.timestamp; // NOTE 注意这是不对的 应该加上一个标准化的随机延迟。

        let instrument = Instrument { base: Token::from(base),
                                      quote: Token::from(quote),
                                      kind: Perpetual };

        MarketEvent { exchange_time,
                      received_time,
                      exchange: Exchange(exchange.to_string()),
                      instrument,
                      kind: trade }
    }
}

// 从 TradeDataFromClickhouse 到 WsTrade 的转换实现
impl From<ClickhouseTrade> for WsTrade
{
    fn from(trade: ClickhouseTrade) -> Self
    {
        WsTrade { instId: trade.basequote,
                  side: trade.side,
                  px: trade.price.to_string(),
                  ts: trade.timestamp.to_string() }
    }
}

pub fn parse_base_and_quote(basequote: &str) -> (String, String)
{
    let quote_assets = ["USDT", "USDC","USD","UST","DAI","FDUSD"];
    for &quote in &quote_assets {
        if basequote.ends_with(quote) {
            let base = &basequote[..basequote.len() - quote.len()];
            return (base.to_string(), quote.to_string());
        }
    }
    (basequote.to_string(), String::new()) // 如果无法解析，返回原始值
}

#[allow(dead_code)]
impl WsTrade
{
    pub(crate) fn from_ref(data: &ClickhouseTrade) -> Self
    {
        WsTrade { // 这里假设 WsTrade 结构体字段和 TradeDataFromClickhouse 结构体字段对应
                  instId: data.basequote.clone(),
                  side: data.side.clone(),
                  px: data.price.to_string(),
                  ts: data.timestamp.to_string() }
    }
}
