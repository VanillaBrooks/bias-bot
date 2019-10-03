use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;

use std::fs;
use std::io;

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
pub struct Database <'a> {
    conn: postgres::Connection,
    save_folder: &'a std::path::Path,
}

impl <'a> Database <'a> {
    pub fn new(save_path: &'a std::path::Path) -> Result<Self, Error> {
        let config = DatabaseConfig::new()?;
        let url = config.connection_url();
        let connection = postgres::Connection::connect(url, postgres::TlsMode::None)?;

        Ok(
            Self {
                conn: connection,
                save_folder: save_path,
            }
        )
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
    pub fn fetch_file_for_user(&self, username: &str) -> Result<Picture<'a>, Error> {
        dbg! {"fetching new file"};

        let query = "with curr_id as (
                SELECT user_id FROM users WHERE username = $1
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
                SELECT files.file_name, files.character_id, files.file_id FROM files where (select * from file_to_upload) = files.file_id
            ),
            character_name as (
                SELECT character_name, anime_id from people where character_id = (select character_id from good_file_id)
            ),
            anime_name as (
                select anime_name from anime where anime_id = (select anime_id from character_name)
            )

            select (select * from anime_name), (select character_name from character_name), (select file_name from good_file_id), (select file_id from files), (select user_id from curr_id);";

        let data = self.conn.query(query, &[&username])?;

        let row = data.get(0);

        let anime = row.get(0);
        let character = row.get(1);
        let file_name = row.get(2);
        let file_id: u32 = row.get(3);
        let user_id: u32 = row.get(4);

        Ok(Picture::new(file_name, character, anime, file_id, user_id, self.save_folder))
    }

    pub fn insert_new_rating(&self, picture: &Picture, rating: u8) -> Result<(), Error> {
        dbg! {"inserting new rating"};

        let query = "with curr_user_id as (
                SELECT user_id FROM users WHERE username = {}
            ),
            INSERT INTO rating (user_id, file_id, score) VALUES ($1, $2, $3)";

        let data = self.conn.query(
            &query,
            &[&picture.user_id, &picture.file_id, &(rating as u32)],
        )?;

        Ok(())
    }

    pub fn create_user(&self, username: &str) -> Result<(), Error> {
        // dbg!{"creating new user"};
        let query = "INSERT INTO users (username) VALUES ($1)";
        self.conn.query(query, &[&username])?;

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

        let ans = self
            .conn
            .query(&query, &[&anime_name, &character_name, &link, &file_name])?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Picture <'a> {
    file_name: String,
    pub character_name: String,
    pub anime_name: String,
    file_id: u32,
    user_id: u32,
    save_folder: &'a path::Path
}
impl <'a> Picture  <'a> {
    fn new(path: String, name: String, anime: String, file_id: u32, user_id: u32, save_folder: &'a path::Path) -> Self {
        Self {
            file_name: path,
            character_name: name,
            anime_name: anime,
            file_id: file_id,
            user_id: user_id,
            save_folder: save_folder
        }
    }
    fn file_path(&self) -> path::PathBuf {
        // let c : String = self.file_name.clone();
        // let p = *self.save_folder;

        let mut  c = self.file_name.clone();
        c.push_str(".png");

        self.save_folder.join(c)
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
