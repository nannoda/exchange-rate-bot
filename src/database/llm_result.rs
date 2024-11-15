use rusqlite::Connection;

use crate::environment;

/**
 * Save raw LLM result to database
 */
pub fn save_llm_result(prompt: &str, result: &str) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO llm_result (prompt, result) VALUES (?, ?)";

    con.execute(query, &[prompt, result]).unwrap();

    log::debug!("Saved llm result: {} -> {}", prompt, result);
}
