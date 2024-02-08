use serenity::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::framework::standard::StandardFramework;
use serenity::model::id::{ChannelId, GuildId};

use crate::environment;
use crate::llm::generate_sentence;
use crate::utils::{get_exchange_rate, get_prompt};

async fn send_exchange_rate_message(ctx: Arc<Context>, from: &str, to: &str) {
    let rate_result = get_exchange_rate(from, to).await;

    if rate_result.is_err() {
        // send_message(&ctx, "Error getting exchange rate").await;
        send_message(
            &ctx,
            format!(
                "Error getting exchange rate: {}",
                rate_result.err().unwrap()
            )
            .as_str(),
        )
        .await;
        return;
    }

    let rate = rate_result.unwrap();

    let prompt = get_prompt(rate);

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

        let message = ChannelId(channel)
            .send_message(&ctx, |m: &mut CreateMessage| m.content(content))
            .await;

        if let Err(why) = message {
            log::warn!("Error sending message: {:?}", why);
        };
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        log::info!("{} is connected!", ready.user.name);
        send_ready_message(&ctx, &ready).await;
    }

    // case you have for this.
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        log::debug!("Cache built successfully!");

        // Get EXCHANGE_FROM and EXCHANGE_TO from environment
        let exchange_from = environment::get_exchange_from();
        let exchange_to = environment::get_exchange_to();

        // it's safe to clone Context, but Arc is cheaper for this use case.
        // Untested claim, just theoretically. :P
        let ctx = Arc::new(ctx);

        // We need to check that the loop is not already running when this event triggers,
        // as this event triggers every time the bot enters or leaves a guild, along every time the
        // ready shard event triggers.
        //
        // An AtomicBool is used because it doesn't require a mutable reference to be changed, as
        // we don't have one due to self being an immutable reference.
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
