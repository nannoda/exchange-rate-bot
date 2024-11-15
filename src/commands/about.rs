use serenity::all::{CreateCommand, CreateInteractionResponseMessage, CreateMessage, Message};

use crate::environment;

pub const COMMAND_NAME: &str = "about";

// Function to register the exchange command with optional "from" and "to" parameters
pub fn register() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME).description("About the bot")
}
pub fn run() -> CreateInteractionResponseMessage {
    let content = format!(
    "
    **Exchange Rate Bot**\n\
    Version: `{}`\n\n\
    **Configuration**:\n\
    ```\n\
    - Exchange From: `{}`\n\
    - Exchange To: `{}`\n\
    - Interval: `{}` seconds\n\
    - Exchange Rate API: `{}`\n\
    - Ollama Model: `{}`\n\
    ```
    ",
        environment::APP_VERSION,
        environment::get_exchange_from(),
        environment::get_exchange_to(),
        environment::get_interval(),
        environment::get_exchange_rate_api_url(),
        environment::get_ollama_model(),
    );

    return CreateInteractionResponseMessage::new().content(content);
}
