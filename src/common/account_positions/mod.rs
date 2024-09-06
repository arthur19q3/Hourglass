use std::hash::Hash;
use crate::{
    common::{
        account_positions::{
            future::FuturePosition,
            leveraged_token::LeveragedTokenPosition,
            option::OptionPosition,
            perpetual::{PerpetualPosition, PerpetualPositionBuilder, PerpetualPositionConfig},
            position_id::PositionId,
            position_meta::PositionMetaBuilder,
        },
        balance::{Balance, TokenBalance},
        instrument::{kind::InstrumentKind, Instrument},
        trade::ClientTrade,
        Side,
    },
    error::ExchangeError,
    sandbox::account::account_config::AccountConfig,
    Exchange,
};
use chrono::Utc;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod future;
pub(crate) mod leveraged_token;
pub(crate) mod option;
pub mod perpetual;
pub mod position_id;
pub mod position_meta;

#[derive(Clone, Debug)]
pub struct AccountPositions
{
    pub margin_pos_long: Arc<RwLock<HashMap<Instrument, LeveragedTokenPosition>>>,
    pub margin_pos_short: Arc<RwLock<HashMap<Instrument, LeveragedTokenPosition>>>,
    pub perpetual_pos_long: Arc<RwLock<HashMap<Instrument, PerpetualPosition>>>,
    pub perpetual_pos_short: Arc<RwLock<HashMap<Instrument, PerpetualPosition>>>,
    pub futures_pos_long: Arc<RwLock<HashMap<Instrument, FuturePosition>>>,
    pub futures_pos_short: Arc<RwLock<HashMap<Instrument, FuturePosition>>>,
    pub option_pos_long_call: Arc<RwLock<HashMap<Instrument, OptionPosition>>>,
    pub option_pos_long_put: Arc<RwLock<HashMap<Instrument, OptionPosition>>>,
    pub option_pos_short_call: Arc<RwLock<HashMap<Instrument, OptionPosition>>>,
    pub option_pos_short_put: Arc<RwLock<HashMap<Instrument, OptionPosition>>>,
}

impl Serialize for AccountPositions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Helper function to convert Arc<RwLock<HashMap<K, V>>> to HashMap<K, V>
        fn to_map<K, V>(positions: &Arc<RwLock<HashMap<K, V>>>) -> HashMap<K, V>
        where
            K: Clone + Eq + std::hash::Hash,
            V: Clone,
        {
            let positions_read = positions.blocking_read();
            positions_read.clone()
        }

        // Serialize all fields
        let mut state = serializer.serialize_struct("AccountPositions", 10)?;
        state.serialize_field("margin_pos_long", &to_map(&self.margin_pos_long))?;
        state.serialize_field("margin_pos_short", &to_map(&self.margin_pos_short))?;
        state.serialize_field("perpetual_pos_long", &to_map(&self.perpetual_pos_long))?;
        state.serialize_field("perpetual_pos_short", &to_map(&self.perpetual_pos_short))?;
        state.serialize_field("futures_pos_long", &to_map(&self.futures_pos_long))?;
        state.serialize_field("futures_pos_short", &to_map(&self.futures_pos_short))?;
        state.serialize_field("option_pos_long_call", &to_map(&self.option_pos_long_call))?;
        state.serialize_field("option_pos_long_put", &to_map(&self.option_pos_long_put))?;
        state.serialize_field("option_pos_short_call", &to_map(&self.option_pos_short_call))?;
        state.serialize_field("option_pos_short_put", &to_map(&self.option_pos_short_put))?;
        state.end()
    }
}

// Manually implement PartialEq for AccountPositions
impl PartialEq for AccountPositions {
    fn eq(&self, other: &Self) -> bool {
        fn hashmap_eq<K, V>(a: &Arc<RwLock<HashMap<K, V>>>, b: &Arc<RwLock<HashMap<K, V>>>) -> bool
        where
            K: Eq + Hash + Clone,
            V: PartialEq + Clone,
        {
            let a_read = a.blocking_read();
            let b_read = b.blocking_read();

            let a_map: HashMap<K, V> = a_read.clone();
            let b_map: HashMap<K, V> = b_read.clone();

            a_map == b_map
        }

        hashmap_eq(&self.margin_pos_long, &other.margin_pos_long)
            && hashmap_eq(&self.margin_pos_short, &other.margin_pos_short)
            && hashmap_eq(&self.perpetual_pos_long, &other.perpetual_pos_long)
            && hashmap_eq(&self.perpetual_pos_short, &other.perpetual_pos_short)
            && hashmap_eq(&self.futures_pos_long, &other.futures_pos_long)
            && hashmap_eq(&self.futures_pos_short, &other.futures_pos_short)
            && hashmap_eq(&self.option_pos_long_call, &other.option_pos_long_call)
            && hashmap_eq(&self.option_pos_long_put, &other.option_pos_long_put)
            && hashmap_eq(&self.option_pos_short_call, &other.option_pos_short_call)
            && hashmap_eq(&self.option_pos_short_put, &other.option_pos_short_put)
    }
}

impl<'de> Deserialize<'de> for AccountPositions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AccountPositionsData {
            margin_pos_long: HashMap<Instrument, LeveragedTokenPosition>,
            margin_pos_short: HashMap<Instrument, LeveragedTokenPosition>,
            perpetual_pos_long: HashMap<Instrument, PerpetualPosition>,
            perpetual_pos_short: HashMap<Instrument, PerpetualPosition>,
            futures_pos_long: HashMap<Instrument, FuturePosition>,
            futures_pos_short: HashMap<Instrument, FuturePosition>,
            option_pos_long_call: HashMap<Instrument, OptionPosition>,
            option_pos_long_put: HashMap<Instrument, OptionPosition>,
            option_pos_short_call: HashMap<Instrument, OptionPosition>,
            option_pos_short_put: HashMap<Instrument, OptionPosition>,
        }

        let data = AccountPositionsData::deserialize(deserializer)?;

        Ok(AccountPositions {
            margin_pos_long: Arc::new(RwLock::new(data.margin_pos_long)),
            margin_pos_short: Arc::new(RwLock::new(data.margin_pos_short)),
            perpetual_pos_long: Arc::new(RwLock::new(data.perpetual_pos_long)),
            perpetual_pos_short: Arc::new(RwLock::new(data.perpetual_pos_short)),
            futures_pos_long: Arc::new(RwLock::new(data.futures_pos_long)),
            futures_pos_short: Arc::new(RwLock::new(data.futures_pos_short)),
            option_pos_long_call: Arc::new(RwLock::new(data.option_pos_long_call)),
            option_pos_long_put: Arc::new(RwLock::new(data.option_pos_long_put)),
            option_pos_short_call: Arc::new(RwLock::new(data.option_pos_short_call)),
            option_pos_short_put: Arc::new(RwLock::new(data.option_pos_short_put)),
        })
    }
}

impl AccountPositions {
    /// 创建一个新的 `AccountPositions` 实例
    pub fn init() -> Self {
        Self {
            margin_pos_long: Arc::new(RwLock::new(HashMap::new())),
            margin_pos_short: Arc::new(RwLock::new(HashMap::new())),
            perpetual_pos_long: Arc::new(RwLock::new(HashMap::new())),
            perpetual_pos_short: Arc::new(RwLock::new(HashMap::new())),
            futures_pos_long: Arc::new(RwLock::new(HashMap::new())),
            futures_pos_short: Arc::new(RwLock::new(HashMap::new())),
            option_pos_long_call: Arc::new(RwLock::new(HashMap::new())),
            option_pos_long_put: Arc::new(RwLock::new(HashMap::new())),
            option_pos_short_call: Arc::new(RwLock::new(HashMap::new())),
            option_pos_short_put: Arc::new(RwLock::new(HashMap::new())),
        }
    }


    /// TODO check init logic
    pub async fn build_new_perpetual_position(&self, config: &AccountConfig, trade: &ClientTrade, exchange_ts: i64) -> Result<PerpetualPosition, ExchangeError>
    {
        let position_mode = config.position_direction_mode.clone();
        let position_margin_mode = config.position_margin_mode.clone();
        // 计算初始保证金
        let initial_margin = trade.price * trade.quantity / config.account_leverage_rate;

        // 根据 Instrument 和 Side 动态生成 position_id
        let position_meta = PositionMetaBuilder::new().position_id(PositionId::new(&trade.instrument.clone(), trade.timestamp))
                                                      .enter_ts(exchange_ts)
                                                      .update_ts(exchange_ts)
                                                      .exit_balance(TokenBalance { // 初始化为 exit_balance
                                                                                   token: trade.instrument.base.clone(),
                                                                                   balance: Balance { time: Utc::now(),
                                                                                                      current_price: trade.price,
                                                                                                      total: trade.quantity,
                                                                                                      available: trade.quantity } })
                                                      .exchange(Exchange::SandBox)
                                                      .instrument(trade.instrument.clone())
                                                      .side(trade.side)
                                                      .current_size(trade.quantity)
                                                      .current_fees_total(trade.fees)
                                                      .current_avg_price_gross(trade.price)
                                                      .current_symbol_price(trade.price)
                                                      .current_avg_price(trade.price)
                                                      .unrealised_pnl(0.0) // 初始化为 0.0
                                                      .realised_pnl(0.0) // 初始化为 0.0
                                                      .build()
                                                      .map_err(|err| ExchangeError::SandBox(format!("Failed to build account_positions meta: {}", err)))?;

        // 计算 liquidation_price
        let liquidation_price = if trade.side == Side::Buy {
            trade.price * (1.0 - initial_margin / (trade.quantity * trade.price))
        }
        else {
            trade.price * (1.0 + initial_margin / (trade.quantity * trade.price))
        };
        let pos_config = PerpetualPositionConfig { pos_margin_mode: position_margin_mode,
                                                   leverage: config.account_leverage_rate,
                                                   position_mode };

        let new_position = PerpetualPositionBuilder::new().meta(position_meta)
                                                          .pos_config(pos_config)
                                                          .liquidation_price(liquidation_price)
                                                          .margin(initial_margin) // NOTE DOUBLE CHECK
                                                          .build()
                                                          .ok_or_else(|| ExchangeError::SandBox("Failed to build new account_positions".to_string()))?;

        Ok(new_position)
    }

        pub async fn update_position(&self, new_position: Position) {
            match new_position {
                Position::Perpetual(p) => match p.meta.side {
                    Side::Buy => {
                        let positions = &self.perpetual_pos_long;
                        let mut positions_lock = positions.write().await;
                        if let Some(existing_position) = positions_lock.get_mut(&p.meta.instrument) {
                            *existing_position = p;
                        } else {
                            positions_lock.insert(p.meta.instrument.clone(), p);
                        }
                    }
                    Side::Sell => {
                        let positions = &self.perpetual_pos_short;
                        let mut positions_lock = positions.write().await;
                        if let Some(existing_position) = positions_lock.get_mut(&p.meta.instrument) {
                            *existing_position = p;
                        } else {
                            positions_lock.insert(p.meta.instrument.clone(), p);
                        }
                    }
                },
                Position::LeveragedToken(p) => match p.meta.side {
                    Side::Buy => {
                        let positions = &self.margin_pos_long;
                        let mut positions_lock = positions.write().await;
                        if let Some(existing_position) = positions_lock.get_mut(&p.meta.instrument) {
                            *existing_position = p;
                        } else {
                            positions_lock.insert(p.meta.instrument.clone(), p);
                        }
                    }
                    Side::Sell => {
                        let positions = &self.margin_pos_short;
                        let mut positions_lock = positions.write().await;
                        if let Some(existing_position) = positions_lock.get_mut(&p.meta.instrument) {
                            *existing_position = p;
                        } else {
                            positions_lock.insert(p.meta.instrument.clone(), p);
                        }
                    }
                },
                Position::Future(p) => match p.meta.side {
                    Side::Buy => {
                        let positions = &self.futures_pos_long;
                        let mut positions_lock = positions.write().await;
                        if let Some(existing_position) = positions_lock.get_mut(&p.meta.instrument) {
                            *existing_position = p;
                        } else {
                            positions_lock.insert(p.meta.instrument.clone(), p);
                        }
                    }
                    Side::Sell => {
                        let positions = &self.futures_pos_short;
                        let mut positions_lock = positions.write().await;
                        if let Some(existing_position) = positions_lock.get_mut(&p.meta.instrument) {
                            *existing_position = p;
                        } else {
                            positions_lock.insert(p.meta.instrument.clone(), p);
                        }
                    }
                },
                Position::Option(_p) => {
                    todo!()
                }
            }
        }


    /// 检查账户中是否持有指定交易工具的多头仓位
    pub(crate) async fn has_long_position(&self, instrument: &Instrument) -> bool {
        match instrument.kind {
            InstrumentKind::Spot => todo!("[UniLinkExecution] : The system does not support creation or processing of positions of Spot as of yet."),
            InstrumentKind::CommodityOption => todo!("[UniLinkExecution] : The system does not support creation or processing of positions of CommodityOption as of yet."),
            InstrumentKind::CommodityFuture => todo!("[UniLinkExecution] : The system does not support creation or processing of positions of CommodityFuture as of yet."),
            InstrumentKind::Perpetual => {
                let positions = self.perpetual_pos_long.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
            InstrumentKind::Future => {
                let positions = self.futures_pos_long.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
            InstrumentKind::CryptoOption => {
                let positions = self.option_pos_long_call.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
            InstrumentKind::CryptoLeveragedToken => {
                let positions = self.margin_pos_long.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
        }
    }

    /// 检查账户中是否持有指定交易工具的空头仓位
    pub(crate) async fn has_short_position(&self, instrument: &Instrument) -> bool {
        match instrument.kind {
            InstrumentKind::Spot => todo!("[UniLinkExecution] : The system does not support creation or processing of positions of Spot as of yet."),
            InstrumentKind::CommodityOption => todo!("[UniLinkExecution] : The system does not support creation or processing of positions of CommodityOption as of yet."),
            InstrumentKind::CommodityFuture => todo!("[UniLinkExecution] : The system does not support creation or processing of positions of CommodityFuture as of yet."),
            InstrumentKind::Perpetual => {
                let positions = self.perpetual_pos_short.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
            InstrumentKind::Future => {
                let positions = self.futures_pos_short.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
            InstrumentKind::CryptoOption => {
                let positions = self.option_pos_short_put.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
            InstrumentKind::CryptoLeveragedToken => {
                let positions = self.margin_pos_short.read().await;
                positions.iter().any(|(key, _)| key == instrument)
            }
        }
    }
}

///  [NetMode] : 单向模式。在这种模式下，用户只能持有一个方向的仓位（多头或空头），而不能同时持有两个方向的仓位。
///  [LongShortMode] : 双向模式。在这种模式下，用户可以同时持有多头和空头仓位。这在一些复杂的交易策略中可能会有用，例如对冲策略。
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum PositionDirectionMode
{
    LongShortMode, // Note long/short, only applicable to Futures/Swap
    NetMode,       // Note one side per token per account_positions
}

///  [Cross]: 交叉保证金模式。在这种模式下，所有仓位共享一个保证金池，盈亏共用。如果仓位的保证金不足，将从账户余额中提取以补充不足。
///  [Isolated]: 逐仓保证金模式。在这种模式下，每个仓位都有独立的保证金，盈亏互不影响。如果某个仓位的保证金不足，该仓位将被强制平仓，而不会影响到其他仓位。

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum PositionMarginMode
{
    Cross,
    Isolated,
}

/// NOTE: 可能需要多种头寸类型共存
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Position
{
    Perpetual(PerpetualPosition),
    LeveragedToken(LeveragedTokenPosition),
    Future(FuturePosition),
    Option(OptionPosition),
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::common::token::Token;

    fn create_instrument(kind: InstrumentKind) -> Instrument
    {
        Instrument { base: Token::from("BTC"),
                     quote: Token::from("USDT"),
                     kind }
    }

    fn create_perpetual_position(instrument: &Instrument) -> PerpetualPosition
    {
        // 假设初始交易价格和仓位大小
        let initial_trade_price = 50000.0;
        let trade_size = 1.0;

        // 设置杠杆率
        let leverage = 10.0;

        // 计算初始保证金
        let initial_margin = initial_trade_price * trade_size / leverage;

        // 假设当前市场价格略有波动
        let current_market_price = 50500.0;

        // 计算当前仓位的未实现盈亏
        let unrealised_pnl = (current_market_price - initial_trade_price) * trade_size;

        // 计算清算价格 (liquidation_price)
        let liquidation_price = initial_trade_price * (1.0 - initial_margin / (trade_size * initial_trade_price));

        PerpetualPosition { meta: PositionMetaBuilder::new().position_id(PositionId(124124123412412))
                                                            .instrument(instrument.clone())
                                                            .side(Side::Buy)
                                                            .enter_ts(1625097600000)
                                                            .update_ts(1625097600000)
                                                            .exit_balance(TokenBalance { token: instrument.base.clone(),
                                                                                         balance: Balance { time: Utc::now(),
                                                                                                            current_price: current_market_price,
                                                                                                            total: trade_size,
                                                                                                            available: trade_size } })
                                                            .exchange(Exchange::Binance)
                                                            .current_size(trade_size)
                                                            .current_fees_total(0.2)
                                                            .current_avg_price_gross(initial_trade_price)
                                                            .current_symbol_price(current_market_price)
                                                            .current_avg_price(initial_trade_price)
                                                            .unrealised_pnl(unrealised_pnl)
                                                            .realised_pnl(0.0)
                                                            .build()
                                                            .unwrap(),
                            liquidation_price,
                            margin: initial_margin,
                            pos_config: PerpetualPositionConfig { pos_margin_mode: PositionMarginMode::Cross,
                                                                  leverage,
                                                                  position_mode: PositionDirectionMode::NetMode } }
    }

    #[tokio::test] // 使用 tokio 的异步测试宏
    async fn test_has_position() {
        let account_positions = AccountPositions::init();

        let perpetual_instrument = create_instrument(InstrumentKind::Perpetual);
        let future_instrument = create_instrument(InstrumentKind::Future);

        // 初始情况下，没有任何仓位
        assert!(!account_positions.has_long_position(&perpetual_instrument).await);
        assert!(!account_positions.has_short_position(&perpetual_instrument).await);
        assert!(!account_positions.has_long_position(&future_instrument).await);
        assert!(!account_positions.has_short_position(&future_instrument).await);

        // 创建并添加 PerpetualPosition 多头仓位
        let mut perpetual_position = create_perpetual_position(&perpetual_instrument);
        perpetual_position.meta.side = Side::Buy; // 设置为多头仓位
        account_positions.update_position(Position::Perpetual(perpetual_position.clone())).await;

        // 现在应该持有 PerpetualPosition 多头仓位，但不持有 FuturePosition
        assert!(account_positions.has_long_position(&perpetual_instrument).await);
        assert!(!account_positions.has_short_position(&perpetual_instrument).await);
        assert!(!account_positions.has_long_position(&future_instrument).await);
        assert!(!account_positions.has_short_position(&future_instrument).await);

        // 创建并添加 PerpetualPosition 空头仓位
        perpetual_position.meta.side = Side::Sell; // 设置为空头仓位
        account_positions.update_position(Position::Perpetual(perpetual_position.clone())).await;

        // 现在应该持有 PerpetualPosition 的空头和多头仓位
        assert!(account_positions.has_long_position(&perpetual_instrument).await);
        assert!(account_positions.has_short_position(&perpetual_instrument).await);
    }

    #[tokio::test] // 使用 tokio 的异步测试宏
    async fn test_update_existing_position() {
        let  account_positions = AccountPositions::init();

        let perpetual_instrument = create_instrument(InstrumentKind::Perpetual);

        // 添加初始的 PerpetualPosition 多头仓位
        let mut perpetual_position = create_perpetual_position(&perpetual_instrument);
        perpetual_position.meta.side = Side::Buy; // 设置为多头仓位
        account_positions.update_position(Position::Perpetual(perpetual_position.clone())).await;

        // 确保初始 PerpetualPosition 已正确添加
        assert!(account_positions.has_long_position(&perpetual_instrument).await);

        // 获取写锁并检查仓位数量
        {
            let positions = account_positions.perpetual_pos_long.read().await;
            assert_eq!(positions.len(), 1);
        }

        // 更新相同的 PerpetualPosition，修改 `margin`
        let mut updated_position = perpetual_position.clone();
        updated_position.margin = 2000.0; // 修改仓位的保证金

        account_positions.update_position(Position::Perpetual(updated_position.clone())).await;

        // 确保仓位已更新而不是新添加
        {
            let positions = account_positions.perpetual_pos_long.read().await;
            if !positions.is_empty() {
                assert_eq!(positions.len(), 1); // 确保仓位数量未增加
                let pos = positions.get(&perpetual_instrument).unwrap();
                assert_eq!(pos.margin, 2000.0); // 检查仓位是否已正确更新
            } else {
                panic!("PerpetualPosition should exist but was not found.");
            }
        }
    }

    #[tokio::test] // 使用 tokio 的异步测试宏
    async fn test_add_new_position() {
        let account_positions = AccountPositions::init();

        // 创建两个不同的 Instrument
        let perpetual_instrument_1 = Instrument {
            base: Token::from("BTC"),
            quote: Token::from("USDT"),
            kind: InstrumentKind::Perpetual,
        };

        let perpetual_instrument_2 = Instrument {
            base: Token::from("ETH"),
            quote: Token::from("USDT"),
            kind: InstrumentKind::Perpetual,
        };

        // 添加初始的 PerpetualPosition (多头仓位)
        let mut perpetual_position_1 = create_perpetual_position(&perpetual_instrument_1);
        perpetual_position_1.meta.side = Side::Buy; // 设置为多头仓位
        account_positions.update_position(Position::Perpetual(perpetual_position_1.clone())).await;

        // 添加新的 PerpetualPosition (多头仓位)
        let mut perpetual_position_2 = create_perpetual_position(&perpetual_instrument_2);
        perpetual_position_2.meta.side = Side::Buy; // 设置为多头仓位
        account_positions.update_position(Position::Perpetual(perpetual_position_2.clone())).await;

        // 确保新仓位已正确添加
        assert!(account_positions.has_long_position(&perpetual_instrument_1).await);
        assert!(account_positions.has_long_position(&perpetual_instrument_2).await);

        // 获取读锁并检查仓位数量
        {
            let positions = account_positions.perpetual_pos_long.read().await;
            assert_eq!(positions.len(), 2);
        }
    }

}
