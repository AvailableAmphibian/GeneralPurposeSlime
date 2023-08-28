use std::env;
use std::env::VarError;
use std::fmt::{Debug, Display, Formatter};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{debug, error, Level, trace};
use tracing_subscriber::filter::LevelFilter;
use serenity::Error as SerenityError;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

enum SlimeError {
    Var(VarError),
    Serenity(SerenityError),
}

impl Debug for SlimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_debug_msg = match self {
            SlimeError::Var(err) => { format!("Slime says: \"Variable error: {err:?}\"")}
            SlimeError::Serenity(err) => { format!("Slime says: \"Serenity error: {err:?}\"")}
        };
        write!(f, "{err_debug_msg}")
    }
}

impl Display for SlimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_display_msg = match self {
            SlimeError::Var(err) => { format!("SlimeError::Var({err})")}
            SlimeError::Serenity(err) => { format!("SlimeError::SerenityError({err})")}
        };write!(f, "{err_display_msg}")
    }
}

impl std::error::Error for SlimeError {}

#[tokio::main]
async fn main() -> Result<(), SlimeError> {
    let level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    init_tracing(level);

    trace!("Beginning everything. Now retrieving the DISCORD_TOKEN...");

    // Configure the client with your Discord bot token in the environment.
    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => {
            debug!("Here's your token: {token}");
            token
        }
        Err(l_error) => {
            error!("Couldn't retrieve the token: {l_error:?}");
            return Err(SlimeError::Var(l_error));
        }
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    trace!("Intent selected: {intents:?}");

    let mut client = match Client::builder(&token, intents)
        // .event_handler(Handler)
        .await {
        Ok(client) => { client }
        Err(l_error) => {
            error!("Error occurred while creating client: {l_error:?}");
            return Err(SlimeError::Serenity(l_error));
        }
    };

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}", );
    }

    Ok(())
}


fn init_tracing(filter: impl Into<LevelFilter>) {
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .init()
}
