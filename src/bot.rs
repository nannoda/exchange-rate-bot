use serenity::all::CommandOptionType;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::application::{Command, Interaction};

use crate::{commands, environment};
use crate::llm::generate_sentence;
use crate::utils::{get_exchange_rate, get_exchange_rate_message, get_prompt};



async fn send_exchange_rate_message(ctx: Arc<Context>, from: &str, to: &str) {
    let message_content = get_exchange_rate_message(from, to).await;

    send_message(&ctx, &message_content).await;
}

struct ExchangeRateBotEventHandler {
    is_loop_running: AtomicBool,
}

async fn send_ready_message(ctx: &Context, ready: &serenity::model::gateway::Ready) {
    send_message(ctx, &format!("{} is back online!", ready.user.name)).await;
}

async fn send_message(ctx: &Context, content: &str) {
    let channels = environment::get_channels();

    for channel in channels {
        // let channel_id = channel.as_u64().unwrap_or(0);
        log::info!("Channel id: {}", channel);

        let channel_id = ChannelId::new(channel);

        let message = MessageBuilder::new()
            .push(content)
            .build();
        

        if let Err(why) = channel_id.say(&ctx.http, &message).await {
            log::warn!("Error sending message: {:?}", why);
        }
    }
}

#[async_trait]
impl EventHandler for ExchangeRateBotEventHandler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        log::info!("{} is connected!", ready.user.name);
        send_ready_message(&ctx, &ready).await;

        let global_command = Command::create_global_command(&ctx.http, commands::check_rate::register()).await;

        if let Err(why) = global_command {
            log::warn!("Error creating global command: {:?}", why);
        }
        log::info!("Slash commands created");
    }

    // Called when a new interaction is created
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log::debug!("Interaction: {:?}", interaction);

        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.as_str();
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
                    send_exchange_rate_message(Arc::clone(&ctx1), &exchange_from, &exchange_to).await;

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
