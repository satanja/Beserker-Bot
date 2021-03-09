mod api;
mod bout;

use bout::Bout;
use std::collections::HashMap;
use std::env;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

/// The message handler. Contains the list of Discord commands and the internal
/// state of all the bouts.
struct Handler {
    discord_commands: DiscordCommands,
    state: State,
}

/// Actions the bot can perform.
enum InternalCommand {
    /// Removes a player from a tournament bout, given a tournament and team id.
    Remove(usize, usize),

    /// Adds a player to a trounament bout, given a tournament and team id.
    Insert(usize, usize),

    /// Polls the API given a tournament and team id.
    Poll(usize, usize),
}

#[derive(Debug)]
/// Additional arguments to process internal commands
enum Arguments {
    /// Removes a player at a specified index
    Remove(usize),

    /// Insert a player given by a String at a specified index
    Insert(String, usize),
}

/// Dynamic list of Discord commands.
struct DiscordCommands {
    commands: HashMap<String, InternalCommand>,
}

impl DiscordCommands {
    /// Constructs a new empty list of Discord commands.
    pub fn new() -> DiscordCommands {
        DiscordCommands {
            commands: HashMap::new(),
        }
    }

    /// Adds a new command to the bot. Note, `discord_command` should not be
    /// prefixed.
    pub fn add_command(&mut self, discord_command: String, command: InternalCommand) {
        self.commands.insert(discord_command, command);
    }

    /// Removes a command from the bot. Note, `discord_command` should not be
    /// prefixed.
    pub fn remove_command(&mut self, discord_command: &str) {
        self.commands.remove(discord_command);
    }

    /// Gets the associated command given the string. Note, `discord_command`
    /// should not be prefixed.
    pub fn get(&self, discord_command: &str) -> Option<&InternalCommand> {
        self.commands.get(discord_command)
    }
}

/// Data structure to keep track of all the active bouts per tournament per
/// team.
struct State {
    bouts: HashMap<(usize, usize), Bout>,
}

impl State {
    pub fn new() -> State {
        State {
            bouts: HashMap::new(),
        }
    }

    /// Handles the command and updates internal state if necessary.
    pub fn process(
        &mut self,
        command: &InternalCommand,
        args: Option<Arguments>,
    ) -> Result<(), String> {
        match command {
            InternalCommand::Remove(tournament_id, team_id) => {
                self.remove(*tournament_id, *team_id, args)
            }
            InternalCommand::Insert(tournament_id, team_id) => {
                self.insert(*tournament_id, *team_id, args)
            }
            InternalCommand::Poll(tournament_id, team_id) => {
                self.poll(*tournament_id, *team_id, args)
            }
        }
    }

    /// Removes a player from a bout, identified by `tournament_id` and 
    /// `team_id`, at a specified index. Requires `args` to be 
    /// `Some(Arguments::Remove(index))`. In case `args` is incorrect, return
    /// an appropriate error.
    fn remove(
        &mut self,
        tournament_id: usize,
        team_id: usize,
        args: Option<Arguments>,
    ) -> Result<(), String> {
        match args {
            Some(Arguments::Remove(index)) => Ok(()),
            Some(op) => Err(format!("received {:?}, expected remove of map index", op)),
            None => Err("received no arguments, expected map index".to_string()),
        }
    }

    /// Inserts a player into a bout, identified by `tournament_id` and 
    /// `team_id`, at a specified index. Requires `args` to be 
    /// `Some(Arguments::Remove(player, index))`. In case `args` is incorrect, return
    /// an appropriate error.
    fn insert(
        &mut self,
        tournament_id: usize,
        team_id: usize,
        args: Option<Arguments>,
    ) -> Result<(), String> {
        match args {
            Some(Arguments::Insert(player, index)) => Ok(()),
            Some(op) => Err(format!(
                "received {:?}, expected insert of player name and map index",
                op
            )),
            None => Err("received no arguments, insert of player name and map index".to_string()),
        }
    }

    /// Polls the API for the next bout identified by `tournament_id` and 
    /// `team_id`. Forwards any error of the API.
    fn poll(
        &mut self,
        tournament_id: usize,
        team_id: usize,
        args: Option<Arguments>,
    ) -> Result<(), String> {
        Ok(())
    }
}

impl Handler {
    pub fn new() -> Handler {
        let discord_commands = DiscordCommands::new();
        let state = State::new();
        Handler {
            discord_commands,
            state,
        }
    }
}

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

#[tokio::main]
async fn main() {
    // // Configure the client with your Discord bot token in the environment.

    // let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");

    // // Create a new instance of the Client, logging in as a bot. This will
    // // automatically prepend your bot token with "Bot ", which is a requirement
    // // by Discord for bot users.

    // let handler = Handler::new();

    // let mut client = Client::builder(&token)
    //     .event_handler(handler)
    //     .await
    //     .expect("Err creating client");

    // // Finally, start a single shard, and start listening to events.
    // //
    // // Shards will automatically attempt to reconnect, and will perform
    // // exponential backoff until it reconnects.
    
    // if let Err(why) = client.start().await {
    //     println!("Client error: {:?}", why);
    // }
}
