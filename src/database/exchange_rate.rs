use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::environment;

/**
 * Save exchange rate to database
 */
pub fn save_exchange_rate(from: &str, to: &str, rate: f64) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO exchange_rate (from_currency, to_currency, rate) VALUES (?, ?, ?)";

    con.execute(query, &[from, to, rate.to_string().as_str()])
        .unwrap();

    log::debug!("Saved exchange rate from {} to {} as {}", from, to, rate);
}

/**
 * Save raw exchange rate to database
 */
pub fn save_raw_exchange_rate_result(raw: &str) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO exchange_rate_api_raw (raw) VALUES (?)";

    con.execute(query, &[raw]).unwrap();

    log::debug!("Saved raw exchange rate: {}", raw);
}

/**
 * Save exchange rate to database
 */
pub fn save_exchange_rate_fallback(txt: &str, timestamp: DateTime<Utc>) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO exchange_rate_api_fallback (json, time) VALUES (?, ?)";

    con.execute(query, &[txt, &timestamp.to_rfc3339()]).unwrap();

    log::debug!("Saved fallback exchange rate: {txt}");
}

/**
 * Save exchange rate to database
 */
pub fn get_local_exchange_rate_fallback(date: chrono::NaiveDate) -> Option<String> {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();
    // Define the query
    let query = r#"
        SELECT json 
        FROM exchange_rate_api_fallback 
        WHERE DATE(time) = ?
        ORDER BY time DESC
        LIMIT 1;
    "#;

    match con.query_row(query, params![date.format("%Y-%m-%d").to_string()], |row| row.get(0)) {
        Ok(json) => {
            log::debug!("Retrieved fallback exchange rate for {}: {}", date, json);
            Some(json)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            log::debug!("No fallback exchange rate found for {}", date);
            None
        }
        Err(e) => {
            log::error!("Error while querying database: {}", e);
            None
        }
    }
}
