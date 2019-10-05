#![allow(dead_code)]


use serde::{Deserialize, Serialize};
use serde_yaml;

use std::fs;
use std::io;
use std::sync::Arc;

use super::super::error::Error;
use postgres;

use std::collections::HashSet;

use std::path;

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseConfig {
    address: String,
    port: u32,
    database_name: String,
    username: String,
    password: String,
}

impl DatabaseConfig {
    fn new() -> Result<Self, Error> {
        let path = r".\config.yaml";

        let file = fs::File::open(&path).expect("config.yaml DOES NOT EXIST");
        let reader = io::BufReader::new(file);

        Ok(serde_yaml::from_reader(reader)?)
    }
    fn connection_url(&self) -> String {
        format! {"postgresql://{}:{}@{}:{}/{}",self.username, self.password, self.address, self.port, self.database_name}
    }
}

#[derive(Debug)]
pub struct Database {
    conn: postgres::Connection,
    save_folder: String,
}

// safe since Database will be in a mutex later
unsafe impl std::marker::Sync for Database {}

impl Database {
    pub fn new(save_path: String) -> Result<Self, Error> {
        let config = DatabaseConfig::new()?;
        let url = config.connection_url();
        let connection = postgres::Connection::connect(url, postgres::TlsMode::None)?;

        Ok(Self {
            conn: connection,
            save_folder: save_path,
        })
    }

    pub fn get_all_urls(&self) -> Result<HashSet<String>, Error> {
        let pull = self.conn.prepare("SELECT link FROM files")?;

        let data = pull.query(&[])?;

        Ok(data
            .into_iter()
            .map(|x| x.get(0))
            .collect::<HashSet<String>>())
    }

    // TODO: make sure .get()'s will actually return correct data (not empty strings)
    // to prevent problems in other functions
    pub fn fetch_file_for_user(&self, discord_id: &i64) -> Result<Picture, Error> {
        // dbg! {"fetching new file"};

        let query = "with curr_id as (
                SELECT user_id FROM users WHERE discord_id = $1
            ),
            previous_ids as (
                SELECT file_id FROM rating WHERE user_id = (SELECT * FROM curr_id)
            ),
            file_to_upload as (
                SELECT files.file_id, files.character_id, files.file_name from files 
                left join (select previous_ids.file_id from previous_ids) as fid on fid.file_id = files.file_id
                where fid.file_id is null
                limit 1
            ),
            character_name as (
                SELECT character_name, anime_id from people where character_id = (select character_id from file_to_upload)
            ),
            anime_name as (
                select anime_name from anime where anime_id = (select anime_id from character_name)
            )

            select (select anime_name from anime_name), (select character_name from character_name), (select file_name from file_to_upload), (select file_id from file_to_upload);
            ";

        let data = self.conn.query(query, &[&discord_id])?;

        // the user has previous never voted on anything, but started the bot
        if data.len() == 0 {
            self.pull_random_file(discord_id)
        } else {

            match row_matcher(data.get(0), self.save_folder.clone()) {
                Ok(good_picture) => Ok(good_picture),
                Err(err) => match err{
                    Error::Postgres(_) | Error::DatabaseParse => self.pull_random_file(discord_id),
                    _ => Err(Error::DatabaseParse)
                }
            }
        }
    }

    fn pull_random_file(&self, discord_id: &i64) -> Result<Picture, Error> {
        // dbg!{"pulling random file"};

        let query = "with curr_file as (
                SELECT file_id, character_id, file_name FROM files limit 1
            ),
            curr_character_name as (
                SELECT character_name, anime_id from people WHERE character_id = (select character_id from curr_file)
            ),
            anime_name as (
                SELECT anime_name FROM anime where anime_id = (select anime_id from curr_character_name) 
            )
            SELECT (SELECT anime_name from anime_name), (SELECT character_name from curr_character_name), (SELECT file_name from curr_file), (select file_id from curr_file);";

        let data = self.conn.query(query, &[])?;

        if data.len() >= 1 {
            row_matcher(data.get(0), self.save_folder.clone())
        } else {
            Err(Error::EmptyQuery)
        }
    }

    pub fn insert_new_rating(
        &self,
        discord_id: i64,
        file_id: i32,
        rating: i16,
    ) -> Result<(), Error> {
        dbg! {"inserting new rating"};

        let query = "with curr_user_id as (
                SELECT user_id FROM users WHERE discord_id = $1
            )
            INSERT INTO rating ( user_id, file_id, score) VALUES ((SELECT user_id from curr_user_id), $2, $3)
            ON CONFLICT (user_id, file_id) DO UPDATE SET score = $3";

        let x = self.conn.query(&query, &[&discord_id, &file_id, &rating]);

        Ok(())
    }

    pub fn need_new_user(&self, discord_id: &i64) -> Result<bool, Error> {
        let query = "SELECT user_id from users where discord_id = $1";

        let data = self.conn.query(query, &[discord_id])?;

        if data.len() == 1 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn create_user(&self, username: &str, discord_id: &i64) -> Result<(), Error> {
        // dbg!{"creating new user"};
        let query = "INSERT INTO users (username, discord_id) VALUES ($1, $2)";
        self.conn.query(query, &[&username, &discord_id])?;

        Ok(())
    }

    pub fn new_anime(&self, anime_name: &str) -> Result<(), Error> {
        // dbg!{"createing new anime"};
        let query = "INSERT INTO anime (anime_name) VALUES ($1)";
        self.conn.query(&query, &[&anime_name])?;

        Ok(())
    }

    pub fn new_character(&self, anime_name: &str, character_name: &str) -> Result<(), Error> {
        // dbg!{"creating new character"};
        let query = "with curr_anime_id as (
            SELECT anime_id FROM anime WHERE anime_name = $1
        )
        INSERT INTO people (anime_id, character_name) VALUES ( (SELECT anime_id FROM curr_anime_id), $2)";
        self.conn.query(&query, &[&anime_name, &character_name])?;

        Ok(())
    }

    pub fn need_to_download(&self, link: &str) -> Result<bool, Error> {
        let query = "SELECT link FROM files WHERE link = $1";
        let data = self.conn.query(&query, &[&link])?;

        if data.len() == 1 {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn insert_new_file(
        &self,
        link: &str,
        file_name: &str,
        anime_name: &str,
        character_name: &str,
    ) -> Result<(), Error> {
        // dbg!{"inserting new file"};
        let query = "with curr_anime_id as (
                select anime_id from anime where anime_name = $1
            ),
            curr_char_id as (
                select character_id from people WHERE character_name = $2 and anime_id = (select * from curr_anime_id)
            )
            INSERT INTO files (link, character_id, file_name) VALUES ($3, (select * from curr_char_id), $4)";

        self.conn
            .query(&query, &[&anime_name, &character_name, &link, &file_name])?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Picture {
    file_name: String,
    character_name: String,
    anime_name: String,
    file_id: i32,
    save_folder: String,
}
impl Picture {
    pub fn file_path(&self) -> String {
        let c = self.file_name.clone();

        let mut save = self.save_folder.clone();
        save.push_str(r"\");
        save.push_str(&c);

        save
    }
    pub fn character(&self) -> &String {
        &self.character_name
    }
    pub fn anime(&self) -> &String {
        &self.anime_name
    }
    pub fn file_id(&self) -> &i32 {
        &self.file_id
    }
}


// (SELECT anime_name from anime_name), (SELECT character_name from curr_character_name), (SELECT file_name from curr_file), (select file_id from curr_file)

/// Checks to make sure that each entry in the row returned from the query is not null.
fn row_matcher(row: postgres::rows::Row, save_folder: String) -> Result<Picture, Error> {
    // dbg!{"attempting to anime name"};
    let anime_name: String = if let Some(anime_name) = row.get_opt(0) {
        anime_name?
    } else {
        return Err(Error::DatabaseParse);
    };

    // dbg!{"attempting to character name"};
    let character_name: String = if let Some(character_name) = row.get_opt(1) {
        character_name?
    } else {
        return Err(Error::DatabaseParse);
    };

    // dbg!{"attempting to match file name"};
    let file_name: String = if let Some(file_name) = row.get_opt(2) {
        file_name?
    } else {
        return Err(Error::DatabaseParse);
    };

    // dbg!{"attempting to file_id"};
    let file_id: i32 = if let Some(file_id) = row.get_opt(3) {
        file_id?
    } else {
        return Err(Error::DatabaseParse);
    };
    // dbg!{"wrapping to sturct"};

    Ok(Picture {
        file_name: file_name,
        character_name: character_name,
        anime_name: anime_name,
        file_id: file_id,
        save_folder: save_folder,
    })
}

#[test]
fn __get_all_urls() {
    let database = Database::new();

    dbg! {&database};

    let database = database.expect("database could not be formed");

    let urls = database.get_all_urls();
    dbg! {&urls};
    let urls = urls.expect("could not get all urls");
}

#[test]
fn __get_new_picture() {
    let database = Database::new();

    dbg! {&database};

    let database = database.expect("database could not be formed");

    let urls = database.fetch_file_for_user("test_user");
    dbg! {&urls};
    let urls = urls.expect("could not get all urls");
}
