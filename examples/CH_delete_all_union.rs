use hourglass::hourglass::clickhouse_api::queries_operations::ClickHouseClient;

#[tokio::main]
async fn main()
{
    let client = ClickHouseClient::new();
    // fields for binance
    let exchange = "binance";
    let instrument = "futures";
    let channel = "trades";

    // 将上述变量拼接成数据库名
    let database = client.construct_database_name(exchange, instrument, channel);

    // 获取所有表名
    let all_tables = client.get_table_names(&database).await;

    // 筛选出包含 union 字样的表名
    let union_tables: Vec<String> = all_tables.into_iter().filter(|table_name| table_name.contains("union")).collect();
    println!("Union tables: {:?}", union_tables); // 打印 union 表名的调试信息

    // 删除 union 表
    for table in union_tables {
        let drop_query = format!("DROP TABLE {}.{}", database, table);
        println!("[ClickHouse] : Executing query: {}", drop_query);
        println!("Executing query: {}", drop_query);

        match client.client.read().await.query(&drop_query).execute().await {
            | Ok(_) => println!("Successfully dropped table: {}.{}", database, table),
            | Err(e) => eprintln!("Error dropping table: {}", e),
        }
    }
}
