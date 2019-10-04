//! Reads file that need to be downloaded from the internet and places them in the file system
//!

#![allow(unused_must_use)]

use reqwest;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::Deserialize;
use serde_yaml;

use super::super::error::Error;
use std::fs;
use std::io::prelude::*;

use super::database;

pub fn get_data_handler<'a>(path: &'a Path) -> Result<database::Database<'a>, Error> {
    let reader = fs::File::open(path)?;
    let ser_data: Data = serde_yaml::from_reader(reader)?;

    let db = database::Database::new(&path)?;

    let download = Downloader::new(ser_data.path());

    for show in &ser_data.shows {
        db.new_anime(&show.name);

        for character in &show.characters {
            db.new_character(&show.name, &character.name);

            for link in &character.links {
                if db.need_to_download(&link)? {
                    let hash = download.get(&link)?;
                    db.insert_new_file(&link, &hash, &show.name, &character.name);
                }
            }
        }
    }

    Ok(db)
}

struct Downloader<'a> {
    client: reqwest::Client,
    path: &'a Path,
}
impl<'a> Downloader<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            client: reqwest::Client::new(),
            path: &path,
        }
    }

    fn get(&self, link: &String) -> Result<String, Error> {
        let mut data = self.client.get(link).send()?;

        let mut buffer = Vec::with_capacity(1024 * 1024);
        data.read_to_end(&mut buffer)?;

        let mut hasher = DefaultHasher::new();
        link.hash(&mut hasher);
        let mut file_name = hasher.finish().to_string();
        file_name.push_str(".png");

        let path = self.path.join(&file_name);

        let mut file = fs::File::create(&path)?;
        file.write_all(&buffer)?;

        return Ok(file_name);
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Data {
    shows: Vec<Anime>,
    save_location: String,
}

impl Data {
    pub fn path(&self) -> &Path {
        Path::new(&self.save_location)
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Anime {
    name: String,
    characters: Vec<Character>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Character {
    name: String,
    links: Vec<String>,
}
