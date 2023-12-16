# DNS Checker

This application requires several environment variables to be set in order to function correctly.

It queries the opnsense API to check which WAN ip is assigned and the corresponding address in DNS.

## Environment Variables

- `RUST_LOG`: This variable sets the logging level for the application. If not set, it defaults to `INFO`. Possible values are `ERROR`, `WARN`, `INFO`, `DEBUG`, and `TRACE`.

- `TELEGRAM_BOT_TOKEN`: This variable should be set to the token of your Telegram bot. This is used to authenticate your bot with the Telegram API.

- `CHAT_ID`: This variable should be set to the ID of the Telegram chat where the bot should send messages. You can get this ID by adding the bot to the chat and sending a message to the chat. The bot can then use the Telegram API to get the ID of the chat.

- `URL`: This variable should be set to the URL of the API that the application will make requests to.

- `API_KEY`: This variable should be set to the API key used for authenticating with the API.

- `API_SECRET`: This variable should be set to the API secret used for authenticating with the API.

- `DNS_HOSTNAME`: This variable should be set to the DNS hostname that will be looked up.

## Setting Environment Variables

Environment variables can be set in various ways depending on your operating system and shell. Here are examples for Bash and PowerShell:

### Bash

```bash
export RUST_LOG=INFO
export TELEGRAM_BOT_TOKEN=your_bot_token
export CHAT_ID=your_chat_id
export URL=your_api_url
export API_KEY=your_api_key
export API_SECRET=your_api_secret
export DNS_HOSTNAME=your_hostname
```