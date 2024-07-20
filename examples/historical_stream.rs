use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::mpsc::unbounded_channel;

use unilink_execution::{
    common_skeleton::datafeed::event::MarketEvent,
    simulated_exchange::{account::account_market_feed::*, load_from_clickhouse::queries_operations::*},
};

// 使用 lazy_static 宏来创建一个全局静态变量 CLIENT
lazy_static! {
    pub static ref CLIENT: Arc<ClickHouseClient> = Arc::new(ClickHouseClient::new());
}
#[tokio::main]
async fn main() {
    let client = Arc::new(ClickHouseClient::new());


    // 测试查询
    let test_query = "SELECT symbol, side, price, timestamp FROM binance_futures_trades.binance_futures_trades_union_2024_03_03 ORDER BY timestamp LIMIT 1000000 OFFSET 0";
    match client.client.read().await.query(test_query).fetch_all::<ClickhouseTrade>().await {
        Ok(trade_datas) => {
            println!("Query succeeded: fetched {} rows", trade_datas.len());
            for _trade_data in &trade_datas {
                continue
            }
        }
        Err(e) => {
            eprintln!("Failed test query: {}", e);
            return;
        }
    }

    // 定义交易所、金融工具、频道和日期的字符串变量
    let stream_params = vec![("binance", "futures", "trades", "2024_03_03", 1000000)];

    // 创建 AccountMarketStreams 实例
    let mut account_streams: AccountDataStreams<MarketEvent<ClickhouseTrade>> = AccountDataStreams::new();

    for (exchange, instrument, channel, date, batch_size) in stream_params {
        let client = client.clone();
        match client
            .query_unioned_trade_table_batched_for_dates(exchange, instrument, channel, date, date, batch_size)
            .await
        {
            Ok(mut rx) => {
                let (tx, rx_clone) = unbounded_channel::<MarketEvent<ClickhouseTrade>>();

                let stream_id = format!("{}_{}_{}_{}", exchange, instrument, channel, date);
                account_streams.add_stream(stream_id.clone(), rx_clone);
                println!("Hooooray! New stream has been added.");

                while let Some(event) = rx.recv().await {
                    println!("Received event: {:?}", event);
                    if tx.send(event).is_err() {
                        eprintln!("Failed to send event");
                        break;
                    }
                }
                println!("Finished processing events for stream: {}", stream_id);
            }
            Err(e) => {
                eprintln!("Failed to query events: {}", e);
            }
        }
    }
}