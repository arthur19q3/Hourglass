// 引入多生产者单消费者通道模块和执行错误类型。
use tokio::sync::mpsc;
use crate::ExecutionError;

// 引入上级模块中的客户账户和模拟事件类型。
use super::{exchange::account::ClientAccount, SimulatedEvent};

/// [`SimulatedExchange`] 的账户余额、开放订单、费用和延迟。
pub mod account;

/// 响应 [`SimulatedEvent`] 的 [`SimulatedExchange`]。
#[derive(Debug)]
pub struct SimulatedExchange {
    // 模拟事件的无界接收器。
    pub event_simulated_rx: mpsc::UnboundedReceiver<SimulatedEvent>,
    // 客户账户。
    pub account: ClientAccount,
}

impl SimulatedExchange {
    /// 构造一个用于配置新 [`SimulatedExchange`] 的 [`ExchangeBuilder`]。
    pub fn builder() -> ExchangeBuilder {
        ExchangeBuilder::new()
    }

    /// 运行 [`SimulatedExchange`] 并响应 [`SimulatedEvent`]。
    pub async fn run(mut self) {
        // 不断接收并处理模拟事件。
        while let Some(event) = self.event_simulated_rx.recv().await {
            match event {
                // 处理获取开放订单请求。
                SimulatedEvent::FetchOrdersOpen(response_tx) => {
                    self.account.fetch_orders_open(response_tx)
                }
                // 处理获取账户余额请求。
                SimulatedEvent::FetchBalances(response_tx) => {
                    self.account.fetch_balances(response_tx)
                }
                // 处理开启订单请求。
                SimulatedEvent::OpenOrders((open_requests, response_tx)) => {
                    self.account.open_orders(open_requests, response_tx)
                }
                // 处理取消订单请求。
                SimulatedEvent::CancelOrders((cancel_requests, response_tx)) => {
                    self.account.cancel_orders(cancel_requests, response_tx)
                }
                // 处理取消所有订单请求。
                SimulatedEvent::CancelOrdersAll(response_tx) => {
                    self.account.cancel_orders_all(response_tx)
                }
                // 处理市场交易事件。
                SimulatedEvent::MarketTrade((instrument, trade)) => {
                    self.account.match_orders(instrument, trade)
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ExchangeBuilder {
    // 模拟事件的无界接收器，用于构建器。
    event_simulated_rx: Option<mpsc::UnboundedReceiver<SimulatedEvent>>,
    // 客户账户，用于构建器。
    account: Option<ClientAccount>,
}

impl ExchangeBuilder {
    // 构造函数，创建新的构建器实例。
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    // 设置模拟事件的接收器。
    pub fn event_simulated_rx(self, value: mpsc::UnboundedReceiver<SimulatedEvent>) -> Self {
        Self {
            event_simulated_rx: Some(value),
            ..self
        }
    }

    // 设置客户账户。
    pub fn account(self, value: ClientAccount) -> Self {
        Self {
            account: Some(value),
            ..self
        }
    }

    // 构建并返回 `SimulatedExchange` 实例。
    pub fn build(self) -> Result<SimulatedExchange, ExecutionError> {
        Ok(SimulatedExchange {
            event_simulated_rx: self.event_simulated_rx.ok_or_else(|| {
                ExecutionError::BuilderIncomplete("event_simulated_rx".to_string())
            })?,
            account: self
                .account
                .ok_or_else(|| ExecutionError::BuilderIncomplete("account".to_string()))?,
        })
    }
}
