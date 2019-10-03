use super::user;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::super::error::Error;
use reqwest;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::path::Path;

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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Data {
    previous_files: HashMap<String, String>,
    users: Users,
}
impl<'a> Data {
    pub fn to_ref(mut self, ser_data: user::Data) -> Result<RefData<'a>, Error> {
        // fetch all anime titles
        let show_names = ser_data
            .shows
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<_>>();

        // fetch all characters
        let characters = ser_data
            .shows
            .iter()
            .map(|x| {
                x.characters
                    .iter()
                    .map(|x| x.name.clone())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        // downloader struct
        let dl = Downloader::new(ser_data.path());

        // fetch every link from the data
        let mut links = Vec::with_capacity(30);
        for show in ser_data.shows.iter() {
            for character in show.characters.iter() {
                for link in character.links.iter() {
                    links.push(link);
                }
            }
        }

        // download and insert new links into the hashmap
        links
            .into_iter()
            .filter(|x| !self.previous_files.contains_key(*x))
            .map(|x| (x, dl.get(x)))
            .filter(|(link, download)| download.is_ok())
            .map(|(x, y)| (x.clone(), y.unwrap()))
            .collect::<HashMap<String, String>>()
            .into_iter()
            .for_each(|(k, v)| {
                self.previous_files.insert(k, v);
            });
        
        let file_names = self.previous_files.iter().map(|(k,v)| v).collect::<Vec<_>>();


        // Ok(RefData{
        //     previous_files: self.previous_files,
        //     file_names: file_names,
        //     anime_names: vec![],
        //     character_names: vec![],
        //     characters: vec![],
        //     users: vec![],

        // })

        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Users {
    username: String,
    characters: Vec<Character>,
    ratings: Vec<Rating>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Rating {
    character: String,
    filename: String,
    score: u8,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Character {
    name: String,
    anime: String,
}

impl std::ops::Deref for Data {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.previous_files
    }
}

// ////////////////////////////////////////////////////// //
// ////////////////////////////////////////////////////// //
// ////////////////////////////////////////////////////// //

#[derive(Serialize, Debug, Clone)]
pub struct RefData<'a> {
    // url, filename
    previous_files: HashMap<String, String>,
    file_names: Vec<&'a String>,
    anime_names: Vec<String>,
    character_names: Vec<String>,
    characters: Vec<RefCharacter<'a>>,
    users: Vec<RefUsers<'a>>,
}
impl<'a> RefData<'a> {
    pub fn new() -> Self {
        Self {
            previous_files: HashMap::new(),
            file_names: vec![],
            characters: vec![],
            users: vec![],
            anime_names: vec![],
            character_names: vec![],
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct RefUsers<'a> {
    username: String,
    ratings: Vec<RefRating<'a>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RefRating<'a> {
    character: &'a String,
    filename: &'a String,
    score: u8,
}

#[derive(Serialize, Debug, Clone)]
pub struct RefCharacter<'a> {
    name: &'a String,
    anime: &'a String,
}
