use dotenv::dotenv;
mod environment;
mod utils;
mod bot;
mod llm;
mod commands;
mod database;
mod exchange_rate;

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(_) => {
            env_logger::init();
            log::debug!("Loaded .env file")},
        Err(_) => {
            env_logger::init();
            log::debug!("No .env file found")
        }
    }

    log::debug!("Starting bot");

    log::debug!("Log level: {}", log::max_level());

    environment::ensure_environment().await;
    bot::run_bot().await;
}
