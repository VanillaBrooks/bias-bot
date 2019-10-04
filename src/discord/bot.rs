use super::super::data::database;
use super::super::error::Error;
use serde::Deserialize;
use serde_yaml;

use std::collections::HashMap;
use std::fs;
use std::path;

// use serenity;

use std::{env, path::Path};

use serenity::{
    http::AttachmentType,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, Delimiter};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Reaction;

use parking_lot;

#[derive(Deserialize)]
struct Auth {
    discord_token: String,
}


/// Starts the discord bot
pub fn start_bot(config_path: &path::Path, database: database::Database) -> Result<(), Error> {
    // fetch the discord token from config file
    let reader = fs::File::open(&config_path)?;
    let token: Auth = serde_yaml::from_reader(reader)?;

    // make a new handler that holds the database
    let handler = Handler::new(database);

    // Configure the client with your Discord bot token in the environment.
    let mut client = Client::new(token.discord_token, handler).expect("Err creating client");

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }

    Ok(())
}


struct Handler {
    db: parking_lot::RwLock<database::Database>,
    pictures_to_handle: HashMap<String, String>,
}
impl Handler {
    fn new(db: database::Database) -> Handler {
        Self {
            db: parking_lot::RwLock::new(db),
            pictures_to_handle: HashMap::new(),
        }
    }

    // send out a picture to the server. add reactions through the use of 
    // serenity::builder::CreateMessage, and .reactions()
    fn send_picture(&self, ctx: &Context, msg: &Message, args: Args) {
        let author = &msg.author.name;
        dbg! {&author};
        // msg.channel_id.send_message( &ctx.http, |arg| {
        //     arg.title()

        // }

        // )
    }
}

impl EventHandler for Handler {

    fn message(&self, ctx: Context, msg: Message) {
        dbg! {"message recieved"};

        // wrap message text in Args for parsing
        let args = Args::new(&msg.content, &[Delimiter::Single(' ')]);

        // parse the first item in the string to check what command is being run
        let first = match args.parse::<String>() {
            Ok(parsed) => parsed,
            Err(e) => {
                println!{"error with parsing command: {:?}",e }
                return ()
            }
        };


        // if first arg is vote
        if first == "!vote" {
            self.send_picture(&ctx, &msg, args);
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    // when someone reacts, check if that person was mean to react to it in the hashmap
    // if they were, write that rating to the database
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {

    }
}


// let msg = msg.channel_id.send_message(&ctx.http, |m| {
//     m.content("Hello, World!");
//     m.embed(|e| {
//         e.title("This is a title");
//         e.description("This is a description");
//         e.image("attachment://downloaded_data/2163536056233450499.png.png");
//         e.fields(vec![
//             ("This is the first field", "This is a field body", true),
//             (
//                 "This is the second field",
//                 "Both of these fields are inline",
//                 true,
//             ),
//             (
//                 "This is the fourth field",
//                 "Both of these fields are inline",
//                 true,
//             ),
//         ]);
//         e.field(
//             "This is the third field",
//             "This is not an inline field",
//             false,
//         );
//         e.footer(|f| {
//             f.text("This is a footer");

//             f
//         });

//         e
//     });
//     // m.add_file(AttachmentType::Path(Path::new("./downloaded_data/2163536056233450499.png")));
//     m
// });