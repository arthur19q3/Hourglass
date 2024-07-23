use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{
    common_skeleton::{event::ClientOrderId, instrument::Instrument, Side, token::Token},
    ExchangeID,
};

/// 订单类型枚举
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum OrderKind
{
    Market,
    Limit,
    ImmediateOrCancel,
    FillOrKill,
    GoodTilCancelled,
}

impl Display for OrderKind
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", match self {
            | OrderKind::Market => "market",
            | OrderKind::Limit => "limit",
            | OrderKind::ImmediateOrCancel => "immediate_or_cancel (IOC)",
            | OrderKind::FillOrKill => "fill_or_kill (FOK)",
            | OrderKind::GoodTilCancelled => "good_til_cancelled (GTC)",
            // | OrderKind::Stop => "stop",
            // | OrderKind::StopLimit => "stop_limit",
            // | OrderKind::TrailingStop => "trailing_stop",
            // | OrderKind::Iceberg => "iceberg",
        })
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Order<State>
{
    pub exchange: ExchangeID,   // 交易所
    pub instrument: Instrument, // 交易工具
    pub client_ts: i64,
    pub cid: ClientOrderId, // 客户端订单ID
    pub side: Side,         // 买卖方向
    pub state: State,       // 订单状态
}

/// 订单初始状态。发送到client进行操作
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct RequestOpen
{
    pub kind: OrderKind,
    pub price: f64,
    pub size: f64,
}

// NOTE that this needs to be adjusted according to the specifics of our trading instruments.
impl Order<RequestOpen>
{
    pub fn calculate_required_available_balance(&self) -> (&Token, f64)
    {
        match self.side {
            | Side::Buy => (&self.instrument.quote, self.state.price * self.state.size),
            | Side::Sell => (&self.instrument.base, self.state.size),
        }
    }
}

/// 发送RequestOpen到client后尚未收到确认响应时的状态
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct RealPending;

/// 在RequestCancel结构体中只记录OrderId的原因主要是因为取消订单操作通常只需要知道哪个订单需要被取消。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct RequestCancel
{
    pub id: OrderId, // Consider : 需要记录 CID 吗 ????
}

// 从Id直接生成RequestCancel
impl<Id> From<Id> for RequestCancel where Id: Into<OrderId>
{
    fn from(id: Id) -> Self
    {
        Self { id: id.into() }
    }
}

/// 开放状态的订单
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Open
{
    pub kind: OrderKind,
    pub id: OrderId,
    pub price: f64,
    pub size: f64,
    pub filled_quantity: f64,
    // NOTE or [remaining_size]  , essentially the same.
}

impl Open
{
    pub fn remaining_quantity(&self) -> f64
    {
        self.size - self.filled_quantity
    }
}

/// 完全成交状态的订单, FullFill 以后在account层面 [AccountOrders] 删除对应open订单
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct FullyFill
{
    pub id: OrderId,
    pub price: f64,
    pub size: f64,
}

/// 完全成交状态的订单,FullFill 以后在account层面 [AccountOrders] 仍然保留open订单
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct PartialFill
{
    pub id: OrderId,
    pub price: f64,
    pub size: f64,
}

/// 使得Order<Open> 之间可以比较大小
/// NOTE: 此处Self 等同于 Order<Open>，表示 other 参数也是一个 Order<Open> 类型的引用。
impl Ord for Order<Open>
{
    fn cmp(&self, other: &Self) -> Ordering
    {
        self.partial_cmp(other)
            .unwrap_or_else(|| panic!("[UniLinkExecution] : {:?}.partial_cmp({:?}) impossible", self, other))
    }
}

impl PartialOrd for Order<Open>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        match (self.side, other.side) {
            | (Side::Buy, Side::Buy) => match self.state.price.partial_cmp(&other.state.price)? {
                | Ordering::Equal => self.state.remaining_quantity().partial_cmp(&other.state.remaining_quantity()),
                | non_equal => Some(non_equal),
            },
            | (Side::Sell, Side::Sell) => match other.state.price.partial_cmp(&self.state.price)? {
                | Ordering::Equal => other.state.remaining_quantity().partial_cmp(&self.state.remaining_quantity()),
                | non_equal => Some(non_equal),
            },
            | _ => None,
        }
    }
}

// 为Order<Open>实现Eq
impl Eq for Order<Open> {}

/// 构建订单在被取消后的状态
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Cancelled
{
    pub id: OrderId,
}

impl<Id> From<Id> for Cancelled where Id: Into<OrderId>
{
    fn from(id: Id) -> Self
    {
        Self { id: id.into() }
    }
}

/// 订单ID / OrderId，应当由交易所生成。
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct OrderId(pub String);

impl<Id> From<Id> for OrderId where Id: Display
{
    fn from(id: Id) -> Self
    {
        Self(id.to_string())
    }
}

impl From<&Order<RequestOpen>> for Order<RealPending>
{
    fn from(request: &Order<RequestOpen>) -> Self
    {
        Self { exchange: request.exchange.clone(),
               instrument: request.instrument.clone(),
               cid: request.cid,
               client_ts: request.client_ts,
               side: request.side,
               state: RealPending } // NOTE compatability with SimulatedPending is due here
    }
}

impl From<(OrderId, Order<RequestOpen>)> for Order<Open>
{
    fn from((id, request): (OrderId, Order<RequestOpen>)) -> Self
    {
        Self { exchange: request.exchange.clone(),
               instrument: request.instrument.clone(),
               cid: request.cid,
               client_ts: request.client_ts,
               side: request.side,
               state: Open { id,
                             kind: request.state.kind,
                             price: request.state.price,
                             size: request.state.size,
                             filled_quantity: 0.0 } }
    }
}

impl From<Order<Open>> for Order<Cancelled>
{
    fn from(order: Order<Open>) -> Self
    {
        Self { exchange: order.exchange.clone(),
               instrument: order.instrument.clone(),
               cid: order.cid,
               client_ts: order.client_ts,
               side: order.side,
               state: Cancelled { id: order.state.id } }
    }
}
