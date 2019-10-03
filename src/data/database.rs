use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;

use std::fs;
use std::io;

use super::super::error::Error;
use postgres;

use std::collections::HashSet;

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
        // let path = r".\database.yaml".to_string();
        let path = std::path::Path::new("./database.yaml");

        let file = fs::File::open(path).expect("database.yaml DOES NOT EXIST");
        let reader = io::BufReader::new(file);

        Ok(serde_yaml::from_reader(reader)?)

        // Ok(x)
    }
    fn connection_url(&self) -> String {
        format! {"postgresql://{}:{}@{}:{}/{}",self.username, self.password, self.address, self.port, self.database_name}
    }
}

#[derive(Debug)]
pub struct Database {
    conn: postgres::Connection,
}

impl Database {
    fn new() -> Result<Self, Error> {
        let config = DatabaseConfig::new()?;
        let url = config.connection_url();
        let connection = postgres::Connection::connect(url, postgres::TlsMode::None)?;

        Ok(Self { conn: connection })
    }

    fn get_all_urls(&self) -> Result<HashSet<String>, Error> {
        let pull = self.conn.prepare("SELECT link FROM files")?;

        let data = pull.query(&[])?;

        Ok(data
            .into_iter()
            .map(|x| x.get(0))
            .collect::<HashSet<String>>())
    }

    fn fetch_file_for_user(&self, username: &str) -> Result<Picture, Error> {
        let query = format! {"with curr_id as (
                SELECT user_id FROM users WHERE username = {}
            ),
            previous_ids as (
                SELECT file_id FROM rating WHERE user_id = (SELECT * FROM curr_id)
            ),
            file_to_upload as (
                SELECT previous_ids.file_id FROM previous_ids
                LEFT JOIN files on files.character_id = previous_ids.file_id 
                WHERE previous_ids.file_id is null
                limit 1
            ),
            good_file_id as (
                SELECT files.file_name, files.character_id FROM files where (select * from file_to_upload) = files.file_id
            ),
            character_name as (
                SELECT character_name, anime_id from people where character_id = (select character_id from good_file_id)
            ),
            anime_name as (
                select anime_name from anime where anime_id = (select anime_id from character_name)
            )

            select (select * from anime_name), (select character_name from character_name), (select file_name from good_file_id)", 
            username
        };

        let data = self.conn.query(&query, &[])?;

        let row = data.get(0);

        let anime = row.get(0);
        let character = row.get(1);
        let file_name = row.get(2);

        Ok(Picture::new(file_name, character, anime))
    }
}

#[derive(Debug)]
pub struct Picture {
    pub file_path: String,
    pub character_name: String,
    pub anime_name: String,
}
impl Picture {
    fn new(path: String, name: String, anime: String) -> Self {
        Self {
            file_path: path,
            character_name: name,
            anime_name: anime
        }
    }
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