use rusqlite::Connection;

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
