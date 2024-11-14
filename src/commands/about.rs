use serenity::all::{CreateCommand, CreateCommandOption};

use crate::environment;

pub const COMMAND_NAME: &str = "about";

// Function to register the exchange command with optional "from" and "to" parameters
pub fn register() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME).description("About the bot")
}
pub fn run() -> String {
    format!(
    "
    **Exchange Rate Bot**\n\
    Version: `{}`\n\n\
    **Configuration**:\n\
    - **Exchange From**: `{}`\n\
    - **Exchange To**: `{}`\n\
    - **Interval**: `{}` seconds\n\
    - **Exchange Rate API URL**: `{}`\n\
    - **OLLAMA Model**: `{}`\n\n\
    **Channels**: {:?}\n\
    ",
        environment::APP_VERSION,
        environment::get_exchange_from(),
        environment::get_exchange_to(),
        environment::get_interval(),
        environment::get_exchange_rate_api_url(),
        environment::get_ollama_model(),
        environment::get_channels()
    )
}
