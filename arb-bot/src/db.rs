use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS opportunities(
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                ts_utc        TEXT NOT NULL,
                dex_buy       TEXT NOT NULL,
                dex_sell      TEXT NOT NULL,
                token_in      TEXT NOT NULL,
                token_out     TEXT NOT NULL,
                amount_in     TEXT NOT NULL,
                amount_out    TEXT NOT NULL,
                price_buy     REAL NOT NULL,
                price_sell    REAL NOT NULL,
                gross_profit  REAL NOT NULL,
                net_profit    REAL NOT NULL
            );
        "#,
        )?;
        Ok(Self { conn })
    }

    @allow(non_snake_case)
    pub fn insert_opportunity(
        &self,
        dex_buy: &str,
        dex_sell: &str,
        token_in: &str,
        token_out: &str,
        amount_in: &str,
        amount_out: &str,
        price_buy: f64,
        price_sell: f64,
        gross_profit: f64,
        net_profit: f64,
    ) -> Result<()> {
        let now: DateTime<Utc> = Utc::now();
        self.conn.execute(
            r#"INSERT INTO opportunities
            (ts_utc, dex_buy, dex_sell, token_in, token_out, amount_in, amount_out,
             price_buy, price_sell, gross_profit, net_profit)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                now.to_rfc3339(),
                dex_buy,
                dex_sell,
                token_in,
                token_out,
                amount_in,
                amount_out,
                price_buy,
                price_sell,
                gross_profit,
                net_profit
            ],
        )?;
        Ok(())
    }
}
