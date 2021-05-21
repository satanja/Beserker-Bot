mod api;
mod bout;
mod response;

use bout::Bout;
use response::{Response, ResponseType};
use std::env;
use std::{collections::HashMap, usize};

use serenity::{
    async_trait,
    http::Http,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::Colour,
};

use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};

/// The message handler. Contains the list of Discord commands and the internal
/// state of all the bouts.
struct Handler;

/// Actions the bot can perform.
#[derive(Debug)]
enum InternalCommand {
    /// Removes a player from a tournament bout, given a tournament and team id.
    Remove(usize, usize),

    /// Adds a player to a trounament bout, given a tournament and team id.
    Insert(usize, usize),
}

#[derive(Debug)]
/// Additional arguments to process internal commands
enum Arguments {
    #[allow(dead_code)]
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
    pub fn remove_command(&mut self, discord_command: &str) -> Option<InternalCommand> {
        self.commands.remove(discord_command)
    }

    /// Gets the associated command given the string. Note, `discord_command`
    /// should not be prefixed.
    pub fn get(&self, discord_command: &str) -> Option<&InternalCommand> {
        self.commands.get(discord_command)
    }
}

/// Data structure to keep track of all the active bouts per tournament per
/// team.
struct Processor {
    bouts: HashMap<(usize, usize), Bout>,
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            bouts: HashMap::new(),
        }
    }

    /// Handles the command and updates internal state if necessary.
    pub async fn process(
        &mut self,
        ctx: &Context,
        msg: &Message,
        command: &InternalCommand,
        args: Option<Arguments>,
    ) {
        match command {
            InternalCommand::Remove(tournament_id, team_id) => {
                self.remove(*tournament_id, *team_id, ctx, msg, args).await;
            }
            InternalCommand::Insert(tournament_id, team_id) => {
                self.insert(*tournament_id, *team_id, ctx, msg, args).await;
            }
        }
    }

    pub fn drop_entry(&mut self, id: (usize, usize)) -> Option<Bout> {
        self.bouts.remove(&id)
    }

    /// Removes a player from a bout, identified by `tournament_id` and
    /// `team_id`, at a specified index. Requires `args` to be
    /// `Some(Arguments::Remove(index))`. In case `args` is incorrect, write
    /// an appropriate error.
    async fn remove(
        &mut self,
        tournament_id: usize,
        team_id: usize,
        ctx: &Context,
        msg: &Message,
        args: Option<Arguments>,
    ) {
        match args {
            Some(op) => match op {
                Arguments::Remove(index) => match self.bouts.get_mut(&(tournament_id, team_id)) {
                    Some(bout) => {
                        if let Err(why) = bout.remove_player(index) {
                            let status = send_message_embed(why, msg, &ctx.http).await;
                            if let Err(why) = status {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                    }
                    None => {
                        let title = String::from("No active matches found");
                        let message = format!(
                            "For further information see https://spire.gg/tournament/{}#brackets.",
                            tournament_id
                        );
                        let response = Response::new_error(title, message);
                        let status = send_message_embed(response, msg, &ctx.http).await;
                        if let Err(why) = status {
                            println!("Error sending message: {:?}", why);
                        }
                        return;
                    }
                },
                _ => {}
            },
            None => {
                let title = String::from("Missing map number");
                let text = String::from("Please specify which map to remove a player from");
                let response = Response::new_error(title, text);

                let status = send_message_embed(response, msg, &ctx.http).await;
                if let Err(why) = status {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }
        }
        let bout = self.bouts.get(&(tournament_id, team_id)).unwrap();

        let status = send_bout_embed(msg, &ctx.http, &bout).await;
        if let Err(why) = status {
            println!("Error sending message: {:?}", why);
        }
    }

    /// Inserts a player into a bout, identified by `tournament_id` and
    /// `team_id`, at a specified index. Requires `args` to be
    /// `Some(Arguments::Remove(player, index))`. In case `args` is incorrect, write
    /// an appropriate error.
    async fn insert(
        &mut self,
        tournament_id: usize,
        team_id: usize,
        ctx: &Context,
        msg: &Message,
        args: Option<Arguments>,
    ) {
        // first update the bout / insert a new bout
        if let Some(bout) = self.bouts.get_mut(&(tournament_id, team_id)) {
            let result = api::find_next_bout(tournament_id, team_id).await;

            match result {
                Ok(next_bout) => {
                    if *bout != next_bout {
                        *bout = next_bout;
                    }
                }
                Err(why) => {
                    let status = send_message_embed(why, msg, &ctx.http).await;
                    if let Err(why) = status {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            }
        } else {
            let result = api::find_next_bout(tournament_id, team_id).await;
            match result {
                Ok(next_bout) => {
                    self.bouts.insert((tournament_id, team_id), next_bout);
                }
                Err(why) => {
                    let status = send_message_embed(why, msg, &ctx.http).await;
                    if let Err(why) = status {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            }
        }

        // no errors so we can unwrap safely
        let bout = self.bouts.get_mut(&(tournament_id, team_id)).unwrap();

        match args {
            Some(Arguments::Insert(player, index)) => {
                if let Err(why) = bout.insert_player(index, player) {
                    let status = send_message_embed(why, msg, &ctx.http).await;
                    if let Err(why) = status {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
            _ => {}
        }

        let status = send_bout_embed(msg, &ctx.http, &bout).await;
        if let Err(why) = status {
            println!("Error sending message: {:?}", why);
        }
    }
}

/// Simple wrapper which is dumped in the context data. The wrapper is nice
/// to simplify ownership details.
struct Wrapper {
    commands: DiscordCommands,
    processor: Processor,
}

impl Wrapper {
    /// Wrapper constructor.
    pub fn new() -> Wrapper {
        Wrapper {
            commands: DiscordCommands::new(),
            processor: Processor::new(),
        }
    }
}

impl TypeMapKey for Wrapper {
    type Value = Self;
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.

    /// Handles processing of custom commands.
    async fn message(&self, ctx: Context, msg: Message) {
        // Extract the state from the context
        let mut data = ctx.data.write().await;
        let wrapper = data.get_mut::<Wrapper>().unwrap();
        let commands = &wrapper.commands;

        // Ensure that the command does not overlap with the admin commands
        if !msg.content.starts_with('!')
            || msg.content.starts_with("!add_command")
            || msg.content.starts_with("!remove_command")
        {
            return;
        }

        let words = get_msg_words(&msg.content);

        // message is by definition non-empty
        let command = words[0];

        // ignore if the message is not a command
        if !command.starts_with('!') {
            return;
        }

        // See if the command has been declared
        match commands.get(&command) {
            Some(x) => match x {
                InternalCommand::Insert(_, _) => {
                    let processor = &mut wrapper.processor;

                    let words = get_msg_words(&msg.content);
                    let args = match words.len() {
                        1 => None,
                        0 => panic!("does not happen"),
                        _ => {
                            let username = msg.author.name.clone();
                            let index: usize;
                            match words[1].parse::<usize>() {
                                Ok(num) => index = num,
                                Err(why) => {
                                    let title =
                                        String::from("Please enter a whole positive number");
                                    let text = why.to_string();
                                    let response = Response::new_error(title, text);
                                    if let Err(why) =
                                        send_message_embed(response, &msg, &ctx.http).await
                                    {
                                        println!("Error sending message: {:?}", why);
                                    }
                                    return;
                                }
                            }

                            Some(Arguments::Insert(username, index))
                        }
                    };

                    // run the command
                    processor.process(&ctx, &msg, x, args).await;
                }
                InternalCommand::Remove(_, _) => {
                    let processor = &mut wrapper.processor;
                    let words = get_msg_words(&msg.content);

                    let args = match words.len() {
                        1 => None,
                        0 => panic!("does not happen"),
                        _ => {
                            let index: usize;
                            match words[1].parse::<usize>() {
                                Ok(num) => index = num,
                                Err(why) => {
                                    let title =
                                        String::from("Please enter a whole positive number");
                                    let text = why.to_string();
                                    let response = Response::new_error(title, text);
                                    if let Err(why) =
                                        send_message_embed(response, &msg, &ctx.http).await
                                    {
                                        println!("Error sending message: {:?}", why);
                                    }
                                    return;
                                }
                            }
                            Some(Arguments::Remove(index))
                        }
                    };
                    processor.process(&ctx, &msg, x, args).await;
                }
            },
            None => {
                // do nothing when the command is not recognized
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

#[group]
#[commands(add_command, remove_command)]
struct Admin;

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to '!'
        .group(&ADMIN_GROUP);

    // Login with a bot token from the environment
    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    // add context data structures
    let wrapper = Wrapper::new();
    let mut data = client.data.write().await;
    data.insert::<Wrapper>(wrapper);
    drop(data);

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
// Syntax: !add_command <new_command> <action> [args]
async fn add_command(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let wrapper = data.get_mut::<Wrapper>().unwrap();
    let commands = &mut wrapper.commands;

    let words = get_msg_words(&msg.content);

    if words.len() < 3 {
        let title = String::from("Not enough identifier");
        let text = String::from("Expected `!add_command <new_command> <action> [args]`");
        let response = Response::new_error(title, text);
        return send_message_embed(response, msg, &ctx.http).await;
    }

    // prefix the new command with the identifier if the user hasn't done that
    let new_command = if words[1].starts_with('!') {
        String::from(words[1])
    } else {
        format!("!{}", words[1])
    };

    let action = words[2];

    match action.to_lowercase().as_ref() {
        "insert" => {
            // !add_command <new_command> <action> <team_id> <tournament_id>
            if words.len() < 5 {
                let title = String::from("Not enough arguments");
                let text = format!("Expected 5 arguments, received {}", words.len() - 1);
                let response = Response::new_error(title, text);
                return send_message_embed(response, msg, &ctx.http).await;
            }

            let team_id: usize;
            match words[3].parse::<usize>() {
                Ok(num) => team_id = num,
                Err(why) => {
                    let title = String::from("Please enter a whole positive number");
                    let text = why.to_string();
                    let response = Response::new_error(title, text);
                    return send_message_embed(response, msg, &ctx.http).await;
                }
            }

            let tournament_id: usize;
            match words[4].parse::<usize>() {
                Ok(num) => tournament_id = num,
                Err(why) => {
                    let title = String::from("Please enter a whole positive number");
                    let text = why.to_string();
                    let response = Response::new_error(title, text);
                    return send_message_embed(response, msg, &ctx.http).await;
                }
            }

            commands.add_command(
                new_command.clone(),
                InternalCommand::Insert(tournament_id, team_id),
            );
        }

        "remove" => {
            if words.len() < 5 {
                let title = String::from("Invalid command");
                let text = format!("Expected 5 arguments, received {}", words.len() - 1);
                let response = Response::new_error(title, text);
                return send_message_embed(response, msg, &ctx.http).await;
            }

            let team_id: usize;
            match words[3].parse::<usize>() {
                Ok(num) => team_id = num,
                Err(why) => {
                    let title = String::from("Please enter a whole positive number");
                    let text = why.to_string();
                    let response = Response::new_error(title, text);
                    return send_message_embed(response, msg, &ctx.http).await;
                }
            }

            let tournament_id: usize;
            match words[4].parse::<usize>() {
                Ok(num) => tournament_id = num,
                Err(why) => {
                    let title = String::from("Please enter a whole positive number");
                    let text = why.to_string();
                    let response = Response::new_error(title, text);
                    return send_message_embed(response, msg, &ctx.http).await;
                }
            }

            commands.add_command(
                new_command.clone(),
                InternalCommand::Remove(tournament_id, team_id),
            );
        }

        _ => {
            let title = String::from("Invalid command");
            let text = String::from("Expected: `!add_command <new_command> <action> [args].`\nInvalid action, please use one of `insert`, `remove`, or `poll`.");
            let response = Response::new_error(title, text);
            return send_message_embed(response, msg, &ctx.http).await;
        }
    }

    let title = String::from("Added command");
    let text = format!("Sucessfully added command `{}`.", new_command);
    let response = Response::new_success(title, text);

    send_message_embed(response, msg, &ctx.http).await
}

#[command]
// Syntax: !remove_command <command>
async fn remove_command(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let wrapper = data.get_mut::<Wrapper>().unwrap();
    let commands = &mut wrapper.commands;

    let words = get_msg_words(&msg.content);

    if words.len() < 2 {
        let title = String::from("Not enough arguments");
        let text = String::from("Usage: !remove_command <command_name>");
        let response = Response::new_error(title, text);
        return send_message_embed(response, msg, &ctx.http).await;
    }

    // prefix the command
    let command = if words[1].starts_with('!') {
        String::from(words[1])
    } else {
        format!("!{}", words[1])
    };

    match commands.remove_command(&command) {
        Some(internal_command) => {
            match internal_command {
                InternalCommand::Insert(team_id, tournament_id) => {
                    let processor = &mut wrapper.processor;
                    processor.drop_entry((team_id, tournament_id));
                }
                _ => {}
            }

            let text = format!("Succesfully removed command `{}`.", &command);
            let response = Response::new_success(String::from("Removed command"), text);
            return send_message_embed(response, msg, &ctx.http).await;
        }
        None => {
            let text = format!("The command `{}` could not be found.", &command);
            let response = Response::new_warning(String::from("Command not found"), text);
            return send_message_embed(response, msg, &ctx.http).await;
        }
    }
}

/// Split message contents by `' '`.
fn get_msg_words(contents: &str) -> Vec<&str> {
    contents.split(" ").collect()
}

async fn send_message_embed(response: Response, msg: &Message, http: &Http) -> CommandResult {
    let color = match response.response_type {
        ResponseType::Error => Colour::RED,
        ResponseType::Success => Colour::DARK_GREEN,
        ResponseType::Warning => Colour::ORANGE,
    };

    msg.channel_id
        .send_message(http, |m| {
            m.embed(|e| {
                e.title(response.title);
                e.description(response.contents);
                e.color(color);
                e
            });
            m
        })
        .await?;
    Ok(())
}

/// Generate an embed of the bout to send to the user(s).
async fn send_bout_embed(msg: &Message, http: &Http, bout: &Bout) -> CommandResult {
    msg.channel_id
        .send_message(http, |m| {
            m.embed(|e| {
                e.title(bout.get_title());
                e.description(bout.get_description());
                e.field("Maps", bout.get_maps(), false);
                e.color(Colour::BLITZ_BLUE);
                e
            });
            m
        })
        .await?;
    Ok(())
}
