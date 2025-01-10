# Exchange Rate Bot
An AI-powered exchange rate bot that gives you updates on the exchange rate between two currencies of your choice.

Example configuration:

```yml
services:
  app:
    image: nannoda/exchange-rate-bot:latest # latest tag is not stable.
    restart: unless-stopped
    volumes:
    # By default the executable will be stored in /app
      - ./data:/app/data
    environment:
    # Required environment variables
      - DISCORD_TOKEN=${DISCORD_TOKEN} # DISCORD_TOKEN: The bot token for the discord bot. You can get it from the discord developer portal.
      - EXCHANGE_FROM=${EXCHANGE_FROM}  # EXCHANGE_FROM: The currency code to convert from. For example, USD. It should be an ISO 4217 currency code.
      - EXCHANGE_TO=${EXCHANGE_TO} # EXCHANGE_TO: The currency code to convert to. For example, EUR. It should be an ISO 4217 currency code.
    # Optional environment variables
      - DB_FILE=/app/data/bot.db # The path to the SQLite database file. By default it will be stored in /app/exchange_rate_bot.db
      - CHANNELS=${CHANNELS} # The channels to listen to. It should be a comma separated list of channel IDs. If not provided, the bot will not send any messages.
      - INTERVAL=${INTERVAL} # The interval to automatically send exchange rate updates. By default it is '24h'.
      - INCREASE_PROMPT_TEMPLATE=${INCREASE_PROMPT_TEMPLATE} # The template for the message to send when the exchange rate increases.
      - DECREASE_PROMPT_TEMPLATE=${DECREASE_PROMPT_TEMPLATE} # The template for the message to send when the exchange rate decreases.
      - EQUAL_PROMPT_TEMPLATE=${EQUAL_PROMPT_TEMPLATE} # The template for the message to send when the exchange rate stays the same.
      - EXCHANGE_RATE_CHANGE_THRESHOLD=${EXCHANGE_RATE_CHANGE_THRESHOLD} # The threshold for the exchange rate change. If the change is greater than this value, the bot will send a message. By default it is 0.001.
      - RUST_LOG=exchange_rate_bot=info # The log level for the bot.
      - EXCHANGE_RATE_API_URL=${EXCHANGE_RATE_API_URL} # The exchange rate API. Default to https://api.frankfurter.dev/v1. You can learn how to self host an API here: https://github.com/lineofflight/frankfurter
```
