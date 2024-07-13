use std::str::FromStr;

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
use chess::{Board, ChessMove, Color, BoardStatus};
mod quotes;
mod jokes;

const HODGEY_BOT_ID: u64 = 873373606900559943;

//Having this link hardcoded is bad, I should it fix later
const BLANK_CHESS_GAME_LINK: &str = "https://www.chess.com/dynboard?fen=rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR&board=bases&piece=classic&size=3&coordinates=1";

struct ChessGame {
    white_id: u64,
    black_id: u64,
    board: Board,
    show_coordinates: bool,
    board_flips: bool,
}

impl ChessGame {
    fn new_game_random_sides(player1_id: u64, player2_id: u64) -> Self {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.5) {
            Self {
                white_id: player1_id,
                black_id: player2_id,
                board: Board::default(),
                show_coordinates: true,
                board_flips: false,
            }
        }
        else {
            Self {
                white_id: player2_id,
                black_id: player1_id,
                board: Board::default(),
                show_coordinates: true,
                board_flips: false,
            }
        }
    }

    const fn has_user(&self, id: u64) -> bool {
        self.white_id == id || self.black_id == id
    }

    fn id_to_move(&self) -> u64 {
        match self.board.side_to_move() {
            Color::White => self.white_id,
            Color::Black => self.black_id
        }
    }

    fn make_move(&mut self, move_str: &str) -> Result<(), chess::Error> {
        let selected_move_result = match ChessMove::from_str(move_str) {
            Ok(selected_move) => {
                if self.board.legal(selected_move) {
                    Ok(selected_move)
                }
                else {
                    //The move is invalid if it is illegal
                    Err(chess::Error::InvalidUciMove)
                }
            }
            Err(_) => {
                //If the move is invalid UCI attempt to get SAN
                ChessMove::from_san(&self.board, move_str)
            }
        };

        return match selected_move_result {
            Ok(selected_move) => {
                self.board = self.board.make_move_new(selected_move);
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    fn to_link(&self) -> String {
        let board_str = self.board.to_string();
        let fen = board_str.split(' ').next().unwrap();
        let mut result = format!("https://www.chess.com/dynboard?fen={fen}&board=bases&piece=classic&size=3");

        if self.show_coordinates {
            result += "&coordinates=1";
        }

        if self.board_flips && self.board.side_to_move() == Color::Black {
            result += "&flip=1";
        }

        result
    }
}

struct ChessGames;

impl TypeMapKey for ChessGames {
    type Value = Mutex<Vec<ChessGame>>;
}

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
        // let has_admin = ?
        //Figure out how to tell if user is full admin on the server

        //Hodgey Help
        if msg_lower == "hodgey help" {
            if let Err(e) = msg.reply(&ctx.http, quotes::HELP_MESSAGE).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower == "hodgey help chess" {
            if let Err(e) = msg.reply(&ctx.http, quotes::CHESS_HELP_MESSAGE).await {
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
                    .fields(fields.to_vec()); //I can probably avoid turning this into a vector, I have no clue what I am doing :)
            
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
        else if msg_lower == "chess resign" || msg_lower ==  "chess surrender" {
            let rw_lock = ctx.data.read().await;
            let mut chess_games = rw_lock.get::<ChessGames>().expect("ChessGames not in TypeMap.").lock().await;
            chess_games.retain(|game| game.has_user(msg.author.id.get()));
        }
        else if msg_lower == "toggle coordinates" {
            let rw_lock = ctx.data.read().await;
            let mut chess_games = rw_lock.get::<ChessGames>().expect("ChessGames not in TypeMap.").lock().await;
            for game in chess_games.iter_mut() {
                if game.has_user(msg.author.id.get()) {
                    game.show_coordinates = !game.show_coordinates;
                    
                    let response = match game.show_coordinates {
                        true => "Coordinates enabled.",
                        false => "Coordinates disabled."
                    };

                    if let Err(e) = msg.reply(&ctx.http, response).await {
                        error!("Error sending message: {e:?}");
                    }
                    return;
                }
            }
            drop(chess_games); // drop mutex lock as soon as possible
            if let Err(e) = msg.reply(&ctx.http, quotes::NO_ACTIVE_CHESS_GAME).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower == "toggle board flip" {
            let rw_lock = ctx.data.read().await;
            let mut chess_games = rw_lock.get::<ChessGames>().expect("ChessGames not in TypeMap.").lock().await;
            for game in chess_games.iter_mut() {
                if game.has_user(msg.author.id.get()) {
                    game.board_flips = !game.board_flips;
                    
                    let response = match game.board_flips {
                        true => "Board flip enabled.",
                        false => "Board flip disabled."
                    };

                    if let Err(e) = msg.reply(&ctx.http, response).await {
                        error!("Error sending message: {e:?}");
                    }
                    return;
                }
            }
            drop(chess_games); // drop mutex lock as soon as possible
            if let Err(e) = msg.reply(&ctx.http, quotes::NO_ACTIVE_CHESS_GAME).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower == "chess show" {
            let rw_lock = ctx.data.read().await;
            let mut chess_games = rw_lock.get::<ChessGames>().expect("ChessGames not in TypeMap.").lock().await;
            for game in chess_games.iter_mut() {
                if game.has_user(msg.author.id.get()) {
                    if let Err(e) = msg.reply(&ctx.http, game.to_link()).await {
                        error!("Error sending message: {e:?}");
                    }
                    return;
                }
            }
            drop(chess_games); // drop mutex lock as soon as possible
            if let Err(e) = msg.reply(&ctx.http, quotes::NO_ACTIVE_CHESS_GAME).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.starts_with("chess new") {
            //Do this before locking mutex
            let author_id = msg.author.id.get();
            let opponent_id = if let Some(user) = msg.mentions.choose(&mut rand::thread_rng()) {
                user.id.get()
            }
            else {
                HODGEY_BOT_ID
            };

            let rw_lock = ctx.data.read().await;
            let mut chess_games = rw_lock.get::<ChessGames>().expect("ChessGames not in TypeMap.").lock().await;
            for game in chess_games.iter_mut() {
                if game.has_user(author_id) {
                    *game = ChessGame::new_game_random_sides(author_id, opponent_id);
                    if let Err(e) = msg.reply(&ctx.http, format!("New game created!\nWhite: <@{}>\nBlack: <@{}>", game.white_id, game.black_id)).await {
                        error!("Error sending message: {e:?}");
                    }
                    if let Err(e) = msg.channel_id.say(&ctx.http, game.to_link()).await {
                        error!("Error sending message: {e:?}");
                    }
                    return;
                }
            }
            let game = ChessGame::new_game_random_sides(author_id, opponent_id);
            let white_id = game.white_id;
            let black_id = game.black_id;
            chess_games.push(game);
            drop(chess_games); // drop mutex lock as soon as possible
            if let Err(e) = msg.reply(&ctx.http, format!("New game created!\nWhite: <@{white_id}>\nBlack: <@{black_id}>")).await {
                error!("Error sending message: {e:?}");
            }
            if let Err(e) = msg.channel_id.say(&ctx.http, BLANK_CHESS_GAME_LINK).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.starts_with("move ") {
            let move_str = msg.content.splitn(2, ' ').nth(1).unwrap();
            //stolen from https://stackoverflow.com/questions/57063777/remove-all-whitespace-from-a-string
            let move_str: String = move_str.chars().filter(|c| !c.is_whitespace()).collect();
            let author_id = msg.author.id.get();

            let rw_lock = ctx.data.read().await;
            let mut chess_games = rw_lock.get::<ChessGames>().expect("ChessGames not in TypeMap.").lock().await;
            for game in chess_games.iter_mut() {
                if game.has_user(author_id) {
                    //can't move on gameover
                    if game.board.status() != BoardStatus::Ongoing {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("The game has ended.")).await {
                            error!("Error sending message: {e:?}");
                        }
                        return;
                    }

                    if game.id_to_move() != author_id {
                        if let Err(e) = msg.reply(&ctx.http, "It is not your turn").await {
                            error!("Error sending message: {e:?}");
                        }
                        return;
                    }
                    if game.make_move(&move_str).is_err() {
                        if let Err(e) = msg.reply(&ctx.http, "I don't understand the move you are trying to make (or it is illegal)").await {
                            error!("Error sending message: {e:?}");
                        }
                        return
                    }
                    
                    let id_to_move = game.id_to_move();

                    //gameover
                    if game.board.status() != BoardStatus::Ongoing {
                        if game.board.status() == BoardStatus::Checkmate {
                            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Checkmate!")).await {
                                error!("Error sending message: {e:?}");
                            }
                        }
                        else {
                            if let Err(e) = msg.channel_id.say(&ctx.http, format!("Stalemate!")).await {
                                error!("Error sending message: {e:?}");
                            }
                        }
                        return;
                    }

                    if id_to_move == HODGEY_BOT_ID {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("It is my turn, but I don't know how to play chess yet :(")).await {
                            error!("Error sending message: {e:?}");
                        }
                        return;
                    }
                    
                    //Check if the player is in check
                    if game.board.checkers().popcnt() != 0 {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("You are in check <@{id_to_move}>!")).await {
                            error!("Error sending message: {e:?}");
                        }
                    }
                    else {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("Your turn <@{id_to_move}>!")).await {
                            error!("Error sending message: {e:?}");
                        }
                    }
                    
                    if let Err(e) = msg.channel_id.say(&ctx.http, game.to_link()).await {
                        error!("Error sending message: {e:?}");
                    }

                    return;
                }
            }
            drop(chess_games); // drop mutex lock as soon as possible
            if let Err(e) = msg.reply(&ctx.http, quotes::NO_ACTIVE_CHESS_GAME).await {
                error!("Error sending message: {e:?}");
            }
        }
        else if msg_lower.starts_with("spam ") {
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
            else {
                if let Err(e) = msg.channel_id.say(&ctx.http, "I don't understand your message").await {
                    error!("Error sending message: {e:?}");
                }
            }
        }
        else if msg.content.contains("@everyone") {
            if let Err(e) = msg.reply(&ctx.http, "Wow, you would ping @everyone!").await {
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
        .type_map_insert::<ChessGames>(Mutex::new(Vec::new()))
        .await
        .expect("Err creating client");

    Ok(client.into())
}