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
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::{Args, Delimiter};
use serenity::model::channel::Reaction;
use serenity::model::channel::ReactionType;
use serenity::model::id::ChannelId;

use parking_lot;

use lazy_static::lazy_static;

const ONE: &str = "1\u{20e3}";
const TWO: &str = "2\u{20e3}";
const THREE: &str = "3\u{20e3}";
const FOUR: &str = "4\u{20e3}";
const FIVE: &str = "5\u{20e3}";
const BOT_ID: u64 = 629418348601540609;

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
    db: parking_lot::Mutex<database::Database>,
    pictures_to_handle: parking_lot::RwLock<HashMap<u64, i32>>,
}
impl Handler {
    fn new(db: database::Database) -> Handler {
        Self {
            db: parking_lot::Mutex::new(db),
            pictures_to_handle: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    // send out a picture to the server. add reactions through the use of
    // serenity::builder::CreateMessage, and .reactions()
    fn send_picture(&self, ctx: &Context, channel_id: &ChannelId, author_discord_id: &u64) {
        let id = *author_discord_id as i64;

        let db = self.db.lock();

        let picture = match db.fetch_file_for_user(&id) {
            Ok(file) => file,
            Err(e) => {
                println! {"there was an error fetching a new file for user {}\nERROR:\t{:?}", &author_discord_id, e};
                return ();
            }
        };

        // get path of the file we are sending
        let path = picture.file_path();

        // discord text message content
        let title = format! {"{} from {}", picture.character(), picture.anime()};

        // send the message to the server
        let msg = channel_id.send_message(&ctx.http, |m| {
            m.content(title);
            m.add_file(AttachmentType::Path(Path::new(&path)));
            m.reactions(vec![ONE, TWO, THREE, FOUR, FIVE]);
            m
        });

        match msg {
            Ok(good_message) => {
                // write the message ID we just sent out to a hash map.
                // This is done so that we can retrieve what picture is stored in each picture
                // based on what message ID is used
                let id = good_message.id.as_u64();

                let mut hash_map = self.pictures_to_handle.write();

                hash_map.insert(*id, *picture.file_id());
            }
            Err(e) => {
                println! {"error sending a picture message out : {:?}", e}
                return ();
            }
        }
    }
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {

        let user_id = msg.author.id.as_u64();
        if *user_id == BOT_ID {
            return ()
        }

        dbg! {"message recieved"};

        // wrap message text in Args for parsing
        let args = Args::new(&msg.content, &[Delimiter::Single(' ')]);

        // parse the first item in the string to check what command is being run
        let first = match args.parse::<String>() {
            Ok(parsed) => parsed,
            Err(e) => {
                println! {"error with parsing command: {:?}",e }
                return ();
            }
        };

        // if first arg is vote
        if first == "!vote" {
            self.send_picture(&ctx, &msg.channel_id, user_id);
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    // when someone reacts, check if that person was mean to react to it in the hashmap
    // if they were, write that rating to the database
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {

        // if the reaction was made by us it doesnt matter
        let user_id = reaction.user_id.as_u64();
        if *user_id == BOT_ID {
            return ();
        }

        // make sure the emoji is a unicode one and not custom
        let emoji = match &reaction.emoji {
            ReactionType::Unicode(string) => string,
            _ => return (),
        };

        // parse a score from the emoticons
        // TODO : unicode parsing here instead would be faster;
        let score = if emoji == ONE {
            1
        } else if emoji == TWO {
            2
        } else if emoji == THREE {
            3
        } else if emoji == FOUR {
            4
        } else if emoji == FIVE {
            5
        } else {
            return ();
        };

        // scoping for mutexes
        {
            // lock the database mutex
            let db = self.db.lock();

            let user_id_i64 = *user_id as i64;

            // TODO: cache the fact that this person exists in the database
            // check if we need to create a new user reaction for this person
            match db.need_new_user(&user_id_i64) {
                Ok(need_user) => {
                    // fetch the username of the author
                    let author = match reaction.user_id.to_user(&ctx) {
                        Ok(username) => username.name,
                        Err(e) => {
                            println! {"there was an error creating a new user: {:?}", e}
                            return ();
                        }
                    };

                    if need_user == true {
                        db.create_user(&author, &user_id_i64);
                    }
                }
                Err(e) => {
                    println! {"error checking if a user needs to be created: {:?}", e}
                    return ();
                }
            }

            // acquire read access to hashmap
            let hm = self.pictures_to_handle.read();
            match hm.get(reaction.message_id.as_u64()) {
                Some(id) => {
                    db.insert_new_rating(user_id_i64, *id, score);
                }
                None => {
                    println! {"PROBLEM!!!! message id was not found in the hashmap"}
                    return ();
                }
            }
            
        }
        // hashmap and database mutex dropped


        self.send_picture(&ctx, &reaction.channel_id, user_id);
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
