//! Reads file that need to be downloaded from the internet and places them in the file system
//!

use reqwest;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_yaml;

use super::super::error::Error;
use std::fs;
use std::io::prelude::*;

use super::database;

// pub fn get_files(path: &str) -> Result<cpu::Data, Error> {
//     let reader = fs::File::open(path)?;

//     let ser_data: user::Data = serde_yaml::from_reader(reader)?;

//     // let downloader = Downloader::new(ser_data.path());
//     let mut previous_data = get_older_data(ser_data)?;
//     // let mut previous_hm = previous_data.to_hashmap();

//     // Ok(ser_data)

//     unimplemented!()
// }

// fn get_older_data<'a>(data: user::Data) -> Result<cpu::RefData<'a>, Error> {
//     let save_location = data.path();
//     // create the directory if it does not exist
//     if !save_location.is_dir() {
//         fs::create_dir_all(save_location)?;
//     }

//     let file_path = save_location.join("previously_saved.yaml");

//     let previous = match file_path.is_file() {
//         true => {
//             let file = fs::File::open(&file_path)?;
//             let ser: Result<cpu::Data, _> = serde_yaml::from_reader(file);
//             match ser {
//                 Ok(ser_data) => ser_data.to_ref(data)?,
//                 Err(_) => {
//                     println! {"error when reading from the generated config file"};
//                     cpu::RefData::new()
//                 }
//             }
//         }
//         false => {
//             fs::File::create(&file_path)?;
//             cpu::RefData::new()
//         }
//     };

//     Ok(previous)
// }

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
pub struct Data {
    shows: Vec<Anime>,
    save_location: String,
}

impl Data {
    pub fn path(&self) -> &Path {
        Path::new(&self.save_location)
    }
    pub fn shows(&self) -> &Vec<Anime> {
        &self.shows
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Anime {
    name: String,
    characters: Vec<Character>,
}
impl Anime {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn characters(&self) -> &Vec<Character> {
        &self.characters
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Character {
    name: String,
    links: Vec<String>,
}
impl Character {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn links(&self) -> &Vec<String> {
        &self.links
    }
}
