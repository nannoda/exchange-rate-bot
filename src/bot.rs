use serenity::all::{
    CommandDataOptionValue, CreateAttachment, CreateMessage, EditInteractionResponse,
};
use serenity::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::id::{ChannelId, GuildId};

use crate::utils::message::get_exchange_rate_message;
use crate::{commands, environment};

async fn send_exchange_rate_message(ctx: Arc<Context>, from: &str, to: &str) {
    let msg = get_exchange_rate_message(from, to).await;

    let mut message = CreateMessage::new().content(msg.message);

    // Conditionally add the file if `msg.graph` exists
    if let Some(graph) = msg.graph {
        message = message.add_file(CreateAttachment::bytes(graph, "graph.png"));
    }

    send_message(&ctx, &message).await;
}

struct ExchangeRateBotEventHandler {
    is_loop_running: AtomicBool,
}

async fn send_ready_message(ctx: &Context, ready: &serenity::model::gateway::Ready) {
    send_text(ctx, &format!("{} is back online!", ready.user.name)).await;
}

async fn send_message(ctx: &Context, message: &CreateMessage) {
    let channels = environment::get_channels();

    for channel in channels {
        // let channel_id = channel.as_u64().unwrap_or(0);
        log::info!("Channel id: {}", channel);

        let channel_id = ChannelId::new(channel);

        if let Err(why) = channel_id.send_message(&ctx.http, message.clone()).await {
            log::warn!("Error sending message: {:?}", why);
        }
    }
}

async fn send_text(ctx: &Context, text: &str) {
    send_message(ctx, &CreateMessage::new().content(text)).await
}

#[async_trait]
impl EventHandler for ExchangeRateBotEventHandler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        log::info!("{} is connected!", ready.user.name);
        send_ready_message(&ctx, &ready).await;

        let global_commands = Command::set_global_commands(
            &ctx.http,
            vec![
                commands::check_rate::register(),
                commands::about::register(),
            ],
        )
        .await;

        if let Err(why) = global_commands {
            log::warn!("Error creating global command: {:?}", why);
        }

        log::info!("Slash commands created");
    }

    // Called when a new interaction is created
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log::debug!("Interaction: {:?}", interaction);

        if let Interaction::Command(command) = &interaction {
            log::debug!("Received command interaction: {command:#?}");

            // Step 1: Acknowledge with a deferred response, allowing more time for processing
            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Defer(
                        CreateInteractionResponseMessage::new().content("Waiting..."),
                    ),
                )
                .await
            {
                log::warn!("Failed to acknowledge interaction: {why}");
                return; // Exit if the initial response fails
            }

            let content: Option<EditInteractionResponse> = match command.data.name.as_str() {
                commands::check_rate::COMMAND_NAME => {
                    Some(commands::check_rate::run(&command.data.options()).await)
                }
                commands::about::COMMAND_NAME => Some(commands::about::run()),
                _ => Some(EditInteractionResponse::new().content("not implemented :(".to_string())),
            };

            if let Some(content) = content {
                // Step 3: Edit the deferred response to send the final result
                if let Err(why) = command.edit_response(&ctx.http, content).await {
                    log::warn!("Cannot send final response to slash command: {why}");
                }
            }
        }

        if let Interaction::Autocomplete(autocomplete) = &interaction {
            if let Some(autocomplete_option) = autocomplete.data.options.iter().find_map(|option| {
                if let CommandDataOptionValue::Autocomplete { value, .. } = &option.value {
                    Some(value)
                } else {
                    None
                }
            }) {
                let complete_result = match autocomplete.data.name.as_str() {
                    commands::check_rate::COMMAND_NAME => {
                        Some(commands::check_rate::autocomplete(&autocomplete_option))
                    }
                    _ => None,
                };

                if let Some(complete_result) = complete_result {
                    let response = CreateInteractionResponse::Autocomplete(complete_result);
                    // Send the autocomplete response
                    if let Err(why) = autocomplete.create_response(&ctx.http, response).await {
                        log::warn!("Error creating autocomplete response: {:?}", why);
                    }
                }
            }
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        log::debug!("Cache built successfully!");

        // Get EXCHANGE_FROM and EXCHANGE_TO from environment
        let exchange_from = environment::get_exchange_from();
        let exchange_to = environment::get_exchange_to();

        // it's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
        let ctx = Arc::new(ctx);
        if !self.is_loop_running.load(Ordering::Relaxed) {
            // We have to clone the Arc, as it gets moved into the new thread.
            let ctx1 = Arc::clone(&ctx);
            // tokio::spawn creates a new green thread that can run in parallel with the rest of
            // the application.
            tokio::spawn(async move {
                loop {
                    // We clone Context again here, because Arc is owned, so it moves to the
                    // new function.
                    send_exchange_rate_message(Arc::clone(&ctx1), &exchange_from, &exchange_to)
                        .await;

                    let interval = environment::get_interval();
                    log::debug!("Interval: {}", interval);

                    tokio::time::sleep(Duration::from_secs(interval)).await;
                }
            });
            // Now that the loop is running, we set the bool to true
            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
}

pub async fn run_bot() {
    let token = environment::get_discord_token();

    log::debug!("Using token: {}", token);

    let intents = GatewayIntents::non_privileged();
    let mut client = Client::builder(token, intents)
        .event_handler(ExchangeRateBotEventHandler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        log::warn!("An error occurred while running the client: {:?}", why);
    }
}
