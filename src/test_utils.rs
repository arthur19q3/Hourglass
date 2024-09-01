// src/test_utils

use crate::{
    common::{
        balance::{Balance, TokenBalance},
        friction::{Fees, FutureFees, PerpetualFees},
        instrument::{kind::InstrumentKind, Instrument},
        order::{
            identification::{client_order_id::ClientOrderId, machine_id::generate_machine_id, OrderId},
            order_instructions::OrderInstruction,
            states::{open::Open, request_open::RequestOpen},
            Order, OrderRole,
        },
        position::{
            future::{FuturePosition, FuturePositionConfig},
            perpetual::{PerpetualPosition, PerpetualPositionConfig},
            position_id::PositionId,
            position_meta::PositionMeta,
            AccountPositions, PositionDirectionMode, PositionMarginMode,
        },
        token::Token,
        Side,
    },
    sandbox::account::{
        account_config::{AccountConfig, CommissionLevel, CommissionRates, MarginMode, SandboxMode},
        account_latency::{AccountLatency, FluctuationMode},
        account_orders::AccountOrders,
        account_states::AccountState,
        Account,
    },
    Exchange,
};
use rand::Rng;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicI64, Arc, Weak},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// 创建一个测试用的 `Instrument` 实例。
pub fn create_test_instrument(kind: InstrumentKind) -> Instrument
{
    Instrument {
        base: Token::from("BTC"),
        quote: Token::from("USDT"),
        kind,
    }
}

/// 创建一个测试用的 `AccountConfig` 实例。
pub fn create_test_account_config() -> AccountConfig
{
    let leverage_rate = 1.0;

    AccountConfig {
        margin_mode: MarginMode::SingleCurrencyMargin,
        position_mode: PositionDirectionMode::NetMode,
        position_margin_mode: PositionMarginMode::Isolated,
        commission_level: CommissionLevel::Lv1,
        funding_rate: 0.0,
        account_leverage_rate: leverage_rate,
        fees_book: HashMap::new(),
        execution_mode: SandboxMode::Backtest,
    }
}
// 帮助函数，用于创建测试用的 AccountOrders 实例
pub async fn create_test_account_orders() -> AccountOrders
{
    let instruments = vec![Instrument::new("BTC", "USD", InstrumentKind::Spot)];
    let account_latency = AccountLatency::new(FluctuationMode::Sine, 100, 10);
    AccountOrders::new(123124, instruments, account_latency).await
}

/// 创建一个测试用的 `Order<Open>` 实例。
pub fn create_test_order_open(side: Side, price: f64, size: f64) -> Order<Open>
{
    Order {
        kind: OrderInstruction::Limit, // 假设测试订单使用限价订单类型
        exchange: Exchange::SandBox,   // 假设测试环境使用 SandBox 交易所
        instrument: Instrument {
            base: Token::from("TEST_BASE"),   // 测试用基础货币
            quote: Token::from("TEST_QUOTE"), // 测试用报价货币
            kind: InstrumentKind::Perpetual, /* 测试用永续合约 */
        },
        timestamp: 1625247600000,                       // 假设的客户端时间戳
        cid: ClientOrderId(Some("validCID123".into())), // 假设的客户端订单ID
        side,
        state: Open {
            id: OrderId(123), // 假设的订单ID
            price,
            size,
            filled_quantity: 0.0,         // 初始填充数量为0
            order_role: OrderRole::Taker, // 假设订单角色为 Taker
        },
    }
}

// 帮助函数，用于创建测试用的订单
pub fn create_test_request_open(base: &str, quote: &str) -> Order<RequestOpen>
{
    let machine_id = generate_machine_id().unwrap();
    let mut rng = rand::thread_rng();
    let counter = rng.gen_range(0..10);
    let now_ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("时间出现倒退").as_millis() as u64;

    let order_id = OrderId::new(now_ts, machine_id, counter);
    Order {
        kind: OrderInstruction::Market,
        exchange: Exchange::SandBox,
        instrument: Instrument {
            base: Token::from(base),
            quote: Token::from(quote),
            kind: InstrumentKind::Spot,
        },
        timestamp: 1625247600000,
        cid: ClientOrderId(Some(format!("CID{}", order_id.0 % 1_000_000))),
        side: Side::Buy,
        state: RequestOpen {
            price: 50000.0,
            size: 1.0,
            reduce_only: false,
        },
    }
}

pub async fn create_test_account_state() -> Arc<Mutex<AccountState>> {
    // 创建初始余额
    let mut balances = HashMap::new();
    let token1 = Token::from("TEST_BASE");
    let token2 = Token::from("TEST_QUOTE");
    balances.insert(token1.clone(), Balance::new(100.0, 50.0, 1.0));
    balances.insert(token2.clone(), Balance::new(200.0, 150.0, 1.0));

    // 创建初始持仓
    let positions = AccountPositions {
        margin_pos: Vec::new(),
        perpetual_pos: Vec::new(),
        futures_pos: Vec::new(),
        option_pos: Vec::new(),
    };

    // 创建 AccountState 实例，先不设置 account_ref
    let account_state = AccountState {
        balances: balances.clone(),
        positions: positions.clone(),
        account_ref: Weak::new(),  // 初始为空的 Weak
    };

    // 包装 AccountState 实例在 Arc<Mutex<...>> 中
    let account_state_arc = Arc::new(Mutex::new(account_state));

    // 创建 Account 实例，并将其包装在 Arc<Mutex<...>> 中
    let account = Arc::new(Mutex::new(Account {
        current_session: Uuid::new_v4(),
        machine_id: 0,
        exchange_timestamp: AtomicI64::new(1234567),
        account_event_tx: tokio::sync::mpsc::unbounded_channel().0,
        config: Arc::new(create_test_account_config()),
        states: account_state_arc.clone(),
        orders: Arc::new(tokio::sync::RwLock::new(AccountOrders::new(0, vec![], AccountLatency {
            fluctuation_mode: FluctuationMode::Sine,
            maximum: 0,
            minimum: 0,
            current_value: 0,
        }).await)),
    }));

    // 将 `Arc<Mutex<Account>>` 转换为 `Arc<Account>`
    let account_arc = Arc::clone(&account);
    let account_unwrapped = Arc::new(account_arc.lock().await.clone());

    // 获取 Account 的锁定版本并将其传递给 `Arc::downgrade`
    {
        let mut account_state_locked = account_state_arc.lock().await;
        account_state_locked.account_ref = Arc::downgrade(&account_unwrapped);
    }

    account_state_arc
}


pub async fn create_test_account() -> Arc<Mutex<Account>> {
    let leverage_rate = 1.0;
    let mut balances = HashMap::new();
    balances.insert(Token::from("TEST_BASE"), Balance::new(10.0, 10.0, 1.0));
    balances.insert(Token::from("TEST_QUOTE"), Balance::new(10_000.0, 10_000.0, 1.0));

    let commission_rates = CommissionRates {
        maker_fees: 0.001,
        taker_fees: 0.002,
    };

    let mut account_config = AccountConfig {
        margin_mode: MarginMode::SingleCurrencyMargin,
        position_mode: PositionDirectionMode::NetMode,
        position_margin_mode: PositionMarginMode::Isolated,
        commission_level: CommissionLevel::Lv1,
        funding_rate: 0.0,
        account_leverage_rate: leverage_rate,
        fees_book: HashMap::new(),
        execution_mode: SandboxMode::Backtest,
    };

    account_config
        .fees_book
        .insert(InstrumentKind::Perpetual, commission_rates);

    let positions = AccountPositions {
        margin_pos: Vec::new(),
        perpetual_pos: Vec::new(),
        futures_pos: Vec::new(),
        option_pos: Vec::new(),
    };

    let machine_id = generate_machine_id().unwrap();

    // 预先创建空的 AccountState，然后再初始化 Account
    let account_state = AccountState {
        balances,
        positions,
        account_ref: Weak::new(), // 先初始化为空的 Weak
    };

    let account_state_arc = Arc::new(Mutex::new(account_state));

    // 创建 Account 实例，并将其包裹在 Arc<Mutex<...>> 中
    let account = Arc::new(Mutex::new(Account {
        current_session: Uuid::new_v4(),
        machine_id,
        exchange_timestamp: AtomicI64::new(1234567),
        account_event_tx: tokio::sync::mpsc::unbounded_channel().0,
        config: Arc::new(account_config),
        states: account_state_arc.clone(),
        orders: Arc::new(RwLock::new(
            AccountOrders::new(
                machine_id,
                vec![Instrument::from(("TEST_BASE", "TEST_QUOTE", InstrumentKind::Perpetual))],
                AccountLatency {
                    fluctuation_mode: FluctuationMode::Sine,
                    maximum: 300,
                    minimum: 0,
                    current_value: 0,
                },
            ).await,
        )),
    }));

    // 解包 account 的 Mutex，提取 Arc<Account>
    let account_arc = Arc::clone(&account);
    let account_unwrapped = Arc::new(account_arc.lock().await.clone());

    // 更新 account_ref，使其指向 Weak<Account>
    {
        let mut account_state_locked = account_state_arc.lock().await;
        account_state_locked.account_ref = Arc::downgrade(&account_unwrapped);
    }

    account
}
/// 创建一个测试用的 `PerpetualPosition` 实例。
pub fn create_test_perpetual_position(instrument: Instrument) -> PerpetualPosition
{
    PerpetualPosition {
        meta: PositionMeta {
            position_id: PositionId(12341241241),
            enter_ts: 0,
            update_ts: 0,
            exit_balance: TokenBalance {
                token: instrument.base.clone(),
                balance: Balance::new(0.0, 0.0, 1.0),
            },
            exchange: Exchange::SandBox,
            instrument,
            side: Side::Buy,
            current_size: 1.0,
            current_fees_total: Fees::Perpetual(PerpetualFees {
                maker_fee: 0.0,
                taker_fee: 0.0,
                funding_fee: 0.0,
            }),
            current_avg_price_gross: 0.0,
            current_symbol_price: 0.0,
            current_avg_price: 0.0,
            unrealised_pnl: 0.0,
            realised_pnl: 0.0,
        },
        pos_config: PerpetualPositionConfig {
            pos_margin_mode: PositionMarginMode::Isolated,
            leverage: 1.0,
            position_mode: PositionDirectionMode::LongShortMode,
        },
        liquidation_price: 0.0,
        margin: 0.0,
    }
}

/// 创建一个测试用的 `FuturePosition` 实例，指定 `Side`。
pub fn create_test_future_position_with_side(instrument: Instrument, side: Side) -> FuturePosition
{
    FuturePosition {
        meta: PositionMeta {
            position_id: PositionId(1234124512412),
            enter_ts: 0,
            update_ts: 0,
            exit_balance: TokenBalance {
                token: instrument.base.clone(),
                balance: Balance::new(0.0, 0.0, 1.0),
            },
            exchange: Exchange::SandBox,
            instrument,
            side,
            current_size: 0.0,
            current_fees_total: Fees::Future(FutureFees {
                maker_fee: 0.0,
                taker_fee: 0.0,
                funding_fee: 0.0,
            }),
            current_avg_price_gross: 0.0,
            current_symbol_price: 0.0,
            current_avg_price: 0.0,
            unrealised_pnl: 0.0,
            realised_pnl: 0.0,
        },
        pos_config: FuturePositionConfig {
            pos_margin_mode: PositionMarginMode::Isolated,
            leverage: 1.0,
            position_mode: PositionDirectionMode::LongShortMode,
        },
        liquidation_price: 0.0,
        margin: 0.0,
        funding_fee: 0.0
    }
}
