use chrono::{Duration as ChronoDuration, NaiveDate};
use hourglass::hourglass::clickhouse_api::queries_operations::ClickHouseClient;
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main()
{
    // 创建 ClickHouse 客户端实例
    let client = Arc::new(ClickHouseClient::new());

    // 定义参数
    let exchange = "binance";
    let instrument = "futures";

    // 定义日期范围
    let start_date = NaiveDate::from_ymd_opt(2024, 5, 5).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 5, 7).unwrap();

    // 逐日遍历日期范围
    let mut current_date = start_date;
    while current_date <= end_date {
        let date_str = Arc::new(current_date.format("%Y-%m-%d").to_string());

        // 获取游标，这里假设你有一个方法 cursor_union_trades 用于查询 union 表
        let cursor_result = client.cursor_unioned_public_trades(exchange, instrument, &date_str).await;

        match cursor_result {
            | Ok(mut cursor) => {
                // 创建通道
                let (tx, _rx) = mpsc::unbounded_channel();

                // 克隆 Arc 以在异步任务中使用
                let date_str_clone = Arc::clone(&date_str);

                // 启动一个任务来从游标读取数据并发送到通道
                let cursor_task = tokio::spawn(async move {
                    loop {
                        match cursor.next().await {
                            | Ok(Some(trade)) => {
                                // 打印每个交易数据
                                println!("[{}]: {:?}", date_str_clone, trade);

                                if tx.send(trade).is_err() {
                                    // 如果发送失败（例如接收者已关闭），退出循环
                                    eprintln!("Failed to send trade, receiver might be closed.");
                                    break;
                                }
                            }
                            | Ok(None) => {
                                println!("Cursor data processing for date {} is complete.", date_str_clone);
                                break;
                            }
                            | Err(_e) => {
                                eprintln!("No data available for date {}. Skipping to next date.", date_str_clone);
                                break;
                            }
                        }
                    }
                });

                // 等待 `cursor_task` 完成
                if let Err(e) = cursor_task.await {
                    eprintln!("Cursor task for {} was aborted: {:?}", date_str, e);
                }
            }
            | Err(e) => {
                eprintln!("Error fetching trades for {}: {:?}", date_str, e);
            }
        }

        // 更新到下一天
        current_date += ChronoDuration::days(1);
    }
}
