use anyhow::anyhow;
use serenity::all::ChannelPinsUpdateEvent;
use serenity::{async_trait, all::GuildChannel};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use rand::{Rng, seq::SliceRandom};
mod quotes;
mod jokes;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        //Get mad at MEE6
        if msg.author.id.get() == 159985870458322944 {
            if let Err(e) = msg.reply(&ctx.http, format!("{}", quotes::MEE6.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        //Dont respond to bots
        if msg.author.bot {
            return;
        }

        if msg.channel(&ctx).await.unwrap().guild().is_none() {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Stop messaging me, I'm {}!", quotes::BUSY.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }

        let msg_lower = msg.content.to_lowercase();
        //let has_admin = msg.author
        //Figure out how to tell if user is full admin on the server

        //Hodgey Help
        if msg_lower == "hodgey bot help" || msg_lower == "hodgey help" {
            if let Err(e) = msg.reply(&ctx.http, quotes::HELP_MESSAGE).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower == "hodgey joke" {
            let selected_joke = *jokes::JOKES.choose(&mut rand::thread_rng()).unwrap();
            
            for fields in selected_joke {
                let embed = CreateEmbed::new()
                    .title("Hodgey Joke")
                    .url("https://youtu.be/dQw4w9WgXcQ")
                    .colour(rand::thread_rng().gen_range(0..16777216))
                    .fields(fields.to_vec()); //I can probably avoid turning this into a vector, I have no clue what I am doing
            
                let builder = CreateMessage::new()
                    .embed(embed)
                    .reference_message(&msg);
                
                if let Err(e) = msg.channel_id.send_message(&ctx.http, builder).await {
                    error!("Error sending message: {e:?}");
                }
            }
        }
        //Hodgey Val agent
        else if msg_lower == "hodgey val agent" {
            if let Err(e) = msg.reply(&ctx.http, format!("{}", quotes::VAL_AGENTS.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower == "see" {
            if let Err(e) = msg.reply(&ctx.http, "said the blind man").await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower == "1+1" || msg_lower == "1 + 1" {
            if let Err(e) = msg.reply(&ctx.http, "Two!").await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.starts_with("spam") {
            let mut msg_parts = msg.content.splitn(3, ' ');
            let num_str = msg_parts.nth(1).unwrap();
            let num_repeats = num_str.parse::<usize>();
            if num_repeats.is_err() {
                if let Err(e) = msg.reply(&ctx.http, "You need to enter a number").await {
                    error!("Error sending message: {e:?}");
                }
                return;
            }
            let num_repeats = num_repeats.unwrap();
            if num_repeats > 5 {
                if let Err(e) = msg.channel_id.say(&ctx.http, "You are limited to 5").await {
                    error!("Error sending message: {e:?}");
                }
                return;
            }
            if let Some(contents) = msg_parts.next() {
                for _ in 0..num_repeats {
                    if let Err(e) = msg.channel_id.say(&ctx.http, contents).await {
                        error!("Error sending message: {e:?}");
                    }
                }
            }
        }
        else if msg.content.contains("@everyone") {
            if let Err(e) = msg.channel_id.say(&ctx.http, "Wow, you would ping @everyone!").await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg.content.contains("@here") {
             if let Err(e) = msg.channel_id.say(&ctx.http, "Wow, you would ping @here!").await {
                error!("Error sending message: {e:?}");
            }
        }
        //@Someone
        else if msg_lower.contains("@someone") {
            let channel_members = msg.guild_id.unwrap().members(&ctx.http, None, None).await.unwrap();
            let mut member = channel_members.choose(&mut rand::thread_rng()).unwrap();
            while member.user.bot {
                member = channel_members.choose(&mut rand::thread_rng()).unwrap();
            }
            if let Err(e) = msg.reply(&ctx.http, format!("{}", member.mention())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("hodgey decide") {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("{}", quotes::DECISION.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("chess") {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("{}", quotes::CHESS.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("checkers") {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("{}", quotes::CHECKERS.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("horse") {
            if let Err(e) = msg.reply(&ctx.http, "It's not a horse, it's a knight").await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("castle") {
            if let Err(e) = msg.reply(&ctx.http, "It's not a castle, it's a rook").await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("book") {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Have you read {}?", quotes::BOOKS.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("music") || msg_lower.contains("song") {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("{}", quotes::SONG_STARTS.choose(&mut rand::thread_rng()).unwrap()
                                                                                            .replace("SONG", quotes::SONGS.choose(&mut rand::thread_rng()).unwrap()))).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.contains("movie") {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Have you seen {}?", quotes::MOVIES.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if {let mut rng = rand::thread_rng(); rng.gen_range(0..100)} == 0 {
            if let Err(e) = msg.channel_id.say(&ctx.http, format!("{}", quotes::RANDOM.choose(&mut rand::thread_rng()).unwrap())).await {
                error!("Error sending message: {e:?}");
            }
        }
    }
    
    async fn channel_create(&self, ctx: Context, ch: GuildChannel) {
        if let Err(e) = ch.say(&ctx.http, format!("{}", quotes::NEW_CHANNEL.choose(&mut rand::thread_rng()).unwrap())).await {
            error!("Error sending message: {e:?}");
        }
    }

    async fn channel_pins_update(&self, ctx: Context, pin: ChannelPinsUpdateEvent) {
        if let Err(e) = pin.channel_id.say(&ctx.http, "Who is messing with the pinned messages?").await {
            error!("Error sending message: {e:?}");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        ctx.set_activity(Some(serenity::gateway::ActivityData::playing("Hodgey Help")));
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT
                //| GatewayIntents::GUILD_MEMBERS //| GatewayIntents::GUILD_PRESENCES
                | GatewayIntents::GUILDS | GatewayIntents::DIRECT_MESSAGES;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
