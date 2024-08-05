use serenity::all::CommandOptionType;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::framework::standard::StandardFramework;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::application::{Command, Interaction};

use crate::environment;
use crate::llm::generate_sentence;
use crate::utils::{get_exchange_rate, get_prompt};

async fn get_exchange_rate_message(from: &str, to: &str) -> String {
    let rate_result = get_exchange_rate(from, to).await;

    if rate_result.is_err() {
        return format!("Error getting exchange rate from {} to {}", from, to);
    }

    let rate = rate_result.unwrap();
    // let rate: f64 = 0.0;

    let prompt = get_prompt(rate, from, to);

    // keep track how much time it takes to generate the sentence
    let start = std::time::Instant::now();

    let llm_res = generate_sentence(prompt.as_str());

    let res_without_prompt = llm_res.await;

    let elapsed = start.elapsed();

    let message_content = format!(
        "{}\n\
```
1 {} = {:.4} {}\n\
Generated in {}.{:03} seconds\n\
```",
        res_without_prompt, from, rate, to, elapsed.as_secs(), elapsed.subsec_millis(),
    );

    return message_content;
}

async fn send_exchange_rate_message(ctx: Arc<Context>, from: &str, to: &str) {
    let message_content = get_exchange_rate_message(from, to).await;

    send_message(&ctx, &message_content).await;
}

struct Handler {
    is_loop_running: AtomicBool,
}

async fn send_ready_message(ctx: &Context, ready: &serenity::model::gateway::Ready) {
    send_message(ctx, &format!("{} is back!", ready.user.name)).await;
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
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        log::info!("{} is connected!", ready.user.name);
        send_ready_message(&ctx, &ready).await;

        // create slash commands
        let check_ex_command = CreateCommand::new("check-ex")
            .description("Check exchange rate");

        let global_command = Command::create_global_command(&ctx.http, check_ex_command).await;

        if let Err(why) = global_command {
            log::warn!("Error creating global command: {:?}", why);
        }

        let custom_ex_check_command = CreateCommand::new("custom-ex-check")
            .description("Check exchange rate")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "from", "From currency")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "to", "To currency")
                    .required(true),
            );

        let custom_command = Command::create_global_command(&ctx.http, custom_ex_check_command).await;

        if let Err(why) = custom_command {
            log::warn!("Error creating custom command: {:?}", why);
        }

        log::info!("Slash commands created");
    }

    // Called when a new interaction is created
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        log::debug!("Interaction: {:?}", interaction);

        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.as_str();

            if command_name == "check-ex" {
                let exchange_from = environment::get_exchange_from();
                let exchange_to = environment::get_exchange_to();

                let exchange_rate_message = get_exchange_rate_message(&exchange_from, &exchange_to).await;

                let data = CreateInteractionResponseMessage::new()
                    .content(exchange_rate_message);

                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    log::warn!("Error creating interaction response: {:?}", why);
                }                
            }

            if command_name == "custom-ex-check" {
                let from = command.data.options.get(0).unwrap().value.as_str().unwrap();
                let to = command.data.options.get(1).unwrap().value.as_str().unwrap();

                log::debug!("from: {}, to: {}", from, to);

                let exchange_rate_message = get_exchange_rate_message(from, to).await;

                let data = CreateInteractionResponseMessage::new()
                    .content(exchange_rate_message);

                let builder = CreateInteractionResponse::Message(data);

                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    log::warn!("Error creating interaction response: {:?}", why);
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
    let framework = StandardFramework::new();

    let token = environment::get_discord_token();

    log::debug!("Using token: {}", token);

    let intents = GatewayIntents::non_privileged();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        log::warn!("An error occurred while running the client: {:?}", why);
    }
}
